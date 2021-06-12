use std::io;
use std::io::BufRead;
use std::sync::Mutex;

type LinesList = Vec<Vec<String>>;
type TokensList = Vec<Token>;

pub mod op_codes {
    pub type Val = i32;
    pub const REFERENCE: Val = 0;
    pub const VAR_DEF: Val = 1;
    pub const LEFT_ASSIGN: Val = 2;
    pub const EXPRESSION: Val = 3;
    pub const FN_CALL: Val = 4;
    pub const OPEN_PARENT: Val = 5;
    pub const CLOSE_PARENT: Val = 6;
    pub const BOOLEAN: Val = 7;
    pub const NUMBER: Val = 8;
    pub const STRING: Val = 9;
    pub const VAR_ASSIGN: Val = 10;

    const CODES_RANGE: Val = 10;

    pub fn is_valid(op_code: Val) -> bool {
        if op_code < 0 || op_code > CODES_RANGE {
            false
        } else {
            true
        }
    }
}

pub mod primitive_values {
    use std::any::Any;

    pub trait PrimitiveValueBase: dyn_clone::DynClone {
        fn as_self(&self) -> &dyn Any;
    }

    dyn_clone::clone_trait_object!(PrimitiveValueBase);

    // REFERENCE

    #[derive(Clone)]
    pub struct Reference(pub String);

    // Implement base methods for String
    impl PrimitiveValueBase for Reference {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for String
    pub trait ReferenceValueBase {
        fn new(val: String) -> Reference;
        fn get_state(&self) -> String;
    }

    impl ReferenceValueBase for Reference {
        fn new(val: String) -> Reference {
            Reference(val)
        }

        fn get_state(&self) -> String {
            self.0.clone()
        }
    }

    // STRING

    #[derive(Clone)]
    pub struct StringVal(pub String);

    // Implement base methods for String
    impl PrimitiveValueBase for StringVal {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for String
    pub trait StringValueBase {
        fn new(val: String) -> StringVal;
        fn get_state(&self) -> String;
    }

    impl StringValueBase for StringVal {
        fn new(val: String) -> StringVal {
            StringVal(val)
        }

        fn get_state(&self) -> String {
            self.0.clone()
        }
    }

    // NUMBER

    #[derive(Clone)]
    pub struct Number(pub i32);

    // Implement base methods for Number
    impl PrimitiveValueBase for Number {
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

    #[derive(Clone)]
    pub struct Boolean(pub bool);

    // Implement base methods for Boolean
    impl PrimitiveValueBase for Boolean {
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

    /* BASE */
    use std::any::Any;

    pub trait AstBase {
        fn get_type(&self) -> crate::op_codes::Val;
        fn as_self(&self) -> &dyn Any;
    }

    pub struct Ast {
        pub body: Vec<Box<dyn self::AstBase>>,
        pub token_type: crate::op_codes::Val,
    }

    /* FUNCTION ARGUMENT */
    pub struct Argument {
        pub val_type: crate::op_codes::Val,
        pub value: String,
    }

