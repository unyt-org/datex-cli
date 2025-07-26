use datex_core::crypto::crypto_native::CryptoNative;
use datex_core::runtime::global_context::{set_global_context, DebugFlags, GlobalContext};
use datex_core::runtime::{Runtime, RuntimeConfig};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use datex_core::values::core_values::endpoint::Endpoint;
use datex_core::network::com_hub::InterfacePriority;
use datex_core::network::com_interfaces::default_com_interfaces::websocket::websocket_server_native_interface::WebSocketServerNativeInterface;
use datex_core::utils::time_native::TimeNative;
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::Helper;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use tokio::task::LocalSet;

mod command_line_args;
mod lsp;
mod repl;
mod workbench;

use crate::command_line_args::Repl;
use crate::lsp::Backend;
use crate::repl::{ReplOptions, repl};
use command_line_args::{Subcommands, get_command};
use tower_lsp::{LspService, Server};

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
                let runtime = Runtime::new(RuntimeConfig::default());
            }
            Subcommands::Repl(Repl { verbose, config }) => {
                let options = ReplOptions { verbose, config_path: config };
                repl(options).await.unwrap();
            }
            Subcommands::Workbench(_) => {
                let local = LocalSet::new();
                local.run_until(workbench()).await;
            }
        }
    }
    // run REPL if no command is provided
    else {
        repl(ReplOptions::default()).await.unwrap();
    }
}

async fn workbench() {
    set_global_context(GlobalContext {
        crypto: Arc::new(Mutex::new(CryptoNative)),
        time: Arc::new(Mutex::new(TimeNative)),
        debug_flags: DebugFlags::default(),
    });
    let runtime = Rc::new(RefCell::new(Runtime::new(RuntimeConfig::default())));
    runtime.borrow().start().await;

    // add socket server interface
    let socket_interface = WebSocketServerNativeInterface::new(1234).unwrap();
    runtime
        .borrow()
        .com_hub()
        .add_interface(
            Rc::new(RefCell::new(socket_interface)),
            InterfacePriority::Priority(1),
        )
        .unwrap();

    workbench::start_workbench(runtime).await.unwrap();
}
