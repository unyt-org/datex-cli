use std::io::{stdout, self, Write, stdin};

use datex_core::{compiler, utils::logger::{Logger, LoggerContext}};

mod command_line_args;
mod lsp;
use command_line_args::{get_command, Command};
use lazy_static::lazy_static;
use tower_lsp::{LspService, Server};

use crate::lsp::Backend;



lazy_static! {
    static ref CTX:LoggerContext = LoggerContext {
        log_redirect: None
    };
}



#[tokio::main]
async fn main() {
    let logger = Logger::new_for_development(&CTX, "DATEX");

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
            logger.success("initialized");
            if run.file.is_some() {
                println!("File: {}", run.file.unwrap())
            }
        }
    }

}


fn repl() {

}