use std::ops::Deref;

mod ham {

    use crate::ham::ast_operations::{ExpressionBase, VarDefinitionBase};
    use crate::ham::primitive_values::BooleanValueBase;
    use regex::Regex;

    type LinesList = Vec<Vec<String>>;
    type TokensList = Vec<Token>;

    pub mod tokens_type {
        pub type Val = i32;
        pub const DEFAULT: Val = 0;
        pub const VAR_DEF: Val = 1;
        pub const LEFT_ASSIGN: Val = 2;
        pub const EXPRESSION: Val = 3;
    }

    type PrimitiveValue = String;

    pub mod primitive_values {
        use std::any::Any;

        pub trait PrimitiveValueBase {
            fn get_type(&self) -> String;
            fn as_self(&self) -> &dyn Any;
        }

        pub trait BooleanValueBase {
            fn new(val: bool) -> Boolean;
            fn get_state(&self) -> bool;
        }

        pub struct Boolean(pub bool);

        impl PrimitiveValueBase for Boolean {
            fn get_type(&self) -> String {
                String::from("boolean")
            }
            fn as_self(&self) -> &dyn Any {
                self
            }
        }

        impl BooleanValueBase for Boolean {
            fn new(val: bool) -> Boolean {
                Boolean(val)
            }

            fn get_state(&self) -> bool {
                self.0
            }
        }
    }

    pub mod ast_operations {
        use crate::ham::primitive_values::BooleanValueBase;
        use crate::ham::{primitive_values, tokens_type, PrimitiveValue};
        use std::any::Any;

        /* BASE */
        pub trait AstBase {
            fn get_type(&self) -> tokens_type::Val;
            fn as_self(&self) -> &dyn Any;
        }

        pub struct Ast {
            pub body: Vec<Box<dyn self::AstBase>>,
            pub token_type: tokens_type::Val,
        }

        /* VARIABLE DEFINITION */
        pub trait VarDefinitionBase {
            fn get_def_name(&self) -> String;
            fn new(def_name: String, assignment: Assignment) -> Self;
        }

        pub struct VarDefinition {
            pub def_name: String,
            pub assignment: Assignment,
        }

        impl VarDefinitionBase for VarDefinition {
            fn get_def_name(&self) -> String {
                return self.def_name.clone();
            }
            fn new(def_name: String, assignment: Assignment) -> VarDefinition {
                VarDefinition {
                    def_name,
                    assignment,
                }
            }
        }

        impl AstBase for VarDefinition {
            fn get_type(&self) -> i32 {
                tokens_type::VAR_DEF
            }
            fn as_self(&self) -> &dyn Any {
                self
            }
        }

        /* ASSIGNMENT */

        pub struct Assignment {
            pub interface: PrimitiveValue,
            pub value: Box<dyn self::primitive_values::PrimitiveValueBase>,
        }

        impl AstBase for Assignment {
            fn get_type(&self) -> i32 {
                tokens_type::LEFT_ASSIGN
            }
            fn as_self(&self) -> &dyn Any {
                self
            }
        }

        impl Assignment {
            fn new(&self) -> Assignment {
                Assignment {
                    interface: String::from(""),
                    value: Box::new(self::primitive_values::Boolean::new(false)),
                }
            }
        }

        /* EXPRESSION  */

        pub struct Expression {
            pub body: Vec<Box<dyn self::AstBase>>,
            pub token_type: tokens_type::Val,
        }

        impl AstBase for Expression {
            fn get_type(&self) -> i32 {
                tokens_type::EXPRESSION
            }
            fn as_self(&self) -> &dyn Any {
                self
            }
        }

        pub trait ExpressionBase {
            fn new() -> Self;
        }

        impl ExpressionBase for Expression {
            fn new() -> Expression {
                Expression {
                    token_type: self::tokens_type::EXPRESSION,
                    body: Vec::new(),
                }
            }
        }
    }

    pub struct Token {
        ast_type: i32,
        value: String,
    }

    fn get_lines(code: String) -> LinesList {
        let mut lines = Vec::new();

        // Every line
        for line in code.split("\n") {
            let line = String::from(line);
            let mut line_ast = Vec::new();

            let re = Regex::new(r"[\s+,:]|([()])").unwrap();

            // Every detected word
            for word in re.split(&line) {
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
                    _ => tokens_type::DEFAULT,
                };

                let ast_token = Token {
                    ast_type: token_type,
                    value: word.clone(),
                };

                tokens.push(ast_token);
            }
        }

        return tokens;
    }

    pub fn get_tokens(code: String) -> TokensList {
        let lines = self::get_lines(code);
        return self::transform_into_tokens(lines);
    }

    pub fn get_ast(tokens: TokensList) -> self::ast_operations::Expression {
        let mut ast_tree = self::ast_operations::Expression::new();

        let mut token_n = 0;

        while token_n < tokens.len() {
            let current_token = &tokens[token_n];

            match current_token.ast_type {
                tokens_type::VAR_DEF => {
                    let def_name = String::from(&tokens[token_n + 1].value.clone());

                    let def_value = String::from(&tokens[token_n + 3].value.clone());

                    let def_value_type = match def_value.as_str() {
                        "True" => self::primitive_values::Boolean::new(true),
                        "False" => self::primitive_values::Boolean::new(false),
                        _ => self::primitive_values::Boolean::new(false),
                    };

                    let assignment = ast_operations::Assignment {
                        interface: String::from("number"),
                        value: Box::new(def_value_type),
                    };

                    let mut ast_token = ast_operations::VarDefinition::new(def_name, assignment);

                    ast_tree.body.push(Box::new(ast_token));
                }
                _ => (),
            }

            token_n += 1;
        }

        return ast_tree;
    }
}

fn main() {
    use crate::ham::primitive_values::BooleanValueBase;

    let code = "let test = True \nlet ok = False \n ";

    println!("` \n {} \n`", code);

    let get_ast_tree = ham::get_tokens(String::from(code));
    let ast = ham::get_ast(get_ast_tree);

    for op in ast.body {
        match op.get_type() {
            ham::tokens_type::VAR_DEF => {
                let variable = op
                    .as_self()
                    .downcast_ref::<ham::ast_operations::VarDefinition>()
                    .unwrap();

                let assignment = variable
                    .assignment
                    .value
                    .as_self()
                    .downcast_ref::<ham::primitive_values::Boolean>()
                    .unwrap();

                println!(
                    "[ type: variable | name: {} | value: {:?} ]",
                    variable.def_name,
                    assignment.get_state()
                )
            }
            _ => {
                println!("IDK")
            }
        }
    }
}
