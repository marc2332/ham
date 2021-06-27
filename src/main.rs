use clap::{App, Arg, ArgMatches};
use ham_core::ast;
use ham_core::ast::ast_operations::ExpressionBase;
use ham_core::stack::Stack;
use question::Question;
use std::fs;
use std::sync::Mutex;

fn commands<'a>() -> ArgMatches {
    App::new("ham")
        .version("0.0.2")
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

fn run_repl() {
    let cli_welcome = format!(":: ham REPL ({}) ::", env!("CARGO_PKG_VERSION"));
    let cli_tip = "Use Ctrl+C to exit.";

    println!("{}\n{}", cli_welcome, cli_tip);

    // Global context
    let global_context = ast::ast_operations::Expression::new();

    // Memory stack
    let stack = Mutex::new(Stack::new(global_context.expr_id));

    loop {
        let answer = Question::new(">").ask().unwrap();

        match answer {
            question::Answer::RESPONSE(line) => {
                // Code
                let line = String::from(line);

                // Tokens
                let tokens = ham_core::get_tokens(line);

                // Ast tree root
                let tree = Mutex::new(ast::ast_operations::Expression::new());

                // Tree
                ham_core::move_tokens_into_ast(tokens, &tree);

                // Run the code
                ham_core::run_ast(&tree, &stack);

                print!("  <-\n");
            }
            question::Answer::NO | question::Answer::YES => {}
        }
    }
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
            let tokens = ham_core::get_tokens(filecontent);

            // Global context
            let global_context = ast::ast_operations::Expression::new();

            // Memory stack
            let stack = Mutex::new(Stack::new(global_context.expr_id.clone()));

            // Ast tree root
            let tree = Mutex::new(global_context);

            // Tree
            ham_core::move_tokens_into_ast(tokens, &tree);

            if run_matches.is_present("show_ast_tree") {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tree.lock().unwrap().clone()).unwrap()
                );
            }

            ham_core::run_ast(&tree, &stack);
        }
        Some(("repl", _)) => {
            run_repl();
        }
        _ => {
            // Default to repl
            run_repl();
        }
    }
}
