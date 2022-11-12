use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   #[arg(short, long)]
   pub file: Option<String>,
}

pub fn get_args() -> Args {
	return Args::parse();
}