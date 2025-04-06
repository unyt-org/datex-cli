use datex_core::compiler::compile_body;
use datex_core::runtime::{Context, Runtime};
use rustyline::error::ReadlineError;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use datex_core::crypto::crypto_native::CryptoNative;
use datex_core::runtime::global_context::{set_global_context, GlobalContext};

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
                let ctx = Context::default();
                let runtime = Runtime::new(Rc::new(RefCell::new(ctx)));
            }
            Subcommands::Repl(_) => {
                repl();
            }
            Subcommands::Workbench(_) => {
                set_global_context(
                    GlobalContext {
                        crypto: Arc::new(Mutex::new(CryptoNative)),
                    }
                );
                let ctx = Context::default();
                let runtime = Runtime::new(Rc::new(RefCell::new(ctx)));
                workbench::start_workbench(runtime).unwrap()
            }
        }
    }
    // run REPL if no command is provided
    else {
        repl();
    }
}

fn repl() -> Result<(), ReadlineError> {
    let ctx = Context::default();
    let runtime = Runtime::new(Rc::new(RefCell::new(ctx)));

    let mut rl = rustyline::DefaultEditor::new()?;
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let dxb = compile_body(&line);
                if let Err(e) = dxb {
                    println!("Compile Error: {:?}", e);
                    continue;
                }
                let dxb = dxb.unwrap();
                println!("Compiled: {:?}", dxb);
            }
            Err(_) => break,
        }
    }

    Ok(())
}
