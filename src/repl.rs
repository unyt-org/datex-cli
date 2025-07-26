use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;
use datex_core::crypto::crypto_native::CryptoNative;
use datex_core::decompiler::{DecompileOptions, apply_syntax_highlighting, decompile_value};
use datex_core::network::com_interfaces::default_com_interfaces::websocket::websocket_common::WebSocketClientInterfaceSetupData;
use datex_core::run_async;
use datex_core::runtime::{Runtime, RuntimeConfig};
use datex_core::runtime::execution_context::{ExecutionContext, ScriptExecutionError};
use datex_core::runtime::global_context::{set_global_context, GlobalContext};
use datex_core::utils::time_native::TimeNative;
use datex_core::values::core_values::endpoint::Endpoint;
use datex_core::values::serde::deserializer::DatexDeserializer;
use datex_core::values::serde::error::SerializationError;
use datex_core::values::serde::serializer::{to_value_container, DatexSerializer};
use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use serde::Deserialize;
use tokio::time::sleep;

struct DatexSyntaxHelper;

impl Highlighter for DatexSyntaxHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        std::borrow::Cow::Owned(apply_syntax_highlighting(line.to_string()).unwrap())
    }
    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        true
    }
}

// ref x = {}
// val x = (1,2,3,r);
// val y: ((string|decimal): number)  = ("sadf":234)
// const val x = 10;
// ref x = {};
// x.a = 10;
// ref y = (1,2,3); // Map
// y.x = 10;
// func (1,2,3)

// ref weather: Weather;
// weather = getWeatherFromApi(); -> val
// weather = always cpnvertWearth(getWeatherFromApi()); -> indirect copy

// ref user: User; <-- $user
// #0 <- $user
// for name in endpoint (
//    user = resolveInner/innerRef/collapse/resolve getUserFromApi(name); $a -> $b -> $c;
// )
// user // <- $x
// val x = 10;

// ref x = weather;

// (1: x) == ($(1): x, 1: x)
// (val string: any)
// {x: 1} == {0: x, (0min): 20m}
// x.y  -> (y: 34)
// x.({a}) -> ({a}: 4)

impl Validator for DatexSyntaxHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
    fn validate_while_typing(&self) -> bool {
        true
    }
}
impl Completer for DatexSyntaxHelper {
    type Candidate = String;
}
impl Hinter for DatexSyntaxHelper {
    type Hint = String;
}
impl Helper for DatexSyntaxHelper {}

#[derive(Debug, Clone, Default)]
pub struct ReplOptions {
    pub verbose: bool,
    pub config_path: Option<PathBuf>,
}

pub fn read_config_file(path: PathBuf) -> Result<RuntimeConfig, SerializationError> {
    let deserializer = DatexDeserializer::from_dx_file(path)?;
    let config: RuntimeConfig = Deserialize::deserialize(deserializer)?;
    Ok(config)
}

#[derive(Debug)]
pub enum ReplError {
    ReadlineError(ReadlineError),
    SerializationError(SerializationError),
}

impl From<ReadlineError> for ReplError {
    fn from(err: ReadlineError) -> Self {
        ReplError::ReadlineError(err)
    }
}
impl From<SerializationError> for ReplError {
    fn from(err: SerializationError) -> Self {
        ReplError::SerializationError(err)
    }
}

fn get_dx_files(base_path: PathBuf) -> Result<Vec<PathBuf>, ReplError> {
    let mut config_dir = base_path.clone();
    config_dir.push(".datex");

    // Create the directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| {
            ReplError::SerializationError(SerializationError(e.to_string()))
        })?;
    }

    // Collect all files ending with `.dx`
    let dx_files = fs::read_dir(&config_dir)
        .map_err(|e| ReplError::SerializationError(SerializationError(e.to_string())))?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("dx") {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect();

    Ok(dx_files)
}

fn create_new_config_file(base_path: PathBuf, endpoint: Endpoint) -> Result<PathBuf, ReplError> {
    let mut config = RuntimeConfig::new_with_endpoint(endpoint.clone());

    // add default interface
    config.add_interface("websocket-client".to_string(), WebSocketClientInterfaceSetupData {
        address: "wss://example.unyt.land".to_string(),
    })?;

    let mut config_path = base_path.clone();
    config_path.push(".datex");
    config_path.push(format!("{}.dx", endpoint.to_string()));
    let config = to_value_container(&config)?;
    let datex_script = decompile_value(&config, DecompileOptions {formatted: true, ..DecompileOptions::default()});
    fs::write(config_path.clone(), datex_script).map_err(|e| {
        ReplError::SerializationError(SerializationError(e.to_string()))
    })?;

    println!("Created new config file for {} at {:?}", endpoint, config_path);

    Ok(config_path)
}

