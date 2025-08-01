use datex_core::crypto::crypto_native::CryptoNative;
use datex_core::runtime::global_context::{set_global_context, DebugFlags, GlobalContext};
use datex_core::runtime::{Runtime, RuntimeConfig};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use datex_core::network::com_hub::InterfacePriority;
use datex_core::network::com_interfaces::default_com_interfaces::websocket::websocket_server_native_interface::WebSocketServerNativeInterface;
use datex_core::run_async;
use datex_core::utils::time_native::TimeNative;
use datex_core::values::serde::error::SerializationError;
use tokio::task::LocalSet;

mod command_line_args;
mod lsp;
mod repl;
mod workbench;
mod utils;

use crate::command_line_args::Repl;
use crate::lsp::Backend;
use crate::repl::{ReplOptions, repl};
use command_line_args::{Subcommands, get_command};
use tower_lsp::{LspService, Server};
use crate::utils::config::create_runtime_with_config;

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
                workbench(None).await.expect("Workbench failed");
            }
        }
    }
    // run REPL if no command is provided
    else {
        repl(ReplOptions::default()).await.unwrap();
    }
}

async fn workbench(config_path: Option<PathBuf>) -> Result<(), SerializationError> {
    set_global_context(GlobalContext {
        crypto: Arc::new(Mutex::new(CryptoNative)),
        time: Arc::new(Mutex::new(TimeNative)),
        debug_flags: DebugFlags::default(),
    });
    
    run_async! {
        let runtime = create_runtime_with_config(config_path).await?;
        workbench::start_workbench(runtime).await?;
        
        Ok(())
    }
}
