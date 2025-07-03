use datex_core::values::core_values::endpoint::Endpoint;
use datex_core::decompiler::{apply_syntax_highlighting, decompile_value, DecompileOptions};
use datex_core::runtime::{Runtime};
use datex_core::runtime::execution_context::{ExecutionContext, ScriptExecutionError};
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

// ref x = {}
// val x = (1,2,3,r);
// val y: ((string|decimal): number)  = ("sadf":234)
// const val x = 10;
// ref x = {};
// x.a = 10;
// ref y = (1,2,3); // Map
// y.x = 10;
// func (1,2,3)

// ref weather: Weather;
// weather = getWeatherFromApi(); -> val
// weather = always cpnvertWearth(getWeatherFromApi()); -> indirect copy

// ref user: User; <-- $user
// #0 <- $user
// for name in endpoint (
//    user = resolveInner/innerRef/collapse/resolve getUserFromApi(name); $a -> $b -> $c;
// )
// user // <- $x
// val x = 10;

// ref x = weather;

// (1: x) == ($(1): x, 1: x)
// (val string: any)
// {x: 1} == {0: x, (0min): 20m}
// x.y  -> (y: 34)
// x.({a}) -> ({a}: 4)

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

    // create context
    let mut execution_context = if options.verbose { ExecutionContext::local_debug() } else { ExecutionContext::local() };
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                // special case: if the user entered "clear", clear the console
                if line.trim() == "clear" {
                    rl.clear_screen()?;
                    continue;
                }

                let result = execution_context.execute_local(&line, &[]);

                if let Err(e) = result {
                    match e {
                        ScriptExecutionError::CompilerError(e) => {
                            println!("\x1b[31m[Compiler Error] {e}\x1b[0m");
                        }
                        ScriptExecutionError::ExecutionError(e) => {
                            println!("\x1b[31m[Execution Error] {e}\x1b[0m");
                        }
                    }
                    continue;
                }

                if let Some(result) = result.unwrap() {
                    let decompiled_value = decompile_value(&result, DecompileOptions::colorized());
                    println!("< {decompiled_value}");
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}