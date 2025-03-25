use std::cell::RefCell;
use std::io::{stdout, self, Write, stdin};
use std::rc::Rc;
use datex_core::runtime::{Context, Runtime};

mod command_line_args;
mod lsp;
use command_line_args::{get_command, Command};
use tower_lsp::{LspService, Server};

use crate::lsp::Backend;

#[tokio::main]
async fn main() {
    let command = get_command();

    match command {
        Command::Lsp(lsp) => {
            // println!("Running LSP");
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();
        
            let (service, socket) = LspService::new(|client| Backend { client });
            Server::new(stdin, stdout, socket).serve(service).await;
        },
        Command::Run(run) => {
            if run.file.is_some() {
                println!("File: {}", run.file.unwrap())
            }
            let ctx = Context::default();
            let runtime = Runtime::new(Rc::new(RefCell::new(ctx)));
        }
    }

}


fn repl() {

}