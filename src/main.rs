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
use datex_core::decompiler::{decompile_body, DecompileOptions};
use datex_core::network::com_hub::InterfacePriority;
use datex_core::network::com_interfaces::default_com_interfaces::websocket::websocket_server_native_interface::WebSocketServerNativeInterface;
use datex_core::utils::time_native::TimeNative;
use rustyline::KeyCode::End;
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

fn repl() -> Result<(), ReadlineError> {
    let runtime = Runtime::new(Endpoint::default());

    let mut rl = rustyline::DefaultEditor::new()?;
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
                let decompiled = decompile_body(&dxb, DecompileOptions::default());
                if let Err(e) = decompiled {
                    println!("Decompile Error: {:?}", e);
                    continue;
                }
                let decompiled = decompiled.unwrap();
                println!("Decompiled: {:?}", decompiled);
            }
            Err(_) => break,
        }
    }

    Ok(())
}
