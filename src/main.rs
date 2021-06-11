use regex::Regex;
use std::io;
use std::io::BufRead;

type LinesList = Vec<Vec<String>>;
type TokensList = Vec<Token>;
type PrimitiveValue = String;

pub mod tokens_type {
    pub type Val = i32;
    pub const REFERENCE: Val = 0;
    pub const VAR_DEF: Val = 1;
    pub const LEFT_ASSIGN: Val = 2;
    pub const EXPRESSION: Val = 3;
    pub const FN_CALL: Val = 4;
    pub const OPEN_PARENT: Val = 5;
    pub const CLOSE_PARENT: Val = 6;
}

pub mod primitive_values {
    use std::any::Any;

    pub type ValueType = i32;
    pub const BOOLEAN: ValueType = 7;
    pub const NUMBER: ValueType = 8;
    pub const STRING: ValueType = 9;

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
#[derive(Clone)]
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

    /* FUNCTION ARGUMENT */
    pub struct Argument {
        pub val_type: crate::tokens_type::Val,
        pub value: String,
    }

    impl Argument {
        pub fn new(value: String) -> Argument {
            let val_type = match value.clone() {
                val if val.chars().nth(0).unwrap() == '"'
                    && val.chars().nth(val.len() - 1).unwrap() == '"' =>
                {
                    crate::primitive_values::STRING
                }
                val if val.as_str().parse::<i32>().is_ok() => crate::primitive_values::NUMBER,
                _ => crate::tokens_type::REFERENCE,
            };

            Argument {
                val_type,
                value: value.clone(),
            }
        }
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

    /* FUNCTION CALL  */

    pub struct FnCall {
        pub token_type: crate::tokens_type::Val,
        pub fn_name: String,
        pub arguments: Vec<Argument>,
    }

    impl AstBase for FnCall {
        fn get_type(&self) -> i32 {
            crate::tokens_type::FN_CALL
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    pub trait FnCallBase {
        fn new(fn_name: String) -> Self;
    }

    impl FnCallBase for FnCall {
        fn new(fn_name: String) -> FnCall {
            FnCall {
                token_type: crate::tokens_type::FN_CALL,
                fn_name,
                arguments: Vec::new(),
            }
        }
    }
}

fn split_keep<'a>(r: &Regex, text: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in text.match_indices(r) {
        if last != index {
            result.push(&text[last..index]);
        }
        result.push(matched);
        last = index + matched.len();
    }
    if last < text.len() {
        result.push(&text[last..]);
    }
    result
}

mod ham {

    use crate::ast_operations::ExpressionBase;
    use crate::ast_operations::FnCallBase;
    use crate::ast_operations::VarDefinitionBase;
    use crate::primitive_values::BooleanValueBase;
    use crate::primitive_values::NumberValueBase;
    use crate::split_keep;
    use regex::Regex;
    use std::sync::{Arc, Mutex};

