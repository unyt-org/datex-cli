use datex_core::compiler;

mod command_line_args;
use command_line_args::get_args;




fn main() {
    let args = get_args();
    println!("Hello DATEX!");

    if args.file.is_some() {
        println!("File: {}", args.file.unwrap())
    }

    repl();
}


fn repl(){

}