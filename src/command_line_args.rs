use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None, bin_name = "datex")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Subcommand)]
pub enum Subcommands {
    Run(Run),
    Lsp(Lsp),
    Repl(Repl),
}

#[derive(Args)]
pub struct Run {
    pub file: Option<String>,
}

#[derive(Args)]
pub struct Lsp {}

#[derive(Args)]
pub struct Repl {}

pub fn get_command() -> Option<Subcommands> {
    Cli::parse().command
}
