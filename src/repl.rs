use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use datex_core::crypto::crypto_native::CryptoNative;
use datex_core::decompiler::{DecompileOptions, apply_syntax_highlighting, decompile_value};
use datex_core::run_async;
use datex_core::runtime::{Runtime, RuntimeConfig};
use datex_core::runtime::execution_context::{ExecutionContext, ScriptExecutionError};
use datex_core::runtime::global_context::{set_global_context, GlobalContext};
use datex_core::utils::time_native::TimeNative;
use datex_core::values::core_values::endpoint::Endpoint;
use datex_core::values::serde::deserializer::DatexDeserializer;
use datex_core::values::serde::error::SerializationError;
use datex_core::values::serde::serializer::DatexSerializer;
use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use serde::Deserialize;

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

pub async fn repl(options: ReplOptions) -> Result<(), ReplError> {

    set_global_context(GlobalContext::new(
        Arc::new(Mutex::new(CryptoNative)),
        Arc::new(Mutex::new(TimeNative)),
    ));

    let config = match options.config_path {
        Some(path) => read_config_file(path)?,
        None => RuntimeConfig::new_with_endpoint(Endpoint::random()),
    };

    run_async! {
        let runtime = Runtime::create_native(config).await;

        let mut rl = rustyline::Editor::<DatexSyntaxHelper, _>::new()?;
        rl.set_helper(Some(DatexSyntaxHelper));
        rl.enable_bracketed_paste(true);
        rl.set_auto_add_history(true);

        // create context
        let mut execution_context = if options.verbose {
            ExecutionContext::local_debug()
        } else {
            ExecutionContext::local()
        };
        loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    // special case: if the user entered "clear", clear the console
                    if line.trim() == "clear" {
                        rl.clear_screen()?;
                        continue;
                    }

                    let result = runtime.execute(&line, &[], Some(&mut execution_context)).await;

                    if let Err(e) = result {
                        match e {
                            ScriptExecutionError::CompilerError(e) => {
                                println!("\x1b[31m[Compiler Error] {e}\x1b[0m");
                            }
                            ScriptExecutionError::ExecutionError(e) => {
                                println!("\x1b[31m[Execution Error] {e}\x1b[0m");
                            }
                        }
                        continue;
                    }

                    if let Some(result) = result.unwrap() {
                        let decompiled_value = decompile_value(&result, DecompileOptions::colorized());
                        println!("< {decompiled_value}");
                    }
                }
                Err(_) => break,
            }
        }

        Ok(())
    }
}
