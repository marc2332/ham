mod ast;
mod ham;
mod runtime;
mod stack;
mod types;
mod utils;

use crate::ast::ast_operations::ExpressionBase;
use crate::stack::Stack;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::fs;
use std::io;
use std::io::BufRead;
use std::sync::Mutex;

static CLI_MSG: &str = ":: Welcome to HAM REPL :: \n";

fn commands<'a>() -> ArgMatches<'a> {
    App::new("HAM Interpreter")
        .version("1.0")
        .author("Marc E. <mespinsanz@gmail.com>")
        .subcommand(SubCommand::with_name("repl"))
        .subcommand(SubCommand::with_name("run").arg(Arg::with_name("file")))
        .get_matches()
}

fn main() {
    let matches = commands();

    match matches.subcommand_name() {
        Some("run") => {
            let filename = matches.subcommand().1.unwrap().value_of("file").unwrap();

            let contents =
                fs::read_to_string(filename).expect("Something went wrong reading the file");

            // Memory stack
            let stack = Mutex::new(Stack::new());

            // Tokens
            let tokens = ham::get_tokens(contents);

            // Ast tree root
            let tree = Mutex::new(ast::ast_operations::Expression::new());

            // Tree
            ham::move_tokens_into_ast(tokens, &tree);

            // Run (it will always return a Result<Err> because it doesn't return anything)
            ham::run_ast(&tree, &stack).err();
        }
        Some("repl") => {
            println!("{}", CLI_MSG);

            // Memory stack
            let stack = Mutex::new(Stack::new());

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
                ham::run_ast(&tree, &stack).unwrap();

                println!("  <- ");

                println!(">");
            }
        }
        _ => {}
    }
}
