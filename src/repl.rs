use datex_core::compiler::bytecode::{compile_script, compile_template, CompileOptions, CompileScope};
use datex_core::datex_values::core_values::endpoint::Endpoint;
use datex_core::decompiler::{apply_syntax_highlighting, decompile_body, DecompileOptions};
use datex_core::runtime::execution::{execute_dxb, ExecutionContext, ExecutionInput, ExecutionOptions};
use datex_core::runtime::{execution, Runtime};
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
    let mut compile_scope = CompileScope::default();
    let execution_options = ExecutionOptions {
        verbose: options.verbose,
        ..ExecutionOptions::default()
    };
    let mut execution_context = ExecutionContext::default();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                // special case: if the user entered "clear", clear the console
                if line.trim() == "clear" {
                    rl.clear_screen()?;
                    continue;
                }
                // TODO: no clone for compile_scope
                let compile_result = compile_script(&line, CompileOptions::new_with_scope(compile_scope.clone()));
                if let Err(e) = compile_result {
                    println!("\x1b[31m[Compiler Error] {e}\x1b[0m");
                    continue;
                }
                let compile_result = compile_result.unwrap();
                let dxb = compile_result.0;
                compile_scope = compile_result.1;
                // show decompiled code if verbose is enabled
                if options.verbose {
                    println!("\x1b[32m[Compilation Result] {}", dxb.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(", "));

                    let decompiled = decompile_body(&dxb, DecompileOptions {
                        formatted: true,
                        colorized: true,
                        resolve_slots: true,
                        json_compat: false
                    });
                    if let Err(e) = decompiled {
                        println!("\x1b[31m[Decompiler Error] {e}\x1b[0m");
                        continue;
                    }
                    let decompiled = decompiled.unwrap();
                    println!("[Decompiled]: {}", decompiled);
                }

                // execute
                // update dxb in execution context
                // set index in execution_context to 0
                execution_context.reset_index();
                let execution_input = ExecutionInput {
                    // FIXME: no clone here
                    context: execution_context.clone(),
                    options: execution_options.clone(),
                    dxb_body: &dxb
                };
                let result = execute_dxb(execution_input);
                if let Err(e) = result {
                    eprintln!("\x1b[31m[Execution Error] {e}\x1b[0m");
                    continue;
                }

                let result = result.unwrap();
                execution_context = result.1;

                if let Some(result) = result.0 {
                    // compile and decompile value container for printing
                    let (compiled_value, _) = compile_template("?", &[result], CompileOptions::default()).unwrap();
                    if options.verbose {
                        println!("\x1b[32m[Compilation Result] {}", compiled_value.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(", "));
                    }
                    let decompiled_value = decompile_body(&compiled_value, DecompileOptions {
                        formatted: true,
                        colorized: true,
                        resolve_slots: true,
                        json_compat: false,
                    }).unwrap();
                    println!("< {decompiled_value}");
                }

            }
            Err(_) => break,
        }
    }

    Ok(())
}