pub async fn repl(options: ReplOptions) -> Result<(), ReplError> {

    set_global_context(GlobalContext::new(
        Arc::new(Mutex::new(CryptoNative)),
        Arc::new(Mutex::new(TimeNative)),
    ));

    let config = match options.config_path {
        Some(path) => read_config_file(path)?,
        None => {
            match home::home_dir() {
                Some(path) if !path.as_os_str().is_empty() => {
                    // get all .dx files in the home directory .datex folder
                    let dx_files = get_dx_files(path.clone())?;
                    // if no files yet, create a new config file for a random endpoint
                    if dx_files.is_empty() {
                        let endpoint = Endpoint::random();
                        let config_path = create_new_config_file(path.clone(), endpoint)?;
                        read_config_file(config_path)?
                    }
                    else {
                        // if there are files, read the first one
                        let config_path = dx_files.first().unwrap().clone();
                        read_config_file(config_path)?
                    }
                },
                _ => {
                    eprintln!("Unable to get home directory, using temporary endpoint.");
                    RuntimeConfig::new_with_endpoint(Endpoint::random())
                }
            }
        },
    };

    let (cmd_sender, mut cmd_receiver) = tokio::sync::mpsc::channel::<ReplCommand>(100);
    let (response_sender, response_receiver) = tokio::sync::mpsc::channel::<ReplResponse>(100);
    repl_loop(cmd_sender, response_receiver)?;

    run_async! {
        let runtime = Runtime::create_native(config).await;


        // create context
        let mut execution_context = if options.verbose {
            ExecutionContext::local_debug()
        } else {
            ExecutionContext::local()
        };

        while let Some(command) = cmd_receiver.recv().await {
            match command {
                ReplCommand::Debug => {
                    let metadata = runtime.com_hub().get_metadata().to_string();
                    response_sender.send(ReplResponse::Result(Some(metadata))).await.unwrap();
                }
                ReplCommand::Execute(line) => {
                    let result = runtime.execute(&line, &[], Some(&mut execution_context)).await;

                    let mut result_string = None;

                    if let Err(e) = result {
                        match e {
                            ScriptExecutionError::CompilerError(e) => {
                                result_string = Some(format!("\x1b[31m[Compiler Error] {e}\x1b[0m"));
                            }
                            ScriptExecutionError::ExecutionError(e) => {
                                result_string = Some(format!("\x1b[31m[Execution Error] {e}\x1b[0m"));
                            }
                        }
                    }

                    else if let Some(result) = result.unwrap() {
                        let decompiled_value = decompile_value(&result, DecompileOptions::colorized());
                        result_string = Some(format!("< {decompiled_value}"));
                    }
                    else {
                        result_string = None;
                    }

                    response_sender.send(ReplResponse::Result(result_string)).await.unwrap();
                }
            }
        }

        Ok(())
    }
}


enum ReplCommand {
    Debug,
    Execute(String),
}

enum ReplResponse {
    Result(Option<String>),
}

fn repl_loop(
    sender: tokio::sync::mpsc::Sender<ReplCommand>,
    mut receiver: tokio::sync::mpsc::Receiver<ReplResponse>,
) -> Result<(), ReplError> {

    let mut rl = rustyline::Editor::<DatexSyntaxHelper, _>::new()?;
    rl.set_helper(Some(DatexSyntaxHelper));
    rl.enable_bracketed_paste(true);
    rl.set_auto_add_history(true);

    spawn(move || {
        loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    match line.trim() {
                        "clear" => {
                            rl.clear_screen().unwrap();
                            continue;
                        },
                        "debug" => {
                            sender.blocking_send(ReplCommand::Debug).unwrap();
                        },
                        _ => {
                            sender.blocking_send(ReplCommand::Execute(line.clone())).unwrap();
                        }
                    }
                }
                Err(_) => break,
            }

            let response = receiver.blocking_recv();
            match response {
                Some(ReplResponse::Result(result)) => {
                    if let Some(result) = result {
                        println!("{result}");
                    }
                }
                None => { break; }
            }
        }
    });

    Ok(())
}
