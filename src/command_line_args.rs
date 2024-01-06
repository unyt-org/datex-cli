use clap::{Parser, Subcommand, Args};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Run(Run),
    Lsp(Lsp)
}

#[derive(Args)]
pub struct Run {
   pub file: Option<String>
}

#[derive(Args)]
pub struct Lsp {
}



pub fn get_command() -> Command {
	return Cli::parse().command;
}