mod ast;
mod ham;
mod runtime;
mod stack;
mod types;
mod utils;

use crate::ast::ast_operations::ExpressionBase;
use crate::stack::Stack;
use clap::{App, Arg, ArgMatches};
use std::fs;
use std::io;
use std::io::BufRead;
use std::sync::Mutex;

static CLI_MSG: &str = ":: Welcome to HAM REPL :: \n";

fn commands<'a>() -> ArgMatches {
    App::new("HAM Interpreter")
        .version("1.0")
        .author("Marc E. <mespinsanz@gmail.com>")
        .subcommand(App::new("repl"))
        .subcommand(
            App::new("run")
                .arg(Arg::new("file").about("Live code interpreter."))
                .arg(
                    Arg::new("show_ast_tree")
                        .about("Displays the AST Tree of the code.")
                        .takes_value(false)
                        .short('t')
                        .long("show-ast-tree"),
                ),
        )
        .get_matches()
}

fn main() {
    let matches = commands();

    match matches.subcommand() {
        Some(("run", run_matches)) => {
            let is_file = run_matches.value_of("file");

            let filename = if is_file.is_some() {
                is_file.unwrap().to_string()
            } else {
                format!(
                    "{}/src/main.ham",
                    std::env::current_dir().unwrap().display()
                )
            };

            // File content
            let filecontent = fs::read_to_string(filename.as_str())
                .expect("Something went wrong reading the file");

            // Tokens
            let tokens = ham::get_tokens(filecontent);

            // Global context
            let global_context = ast::ast_operations::Expression::new();

            // Memory stack
            let stack = Mutex::new(Stack::new(global_context.expr_id.clone()));

            // Ast tree root
            let tree = Mutex::new(global_context);

            // Tree
            ham::move_tokens_into_ast(tokens, &tree);

            if run_matches.is_present("show_ast_tree") {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tree.lock().unwrap().clone()).unwrap()
                );
            }

            // Run (it will always return a Result<Err> because it doesn't return anything)
            ham::run_ast(&tree, &stack);
        }
        Some(("repl", _)) => {
            println!("{}", CLI_MSG);

            // Global context
            let global_context = ast::ast_operations::Expression::new();

            // Memory stack
            let stack = Mutex::new(Stack::new(global_context.expr_id));

            let stdin = io::stdin();

            println!(">");
            for line in stdin.lock().lines() {
                // Code
                let line = String::from(line.unwrap());

                // Tokens
                let tokens = ham::get_tokens(line);

                // Ast tree root
                let tree = Mutex::new(ast::ast_operations::Expression::new());

                // Tree
                ham::move_tokens_into_ast(tokens, &tree);

                // Run (it will always return a Result<Err> because it doesn't return anything)
                ham::run_ast(&tree, &stack);

                println!("  <- ");

                println!(">");
            }
        }
        _ => {}
    }
}
