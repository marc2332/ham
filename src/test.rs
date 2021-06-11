use ham::*;

fn main() {
    let code = "
        let okay = 50
        let test = 1
        print(test)
        print(okay)
    ";

    println!("` \n {} \n`", code);

    let get_ast_tree = ham::get_tokens(String::from(code));
    let ast = ham::get_ast(get_ast_tree);
    ham::run_ast(ast);
}