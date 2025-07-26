use std::path::{Path, PathBuf};
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
    Workbench(Workbench),
}

#[derive(Args)]
pub struct Run {
    pub file: Option<String>,
}

#[derive(Args)]
pub struct Lsp {}

#[derive(Args)]
pub struct Repl {
    /// Verbose mode for debugging
    #[arg(short, long)]
    pub verbose: bool,
    /// optional path to dx config file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[derive(Args)]
pub struct Workbench {}

pub fn get_command() -> Option<Subcommands> {
    Cli::parse().command
}
