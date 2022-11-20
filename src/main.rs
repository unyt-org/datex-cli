use datex_core::{compiler, utils::logger::{Logger, LoggerContext}};

mod command_line_args;
use command_line_args::get_args;
use lazy_static::lazy_static;

lazy_static! {
    static ref CTX:LoggerContext = LoggerContext {
        log_redirect: None
    };
}




fn main() {
    let logger = Logger::new_for_development(&CTX, "DATEX");
    logger.success("initialized");

    let args = get_args();
    
    if args.file.is_some() {
        println!("File: {}", args.file.unwrap())
    }

    repl();
}


fn repl(){

}