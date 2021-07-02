use clap::{
    App,
    Arg,
    ArgMatches,
};
use ham_core::{
    ast_types::expression::{
        Expression,
        ExpressionBase,
    },
    stack::Stack,
};
use ham_manager::Manifest;
use question::Question;
use std::{
    fs,
    path::Path,
    sync::Mutex,
};

fn commands() -> ArgMatches {
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

    // CWD
    let cwd = std::env::current_dir().unwrap().display().to_string();

    // Global context
    let global_context = Expression::new();

    // Memory stack
    let stack = Mutex::new(Stack::new(global_context.expr_id));

    loop {
        let answer = Question::new(">").ask().unwrap();

        match answer {
            question::Answer::RESPONSE(line) => {
                // Tokens
                let tokens = ham_core::get_tokens(line);

                // Ast tree root
                let tree = Mutex::new(Expression::new());

                // Tree
                ham_core::move_tokens_into_ast(tokens, &tree, cwd.clone());

                // Run the code
                ham_core::run_ast(&tree, &stack);

                println!("  <-");
            }
            question::Answer::NO | question::Answer::YES => {}
        }
    }
}

fn main() {
    let matches = commands();

    match matches.subcommand() {
        Some(("run", run_matches)) => {
            // CWD
            let cwd = std::env::current_dir().unwrap().display().to_string();

            let filename = run_matches.value_of("file");

            // Manifest file
            let _manifest = {
                let manifest_file = format!("{}/ham.yml", cwd);

                Manifest::from_file(manifest_file.as_str())
            };

            // Main file
            let filename = if let Some(filename) = filename {
                format!("{}/{}", cwd, filename.to_string())
            } else {
                format!("{}/src/main.ham", cwd)
            };

            // Main file content
            let filecontent = fs::read_to_string(filename.as_str())
                .expect("Something went wrong reading the file");

            // Tokens
            let tokens = ham_core::get_tokens(filecontent);

            // Global context
            let global_context = Expression::new();

            // Memory stack
            let stack = Mutex::new(Stack::new(global_context.expr_id.clone()));

            // Ast tree root
            let tree = Mutex::new(global_context);

            // File's folder
            let mut filedir = Path::new(filename.as_str()).ancestors();
            filedir.next().unwrap();

            let filedir = filedir.next().unwrap();
            let filedir = filedir.to_str().unwrap().to_string();

            // Tree
            ham_core::move_tokens_into_ast(tokens, &tree, filedir);

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
