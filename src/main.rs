
mod ham {

    use regex::Regex;

    type LinesList = Vec<Vec<String>>;
    type TokensList = Vec<Token>;

    mod tokens_type {
        pub type Val = i32;
        pub const DEFAULT: Val = 0;
        pub const VAR_DEF: Val = 1;
        pub const LEFT_ASSIGN: Val = 2;
    }

    type PrimitiveValue = String;



    pub struct Ast {
        body: Vec<Ast>,
        value: String
    }

    mod ast_operations {
        use crate::ham::{PrimitiveValue, Ast};

        pub struct Assignment {
        pub interface: PrimitiveValue,
        pub value: i32
    }
}

    impl Ast {
        fn new() -> Self {
            Ast {
                body: Vec::new(),
                value: String::from("")
            }
        }
    }

    #[derive(Debug)]
    pub struct Token {
        ast_type: i32,
        value: String
    }

    fn get_lines(code: String) -> LinesList {
        let mut lines = Vec::new();

        // Every line
        for line in code.split("\n"){

            let line = String::from(line);
            let mut line_ast = Vec::new();

            let re = Regex::new(r"[\s+,:]|([()])").unwrap();

            // Every detected word
            for word in re.split(&line){
                // Prevent empty words
                if word != "" {
                    line_ast.push(String::from(word))
                }

            }
            lines.push(line_ast);
        }

        return lines;
    }

    fn transform_into_tokens(lines: LinesList) -> TokensList {
        let mut tokens = Vec::new();

        for line in lines {
            for word in line {

                let word = String::from(word);

                let token_type: tokens_type::Val = match word.as_str() {
                    "let" => tokens_type::VAR_DEF,
                    "=" => tokens_type::LEFT_ASSIGN,
                    _ => tokens_type::DEFAULT
                };

                let ast_token = Token {
                    ast_type: token_type,
                    value: word.clone()
                };

                println!("{:?}", ast_token);

                tokens.push(ast_token);

            }
        }

        return tokens;
    }

    pub fn get_tokens(code: String) -> TokensList {
        let lines = self::get_lines(code);
        return self::transform_into_tokens(lines);
    }


    pub fn get_ast(tokens: TokensList) -> Ast{

        let mut ast_tree = Ast::new();

        let mut token_n = 0;

        while token_n < tokens.len() {
            let current_token = &tokens[token_n];

            match current_token.ast_type {
                tokens_type::VAR_DEF => {
                    let mut ast_token = Ast::new();
                    let def_name = &tokens[token_n+1];
                    let def_value = &tokens[token_n+3];

                    ast_token.value = def_name.value.clone();

                    ast_token.body.push(ast_operations::Assignment {
                        interface: String::from("number"),
                        value: 0
                    });

                    ast_tree.body.push(ast_token);
                }
                _ => ()
            }

            token_n += 1;
        }

        return ast_tree
    }

}

fn main() {

    let code = "\
     let test = false
    ";

    let get_ast_tree = ham::get_tokens(String::from(code));
    let ast = ham::get_ast(get_ast_tree);


    //let tokens = ham::get_tokens(tokens);

}
