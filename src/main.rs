use datex_core::crypto::crypto_native::CryptoNative;
use datex_core::runtime::global_context::{set_global_context, DebugFlags, GlobalContext};
use datex_core::runtime::{Runtime};
use rustyline::error::ReadlineError;
use std::cell::RefCell;
use std::future;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use datex_core::compiler::bytecode::compile_script;
use datex_core::datex_values::core_values::endpoint::Endpoint;
use datex_core::decompiler::{add_syntax_highlighting, decompile_body, DecompileOptions};
use datex_core::network::com_hub::InterfacePriority;
use datex_core::network::com_interfaces::default_com_interfaces::websocket::websocket_server_native_interface::WebSocketServerNativeInterface;
use datex_core::utils::time_native::TimeNative;
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::{Cmd, Helper, KeyEvent};
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::KeyCode::End;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use tokio::task::{spawn_local, LocalSet};

mod command_line_args;
mod lsp;
mod workbench;
use command_line_args::{get_command, Subcommands};
use tower_lsp::{LspService, Server};

use crate::lsp::Backend;

#[tokio::main]
async fn main() {
    let command = get_command();

    if let Some(cmd) = command {
        match cmd {
            Subcommands::Lsp(lsp) => {
                // println!("Running LSP");
                let stdin = tokio::io::stdin();
                let stdout = tokio::io::stdout();

                let (service, socket) = LspService::new(|client| Backend { client });
                Server::new(stdin, stdout, socket).serve(service).await;
            }
            Subcommands::Run(run) => {
                if run.file.is_some() {
                    println!("File: {}", run.file.unwrap())
                }
                let runtime = Runtime::new(Endpoint::default());
            }
            Subcommands::Repl(_) => {
                repl();
            }
            Subcommands::Workbench(_) => {
                let local = LocalSet::new();
                local.run_until(workbench()).await;
            }
        }
    }
    // run REPL if no command is provided
    else {
        repl();
    }
}

async fn workbench() {
    set_global_context(GlobalContext {
        crypto: Arc::new(Mutex::new(CryptoNative)),
        time: Arc::new(Mutex::new(TimeNative)),
        debug_flags: DebugFlags::default(),
    });
    let runtime = Rc::new(RefCell::new(Runtime::new(Endpoint::random())));
    runtime.borrow().start().await;

    // add socket server interface
    let socket_interface = WebSocketServerNativeInterface::new(1234).unwrap();
    runtime.borrow().com_hub.add_interface(Rc::new(RefCell::new(socket_interface)), InterfacePriority::Priority(1)).unwrap();

    workbench::start_workbench(runtime).await.unwrap();
}


struct DatexSyntaxHelper;

impl Highlighter for DatexSyntaxHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        std::borrow::Cow::Owned(add_syntax_highlighting(line.to_string()).unwrap())
    }
    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        true
    }
}

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

fn repl() -> Result<(), ReadlineError> {
    let runtime = Runtime::new(Endpoint::default());

    let mut rl = rustyline::Editor::<DatexSyntaxHelper, _>::new()?;
    rl.set_helper(Some(DatexSyntaxHelper));
    rl.enable_bracketed_paste(true);
    rl.set_auto_add_history(true);

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let dxb = compile_script(&line);
                if let Err(e) = dxb {
                    println!("Compile Error: {:?}", e);
                    continue;
                }
                let dxb = dxb.unwrap();
                let decompiled = decompile_body(&dxb, DecompileOptions {
                    formatted: true,
                    colorized: true,
                    resolve_slots: true,
                });
                if let Err(e) = decompiled {
                    println!("Decompile Error: {:?}", e);
                    continue;
                }
                let decompiled = decompiled.unwrap();
                println!("Decompiled: {}", decompiled);
            }
            Err(_) => break,
        }
    }

    Ok(())
}