    impl Argument {
        pub fn new(value: String) -> Argument {
            let val_type = match value.clone() {
                // Is String
                val if val.chars().nth(0).unwrap() == '"'
                    && val.chars().nth(val.len() - 1).unwrap() == '"' =>
                {
                    crate::op_codes::STRING
                }
                // Is Number
                val if val.as_str().parse::<i32>().is_ok() => crate::op_codes::NUMBER,
                _ => crate::op_codes::REFERENCE,
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
            crate::op_codes::VAR_DEF
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* VARIABLE ASSIGNMENT */
    pub trait VarAssignmentBase {
        fn get_def_name(&self) -> String;
        fn new(var_name: String, assignment: Assignment) -> Self;
    }

    pub struct VarAssignment {
        pub var_name: String,
        pub assignment: Assignment,
    }

    impl VarAssignmentBase for VarAssignment {
        fn get_def_name(&self) -> String {
            return self.var_name.clone();
        }
        fn new(var_name: String, assignment: Assignment) -> VarAssignment {
            VarAssignment {
                var_name,
                assignment,
            }
        }
    }

    impl AstBase for VarAssignment {
        fn get_type(&self) -> i32 {
            crate::op_codes::VAR_ASSIGN
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* ASSIGNMENT */

    pub struct Assignment {
        pub interface: crate::op_codes::Val,
        pub value: Box<dyn crate::primitive_values::PrimitiveValueBase>,
    }

    impl AstBase for Assignment {
        fn get_type(&self) -> i32 {
            crate::op_codes::LEFT_ASSIGN
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* EXPRESSION  */

    pub struct Expression {
        pub body: Vec<Box<dyn self::AstBase>>,
        pub token_type: crate::op_codes::Val,
    }

    impl AstBase for Expression {
        fn get_type(&self) -> i32 {
            crate::op_codes::EXPRESSION
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
                token_type: crate::op_codes::EXPRESSION,
                body: Vec::new(),
            }
        }
    }

    /* FUNCTION CALL  */

    pub struct FnCall {
        pub token_type: crate::op_codes::Val,
        pub fn_name: String,
        pub arguments: Vec<Argument>,
    }

    impl AstBase for FnCall {
        fn get_type(&self) -> i32 {
            crate::op_codes::FN_CALL
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
                token_type: crate::op_codes::FN_CALL,
                fn_name,
                arguments: Vec::new(),
            }
        }
    }
}

mod ham {

    use crate::ast_operations::FnCallBase;
    use crate::ast_operations::VarDefinitionBase;
    use crate::ast_operations::{Assignment, ExpressionBase, VarAssignmentBase};
    use crate::primitive_values::NumberValueBase;
    use crate::primitive_values::{BooleanValueBase, ReferenceValueBase, StringValueBase};
    use regex::Regex;
    use std::any::Any;
    use std::sync::{Arc, Mutex};

    fn split<'a>(r: &'a Regex, text: &'a str) -> Vec<&'a str> {
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

    fn get_lines(code: String) -> crate::LinesList {
        let mut lines = Vec::new();

        // Every line
        for line in code.split("\n") {
            let line = String::from(line);
            let mut line_ast = Vec::new();

            let re = Regex::new(r"[\s+,:]|([()])").unwrap();

            // Every detected word
            for word in split(&re, &line) {
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

                let token_type: crate::op_codes::Val = match word.as_str() {
                    "let" => crate::op_codes::VAR_DEF,
                    "=" => crate::op_codes::LEFT_ASSIGN,
                    "(" => crate::op_codes::OPEN_PARENT,
                    ")" => crate::op_codes::CLOSE_PARENT,
                    _ => crate::op_codes::REFERENCE,
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

        let get_tokens_from_to = |from: usize, to: crate::op_codes::Val| -> crate::TokensList {
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

        fn get_assignment_token(val: String) -> Assignment {
            match val.as_str() {
                // True boolean
                "true" => crate::ast_operations::Assignment {
                    interface: crate::op_codes::BOOLEAN,
                    value: Box::new(crate::primitive_values::Boolean::new(true)),
                },
                // False boolean
                "false" => crate::ast_operations::Assignment {
                    interface: crate::op_codes::BOOLEAN,
                    value: Box::new(crate::primitive_values::Boolean::new(false)),
                },
                // Numeric values
                val if val.parse::<i32>().is_ok() => crate::ast_operations::Assignment {
                    interface: crate::op_codes::NUMBER,
                    value: Box::new(crate::primitive_values::Number::new(
                        val.parse::<i32>().unwrap(),
                    )),
                },
                // String values
                val if val.chars().nth(0).unwrap() == '"'
                    && val.chars().nth(val.len() - 1).unwrap() == '"' =>
                {
                    crate::ast_operations::Assignment {
                        interface: crate::op_codes::STRING,
                        value: Box::new(crate::primitive_values::StringVal::new(String::from(val))),
                    }
                }
                val => crate::ast_operations::Assignment {
                    interface: crate::op_codes::REFERENCE,
                    value: Box::new(crate::primitive_values::Reference::new(String::from(val))),
                },
            }
        }

        while token_n < tokens.len() {
            let current_token = &tokens[token_n];

            match current_token.ast_type {
                crate::op_codes::VAR_DEF => {
                    let def_name = String::from(&tokens[token_n + 1].value.clone());
                    let def_value = String::from(&tokens[token_n + 3].value.clone());

                    let assignment = get_assignment_token(def_value);

                    let ast_token = crate::ast_operations::VarDefinition::new(def_name, assignment);
                    ast_tree.body.push(Box::new(ast_token));

                    token_n += 3;
                }
                crate::op_codes::REFERENCE => {
                    let next_token = tokens[token_n + 1].ast_type;

                    let reference_type = match next_token {
                        crate::op_codes::OPEN_PARENT => crate::op_codes::FN_CALL,
                        crate::op_codes::LEFT_ASSIGN => crate::op_codes::VAR_ASSIGN,
                        _ => 0,
                    };

                    match reference_type {
                        crate::op_codes::VAR_ASSIGN => {
                            let token_after_equal = tokens[token_n + 2].clone();

                            let assignment = get_assignment_token(token_after_equal.value.clone());

                            let ast_token = crate::ast_operations::VarAssignment::new(
                                current_token.value.clone(),
                                assignment,
                            );

                            ast_tree.body.push(Box::new(ast_token));
                        }
                        crate::op_codes::FN_CALL => {
                            let mut ast_token =
                                crate::ast_operations::FnCall::new(current_token.value.clone());

                            // Ignore itself and the (
                            let starting_token = token_n + 2;

                            let arguments: Vec<crate::ast_operations::Argument> =
                                get_tokens_from_to(starting_token, crate::op_codes::CLOSE_PARENT)
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

    #[derive(Clone)]
    struct VariableDef {
        name: String,
        val_type: crate::op_codes::Val,
        value: Box<dyn crate::primitive_values::PrimitiveValueBase>,
    }

    #[derive(Clone)]
    struct FunctionDef {
        name: String,
        cb: fn(arg: Vec<String>),
    }

    #[derive(Clone)]
    pub struct Heap {
        functions: Vec<FunctionDef>,
        variables: Vec<VariableDef>,
    }

    impl Heap {
        pub fn new() -> Heap {
            Heap {
                functions: Vec::new(),
                variables: Vec::new(),
            }
        }
    }

    mod errors {

        pub type ErrorVal = i32;

        // Function wasn't found in the current scope
        pub const FUNCTION_NOT_FOUND: ErrorVal = 0;

        // Variable wasn't found in the current scope
        pub const VARIABLE_NOT_FOUND: ErrorVal = 1;

        // Unhandled value
        pub const UNHANDLED_VALUE: ErrorVal = 2;

        // Unhandled value type
        pub const UNHANDLED_VALUE_TYPE_CODE: ErrorVal = 3;

        pub fn raise_error(kind: ErrorVal, args: Vec<String>) {
            let msg = match kind {
                FUNCTION_NOT_FOUND => format!("Function <{}> was not found", args[0]),
                VARIABLE_NOT_FOUND => format!("Variable <{}> was not found", args[0]),
                UNHANDLED_VALUE => format!("Value <{}> is not handled", args[0]),
                UNHANDLED_VALUE_TYPE_CODE => {
                    format!("Value type by code {} is not handled", args[0])
                }
                _ => String::from("Unhandled error"),
            };

            println!("  :: Error :: {}", msg);
        }
    }

    pub fn run_ast(ast: crate::ast_operations::Expression, heap: &Mutex<Heap>) {
        let heap = Arc::new(heap);

        // Search variables in the heap by its name
        let get_var_reference = |var_name: String| -> Result<VariableDef, ()> {
            let heap = heap.lock().unwrap();

            for op_var in &heap.variables {
                if op_var.name == var_name {
                    return Ok(op_var.clone());
                }
            }

            errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
            Err(())
        };

        // Search functions in the heap by its name
        let get_fn = |fn_name: String| -> Result<FunctionDef, ()> {
            let heap = heap.lock().unwrap();
            for op_fn in &heap.functions {
                if op_fn.name == fn_name {
                    return Ok(op_fn.clone());
                }
            }

            errors::raise_error(errors::FUNCTION_NOT_FOUND, vec![fn_name.clone()]);
            Err(())
        };

        // Resolve values
        let resolve_val = |val_type: crate::op_codes::Val, value: String| -> Result<String, ()> {
            let res: String = match val_type {
                // If the value is type String, Number or boolean then return it self
                crate::op_codes::STRING => value,
                crate::op_codes::BOOLEAN => value,
                crate::op_codes::NUMBER => value,

                // If the value is a reference to a variable then returns the variable's current value
                crate::op_codes::REFERENCE => {
                    let ref_name = value.clone();
                    let ref_value = get_var_reference(ref_name.clone());

                    let mut val = String::from("");
                    if ref_value.is_ok() {
                        let ref_value = ref_value.unwrap();
                        val = match ref_value.val_type {
                            crate::op_codes::BOOLEAN => {
                                downcast_val::<crate::primitive_values::Boolean>(
                                    ref_value.value.as_self(),
                                )
                                .0
                                .to_string()
                            }
                            crate::op_codes::STRING => {
                                downcast_val::<crate::primitive_values::StringVal>(
                                    ref_value.value.as_self(),
                                )
                                .0
                                .clone()
                            }
                            crate::op_codes::NUMBER => {
                                downcast_val::<crate::primitive_values::Number>(
                                    ref_value.value.as_self(),
                                )
                                .0
                                .to_string()
                            }
                            _ => String::from(""),
                        }
                    }
                    val
                }
                _ => String::from(""),
            };

            Ok(res)
        };

        fn downcast_val<T: 'static>(val: &dyn Any) -> &T {
            val.downcast_ref::<T>().unwrap()
        }

        let resolve_def = |val_type: crate::op_codes::Val,
                           ref_val: Box<dyn crate::primitive_values::PrimitiveValueBase>|
         -> (
            crate::op_codes::Val,
            Box<dyn crate::primitive_values::PrimitiveValueBase>,
        ) {
            match val_type {
                crate::op_codes::STRING => (val_type, ref_val),
                crate::op_codes::BOOLEAN => (val_type, ref_val),
                crate::op_codes::NUMBER => (val_type, ref_val),
                crate::op_codes::REFERENCE => {
                    let val = downcast_val::<crate::primitive_values::Reference>(ref_val.as_self())
                        .0
                        .clone();

                    let ref_def = get_var_reference(val);

                    let ref_assign = ref_def.unwrap();

                    (ref_assign.val_type, ref_assign.value)
                }
                _ => (0, Box::new(crate::primitive_values::Number(0))),
            }
        };

        // Modify var
        let modify_var =
            |var_name: String, value: Box<dyn crate::primitive_values::PrimitiveValueBase>| {
                let mut heap = heap.lock().unwrap();

                for mut op_var in heap.variables.iter_mut() {
                    if op_var.name == var_name {
                        op_var.value = value.clone();
                    }
                }
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
                /*
                 * Handle variables definitions
                 */
                crate::op_codes::VAR_DEF => {
                    let variable =
                        downcast_val::<crate::ast_operations::VarDefinition>(op.as_self());

                    let val_type = variable.assignment.interface;
                    let ref_val = variable.assignment.value.clone();

                    let (ref_type, ref_val) = resolve_def(val_type, ref_val);

                    heap.lock().unwrap().variables.push(VariableDef {
                        name: variable.def_name.clone(),
                        val_type: ref_type,
                        value: ref_val,
                    });

                    //errors::raise_error(errors::UNHANDLED_VALUE_TYPE_CODE, vec![val_type.to_string()])
                }

                /*
                 * Handle variable assignments
                 */
                crate::op_codes::VAR_ASSIGN => {
                    let variable =
                        downcast_val::<crate::ast_operations::VarAssignment>(op.as_self());

                    modify_var(variable.var_name.clone(), variable.assignment.value.clone());
                }

                /*
                 * Handle function calls
                 */
                crate::op_codes::FN_CALL => {
                    let fn_call = downcast_val::<crate::ast_operations::FnCall>(op.as_self());

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
    }
}

static CLI_MSG: &str = ":: Welcome to HAM REPL :: \n";

fn main() {
    println!("{}", CLI_MSG);

    // Memory heap
    let heap = Mutex::new(ham::Heap::new());

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
        ham::run_ast(ast, &heap);

        println!("  <- ");

        println!(">");
    }
}
