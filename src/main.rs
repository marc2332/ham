mod ast;
mod ham;
mod tokenizer;
mod utils;

use std::io;
use std::io::BufRead;
use std::sync::Mutex;

static CLI_MSG: &str = ":: Welcome to HAM REPL :: \n";

fn main() {
    println!("{}", CLI_MSG);

    // Memory stack
    let stack = Mutex::new(ham::Stack::new());

    let stdin = io::stdin();

    println!(">");
    for line in stdin.lock().lines() {
        // Code
        let line = String::from(line.unwrap());

        // Tokens
        let tokens = ham::get_tokens(line);

        // Tree
        let ast = ham::get_ast(tokens);

        // Run
        ham::run_ast(ast, &stack);

        println!("  <- ");

        println!(">");
    }
}
