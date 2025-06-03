use datex_core::compiler::bytecode::{compile_script, compile_template};
use datex_core::datex_values::core_values::endpoint::Endpoint;
use datex_core::decompiler::{apply_syntax_highlighting, decompile_body, DecompileOptions};
use datex_core::runtime::execution::{execute_dxb, ExecutionOptions};
use datex_core::runtime::Runtime;
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::Helper;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};

struct DatexSyntaxHelper;

impl Highlighter for DatexSyntaxHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        std::borrow::Cow::Owned(apply_syntax_highlighting(line.to_string()).unwrap())
    }
    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        true
    }
}

impl Validator for DatexSyntaxHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
    fn validate_while_typing(&self) -> bool {
        true
    }
}
impl Completer for DatexSyntaxHelper {
    type Candidate = String;
}
impl Hinter for DatexSyntaxHelper {
    type Hint = String;
}
impl Helper for DatexSyntaxHelper {}

#[derive(Debug, Clone, Default)]
pub struct ReplOptions {
    pub verbose: bool,
}

pub fn repl(options: ReplOptions) -> Result<(), ReadlineError> {
    let runtime = Runtime::new(Endpoint::default());

    let mut rl = rustyline::Editor::<DatexSyntaxHelper, _>::new()?;
    rl.set_helper(Some(DatexSyntaxHelper));
    rl.enable_bracketed_paste(true);
    rl.set_auto_add_history(true);

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                // special case: if the user entered "clear", clear the console
                if line.trim() == "clear" {
                    rl.clear_screen()?;
                    continue;
                }
                let dxb = compile_script(&line);
                if let Err(e) = dxb {
                    println!("\x1b[31m[Compiler Error] {e}\x1b[0m");
                    continue;
                }
                let dxb = dxb.unwrap();

                // show decompiled code if verbose is enabled
                if options.verbose {
                    let decompiled = decompile_body(&dxb, DecompileOptions {
                        formatted: true,
                        colorized: true,
                        resolve_slots: true,
                    });
                    if let Err(e) = decompiled {
                        println!("\x1b[31m[Decompiler Error] {e}\x1b[0m");
                        continue;
                    }
                    let decompiled = decompiled.unwrap();
                    println!("[Decompiled]: {}", decompiled);
                }

                // execute
                let execution_options = ExecutionOptions {verbose: options.verbose};
                let result = execute_dxb(dxb, execution_options);
                if let Err(e) = result {
                    eprintln!("\x1b[31m[Execution Error] {e}\x1b[0m");
                } else {
                    // compile and decompile value container for printing
                    let compiled_value = compile_template("?", vec![result.unwrap()]).unwrap();
                    let decompiled_value = decompile_body(&compiled_value, DecompileOptions {
                        formatted: true,
                        colorized: true,
                        resolve_slots: true,
                    }).unwrap();
                    println!("< {decompiled_value}");
                }

            }
            Err(_) => break,
        }
    }

    Ok(())
}