    fn get_lines(code: String) -> crate::LinesList {
        let mut lines = Vec::new();

        // Every line
        for line in code.split("\n") {
            let line = String::from(line);
            let mut line_ast = Vec::new();

            let re = Regex::new(r"[\s+,:]|([()])").unwrap();

            // Every detected word
            for word in split_keep(&re, &line) {
                // Prevent empty words
                if word.trim() != "" {
                    line_ast.push(String::from(word.trim()));
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
                    "(" => crate::tokens_type::OPEN_PARENT,
                    ")" => crate::tokens_type::CLOSE_PARENT,
                    _ => crate::tokens_type::REFERENCE,
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

        let get_tokens_from_to = |from: usize, to: crate::tokens_type::Val| -> crate::TokensList {
            let mut found_tokens = Vec::new();

            let mut tok_n = from;

            while tok_n < tokens.len() {
                if tokens[tok_n].ast_type == to {
                    break;
                } else {
                    found_tokens.push(tokens[tok_n].clone())
                }
                tok_n += 1;
            }

            found_tokens
        };

        let mut token_n = 0;

        while token_n < tokens.len() {
            let current_token = &tokens[token_n];

            match current_token.ast_type {
                crate::tokens_type::VAR_DEF => {
                    let def_name = String::from(&tokens[token_n + 1].value.clone());
                    let def_value = String::from(&tokens[token_n + 3].value.clone());

                    match def_value.as_str() {
                        // True boolean
                        "True" => {
                            let assignment = crate::ast_operations::Assignment {
                                interface: String::from("boolean"),
                                value: Box::new(crate::primitive_values::Boolean::new(true)),
                            };
                            let ast_token =
                                crate::ast_operations::VarDefinition::new(def_name, assignment);
                            ast_tree.body.push(Box::new(ast_token));
                        }
                        // False boolean
                        "False" => {
                            let assignment = crate::ast_operations::Assignment {
                                interface: String::from("boolean"),
                                value: Box::new(crate::primitive_values::Boolean::new(false)),
                            };
                            let ast_token =
                                crate::ast_operations::VarDefinition::new(def_name, assignment);
                            ast_tree.body.push(Box::new(ast_token));
                        }
                        // Numeric values
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

                    token_n += 3;
                }
                crate::tokens_type::REFERENCE => {
                    let next_token = tokens[token_n + 1].ast_type;

                    let reference_type = match next_token {
                        crate::tokens_type::OPEN_PARENT => crate::tokens_type::FN_CALL,
                        _ => 0,
                    };

                    match reference_type {
                        crate::tokens_type::FN_CALL => {
                            let mut ast_token =
                                crate::ast_operations::FnCall::new(current_token.value.clone());

                            // Ignore itself and the (
                            let starting_token = token_n + 2;

                            let arguments: Vec<crate::ast_operations::Argument> =
                                get_tokens_from_to(
                                    starting_token,
                                    crate::tokens_type::CLOSE_PARENT,
                                )
                                .iter()
                                .map(|token| {
                                    crate::ast_operations::Argument::new(token.value.clone())
                                })
                                .collect();

                            ast_token.arguments = arguments;
                            ast_tree.body.push(Box::new(ast_token));
                        }
                        _ => (),
                    }

                    token_n += 3;
                }
                // References
                _ => {}
            }

            token_n += 1;
        }

        return ast_tree;
    }

    #[derive(Clone, Debug)]
    struct VariableDef {
        name: String,
        val_type: crate::primitive_values::ValueType,
        value: i32,
    }

    #[derive(Clone, Debug)]
    struct FunctionDef {
        name: String,
        cb: fn(arg: Vec<String>),
    }

    #[derive(Clone)]
    pub struct Heap {
        functions: Vec<FunctionDef>,
        variables: Vec<VariableDef>,
    }

    pub mod heap_errors {
        pub type HeapErrorVal = i32;

        // Function wasn't found in the current scope
        pub const FUNCTION_NOT_FOUND: HeapErrorVal = 0;

        // Variable wasn't found in the current scope
        pub const VARIABLE_NOT_FOUND: HeapErrorVal = 1;
    }

    impl Heap {
        pub fn new() -> Heap {
            Heap {
                functions: Vec::new(),
                variables: Vec::new(),
            }
        }
        pub fn raise_error(kind: heap_errors::HeapErrorVal, args: Vec<String>) {
            let msg = match kind {
                heap_errors::FUNCTION_NOT_FOUND => format!("Function <{}> was not found", args[0]),
                heap_errors::VARIABLE_NOT_FOUND => format!("Variable <{}> was not found", args[0]),
                _ => String::from("Unhandled error"),
            };

            println!("  :: Error :: {}", msg);
        }
    }

    pub fn run_ast(ast: crate::ast_operations::Expression, heap: Heap) -> Heap {
        let heap = Arc::new(Mutex::new(heap));

        let get_var_reference = |var_name: String| -> Result<VariableDef, ()> {
            let heap = heap.lock().unwrap();

            for op_var in &heap.variables {
                if op_var.name == var_name {
                    return Ok(op_var.clone());
                }
            }
            Err(())
        };

        let get_fn = |fn_name: String| -> Result<FunctionDef, ()> {
            let heap = heap.lock().unwrap();
            for op_fn in &heap.functions {
                if op_fn.name == fn_name {
                    return Ok(op_fn.clone());
                }
            }

            Heap::raise_error(heap_errors::FUNCTION_NOT_FOUND, vec![fn_name.clone()]);
            Err(())
        };

        let resolve_val = |val_type: crate::primitive_values::ValueType,
                           value: String|
         -> Result<String, ()> {
            let res: String = match val_type {
                crate::primitive_values::STRING => value,
                crate::tokens_type::REFERENCE => {
                    let ref_name = value.clone();
                    let ref_value = get_var_reference(ref_name.clone());

                    let mut val = String::from("");
                    if ref_value.is_ok() {
                        val = ref_value.unwrap().value.to_string()
                    } else {
                        Heap::raise_error(heap_errors::VARIABLE_NOT_FOUND, vec![ref_name.clone()])
                    }
                    val
                }
                _ => String::from(""),
            };

            Ok(res)
        };

        /*
         * print() function
         */
        heap.lock().unwrap().functions.push(FunctionDef {
            name: String::from("print"),
            cb: |args| {
                print!("{}", args.join(""));
            },
        });

        /*
         * println() function
         */
        heap.lock().unwrap().functions.push(FunctionDef {
            name: String::from("println"),
            cb: |args| {
                println!("{}", args.join(""));
            },
        });

        for op in &ast.body {
            match op.get_type() {
                crate::tokens_type::VAR_DEF => {
                    let variable = op
                        .as_self()
                        .downcast_ref::<crate::ast_operations::VarDefinition>()
                        .unwrap();

                    let assignment_type = variable.assignment.value.get_type();

                    match assignment_type.as_str() {
                        "boolean" => {
                            // WIP

                            let state = variable
                                .assignment
                                .value
                                .as_self()
                                .downcast_ref::<crate::primitive_values::Boolean>()
                                .unwrap()
                                .get_state();

                            let value = if state { 1 } else { 0 };

                            heap.lock().unwrap().variables.push(VariableDef {
                                name: variable.def_name.clone(),
                                val_type: crate::primitive_values::BOOLEAN,
                                value,
                            });
                        }
                        "number" => {
                            let state = variable
                                .assignment
                                .value
                                .as_self()
                                .downcast_ref::<crate::primitive_values::Number>()
                                .unwrap()
                                .get_state();

                            heap.lock().unwrap().variables.push(VariableDef {
                                name: variable.def_name.clone(),
                                val_type: crate::primitive_values::NUMBER,
                                value: state,
                            });
                        }
                        &_ => panic!("value not setted"),
                    };
                }
                crate::tokens_type::FN_CALL => {
                    let fn_call = op
                        .as_self()
                        .downcast_ref::<crate::ast_operations::FnCall>()
                        .unwrap();

                    let ref_fn = get_fn(fn_call.fn_name.clone());

                    // If the calling function is found
                    if ref_fn.is_ok() {
                        let mut arguments = Vec::new();

                        for argument in &fn_call.arguments {
                            let ref_value = resolve_val(argument.val_type, argument.value.clone());

                            if ref_value.is_ok() {
                                arguments.push(ref_value.unwrap().to_string());
                            }
                        }

                        (ref_fn.unwrap().cb)(arguments);
                    }
                }
                _ => {
                    println!("IDK")
                }
            }
        }

        let heap = heap.lock().unwrap();

        return heap.clone();
    }
}

static CLI_MSG: &str = ":: Welcome to HAM REPL :: \n";

fn main() {
    println!("{}", CLI_MSG);

    // Memory heap
    let mut heap = ham::Heap::new();

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
        heap = ham::run_ast(ast, heap.clone());

        println!("  <- ");

        println!(">");
    }
}
