type LinesList = Vec<Vec<String>>;
type TokensList = Vec<Token>;
type PrimitiveValue = String;

pub mod tokens_type {
    pub type Val = i32;
    pub const DEFAULT: Val = 0;
    pub const VAR_DEF: Val = 1;
    pub const LEFT_ASSIGN: Val = 2;
    pub const EXPRESSION: Val = 3;
}

pub mod primitive_values {
    use std::any::Any;

    pub trait PrimitiveValueBase {
        fn get_type(&self) -> String;
        fn as_self(&self) -> &dyn Any;
    }

    // NUMBER

    pub struct Number(pub i32);

    // Implement base methods for Number
    impl PrimitiveValueBase for Number {
        fn get_type(&self) -> String {
            String::from("number")
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for Number
    pub trait NumberValueBase {
        fn new(val: i32) -> Number;
        fn get_state(&self) -> i32;
    }

    impl NumberValueBase for Number {
        fn new(val: i32) -> Number {
            Number(val)
        }

        fn get_state(&self) -> i32 {
            self.0
        }
    }

    // BOOLEAN

    pub struct Boolean(pub bool);

    // Implement base methods for Boolean
    impl PrimitiveValueBase for Boolean {
        fn get_type(&self) -> String {
            String::from("boolean")
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for Boolean
    pub trait BooleanValueBase {
        fn new(val: bool) -> Boolean;
        fn get_state(&self) -> bool;
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

pub struct Token {
    ast_type: i32,
    value: String,
}

pub mod ast_operations {

    use crate::primitive_values::BooleanValueBase;

    /* BASE */
    use std::any::Any;

    pub trait AstBase {
        fn get_type(&self) -> crate::tokens_type::Val;
        fn as_self(&self) -> &dyn Any;
    }

    pub struct Ast {
        pub body: Vec<Box<dyn self::AstBase>>,
        pub token_type: crate::tokens_type::Val,
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
            crate::tokens_type::VAR_DEF
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* ASSIGNMENT */

    pub struct Assignment {
        pub interface: crate::PrimitiveValue,
        pub value: Box<dyn crate::primitive_values::PrimitiveValueBase>,
    }

    impl AstBase for Assignment {
        fn get_type(&self) -> i32 {
            crate::tokens_type::LEFT_ASSIGN
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    impl Assignment {
        #[allow(dead_code)]
        fn new(&self) -> Assignment {
            Assignment {
                interface: String::from(""),
                value: Box::new(crate::primitive_values::Boolean::new(false)),
            }
        }
    }

    /* EXPRESSION  */

    pub struct Expression {
        pub body: Vec<Box<dyn self::AstBase>>,
        pub token_type: crate::tokens_type::Val,
    }

    impl AstBase for Expression {
        fn get_type(&self) -> i32 {
            crate::tokens_type::EXPRESSION
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
                token_type: crate::tokens_type::EXPRESSION,
                body: Vec::new(),
            }
        }
    }
}

mod ham {

    use crate::ast_operations::ExpressionBase;
    use crate::ast_operations::VarDefinitionBase;
    use crate::primitive_values::BooleanValueBase;
    use crate::primitive_values::NumberValueBase;
    use regex::Regex;

    fn get_lines(code: String) -> crate::LinesList {
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

    fn transform_into_tokens(lines: crate::LinesList) -> crate::TokensList {
        let mut tokens = Vec::new();

        for line in lines {
            for word in line {
                let word = String::from(word);

                let token_type: crate::tokens_type::Val = match word.as_str() {
                    "let" => crate::tokens_type::VAR_DEF,
                    "=" => crate::tokens_type::LEFT_ASSIGN,
                    _ => crate::tokens_type::DEFAULT,
                };

                let ast_token = crate::Token {
                    ast_type: token_type,
                    value: word.clone(),
                };

                tokens.push(ast_token);
            }
        }

        return tokens;
    }

    pub fn get_tokens(code: String) -> crate::TokensList {
        let lines = self::get_lines(code);
        return self::transform_into_tokens(lines);
    }

    pub fn get_ast(tokens: crate::TokensList) -> crate::ast_operations::Expression {
        let mut ast_tree = crate::ast_operations::Expression::new();

        let mut token_n = 0;

        while token_n < tokens.len() {
            let current_token = &tokens[token_n];

            match current_token.ast_type {
                crate::tokens_type::VAR_DEF => {
                    let def_name = String::from(&tokens[token_n + 1].value.clone());

                    let def_value = String::from(&tokens[token_n + 3].value.clone());

                    match def_value.as_str() {
                        "True" => {
                            let assignment = crate::ast_operations::Assignment {
                                interface: String::from("boolean"),
                                value: Box::new(crate::primitive_values::Boolean::new(true)),
                            };
                            let ast_token =
                                crate::ast_operations::VarDefinition::new(def_name, assignment);
                            ast_tree.body.push(Box::new(ast_token));
                        }
                        "False" => {
                            let assignment = crate::ast_operations::Assignment {
                                interface: String::from("boolean"),
                                value: Box::new(crate::primitive_values::Boolean::new(false)),
                            };
                            let ast_token =
                                crate::ast_operations::VarDefinition::new(def_name, assignment);
                            ast_tree.body.push(Box::new(ast_token));
                        }
                        val if val.parse::<i32>().is_ok() => {
                            let assignment = crate::ast_operations::Assignment {
                                interface: String::from("number"),
                                value: Box::new(crate::primitive_values::Number::new(
                                    val.parse::<i32>().unwrap(),
                                )),
                            };
                            let ast_token =
                                crate::ast_operations::VarDefinition::new(def_name, assignment);
                            ast_tree.body.push(Box::new(ast_token));
                        }
                        _ => (),
                    };
                }
                _ => (),
            }

            token_n += 1;
        }

        return ast_tree;
    }
}

fn main() {
    use crate::primitive_values::BooleanValueBase;
    use crate::primitive_values::NumberValueBase;

    let code = "let test = 1 \n let ok = False ";

    println!("` \n {} \n`", code);

    let get_ast_tree = ham::get_tokens(String::from(code));
    let ast = ham::get_ast(get_ast_tree);

    for op in ast.body {
        match op.get_type() {
            crate::tokens_type::VAR_DEF => {
                let variable = op
                    .as_self()
                    .downcast_ref::<crate::ast_operations::VarDefinition>()
                    .unwrap();

                let assignment_type = variable.assignment.value.get_type();

                let value = match assignment_type.as_str() {
                    "boolean" => {
                        let state = variable
                            .assignment
                            .value
                            .as_self()
                            .downcast_ref::<crate::primitive_values::Boolean>()
                            .unwrap()
                            .get_state();
                        format!("{}", state)
                    }
                    "number" => {
                        let state = variable
                            .assignment
                            .value
                            .as_self()
                            .downcast_ref::<crate::primitive_values::Number>()
                            .unwrap()
                            .get_state();
                        format!("{}", state)
                    }
                    &_ => panic!("value not setted"),
                };

                println!(
                    "[ type: variable | name: {} | type: {} | value: {:?} ]",
                    variable.def_name, assignment_type, value
                )
            }
            _ => {
                println!("IDK")
            }
        }
    }
}
