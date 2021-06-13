use crate::ast::ast_operations;
use crate::ast::ast_operations::{
    AstBase, ExpressionBase, FnCallBase, FnDefinitionBase, VarAssignmentBase, VarDefinitionBase,
};
use crate::tokenizer::{LinesList, Token, TokensList};
use crate::utils::primitive_values::{
    BooleanValueBase, NumberValueBase, ReferenceValueBase, StringValueBase,
};
use crate::utils::{errors, op_codes, primitive_values};

use regex::Regex;
use std::any::Any;
use std::sync::Mutex;

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

fn get_lines(code: String) -> LinesList {
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

fn transform_into_tokens(lines: LinesList) -> TokensList {
    let mut tokens = Vec::new();

    for line in lines {
        for word in line {
            let word = String::from(word);

            let token_type: op_codes::Val = match word.as_str() {
                "let" => op_codes::VAR_DEF,
                "=" => op_codes::LEFT_ASSIGN,
                "(" => op_codes::OPEN_PARENT,
                ")" => op_codes::CLOSE_PARENT,
                "fn" => op_codes::FN_DEF,
                "{" => op_codes::OPEN_BLOCK,
                "}" => op_codes::CLOSE_BLOCK,
                _ => op_codes::REFERENCE,
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

pub fn move_tokens_into_ast(tokens: TokensList, ast_tree: &Mutex<ast_operations::Expression>) {
    let mut ast_tree = ast_tree.lock().unwrap();

    // Get tokens with index starting on `from` until a token matches its type to `to`
    let get_tokens_from_to = |from: usize, to: op_codes::Val| -> TokensList {
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

    // Get all the tokens inside a expression block
    let get_expr_tokens = |from: usize| -> TokensList {
        let mut found_tokens = Vec::new();

        let mut block_counter = 0;

        let mut tok_n = from;

        while tok_n < tokens.len() {
            let token = tokens[tok_n].clone();

            if token.ast_type == op_codes::OPEN_BLOCK {
                block_counter += 1;
            } else if token.ast_type == op_codes::CLOSE_BLOCK {
                block_counter -= 1;
            }

            if block_counter == 0 {
                break;
            } else if tok_n > from {
                found_tokens.push(token.clone());
            }
            tok_n += 1;
        }

        found_tokens
    };

    let mut token_n = 0;

    fn get_assignment_token(val: String) -> ast_operations::Assignment {
        match val.as_str() {
            // True boolean
            "true" => ast_operations::Assignment {
                interface: op_codes::BOOLEAN,
                value: Box::new(primitive_values::Boolean::new(true)),
            },
            // False boolean
            "false" => ast_operations::Assignment {
                interface: op_codes::BOOLEAN,
                value: Box::new(primitive_values::Boolean::new(false)),
            },
            // Numeric values
            val if val.parse::<i32>().is_ok() => ast_operations::Assignment {
                interface: op_codes::NUMBER,
                value: Box::new(primitive_values::Number::new(val.parse::<i32>().unwrap())),
            },
            // String values
            val if val.chars().nth(0).unwrap() == '"'
                && val.chars().nth(val.len() - 1).unwrap() == '"' =>
            {
                ast_operations::Assignment {
                    interface: op_codes::STRING,
                    value: Box::new(primitive_values::StringVal::new(String::from(val))),
                }
            }
            // References to other values (ej: referencing to a variable)
            val => ast_operations::Assignment {
                interface: op_codes::REFERENCE,
                value: Box::new(primitive_values::Reference::new(String::from(val))),
            },
        }
    }

    while token_n < tokens.len() {
        let current_token = &tokens[token_n];
        match current_token.ast_type {
            // Function definition
            op_codes::FN_DEF => {
                let def_name = String::from(&tokens[token_n + 1].value.clone());

                // Scope tree
                let scope_tree = Mutex::new(ast_operations::Expression::new());

                // Ignore function name and the (
                let starting_token = token_n + 2;

                // Get function arguments, WIP
                let arguments: Vec<ast_operations::Argument> =
                    get_tokens_from_to(starting_token, op_codes::CLOSE_PARENT)
                        .iter()
                        .map(|token| ast_operations::Argument::new(token.value.clone()))
                        .collect();

                // Ignore function name, (, arguments and )
                let open_block_index = starting_token + arguments.len() + 1;

                // Get all tokens inside the function block
                let block_tokens = get_expr_tokens(open_block_index);

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree);

                // Ignore the function body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a function definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token = ast_operations::FnDefinition::new(def_name, body.to_vec());
                ast_tree.body.push(Box::new(ast_token));
            }
            // Variable definition
            op_codes::VAR_DEF => {
                let def_name = String::from(&tokens[token_n + 1].value.clone());
                let def_value = String::from(&tokens[token_n + 3].value.clone());

                let assignment = get_assignment_token(def_value);

                let ast_token = ast_operations::VarDefinition::new(def_name, assignment);
                ast_tree.body.push(Box::new(ast_token));

                token_n += 4;
            }
            // References (fn calls, variable reassignation...)
            op_codes::REFERENCE => {
                let next_token = tokens[token_n + 1].ast_type;

                let reference_type = match next_token {
                    op_codes::OPEN_PARENT => op_codes::FN_CALL,
                    op_codes::LEFT_ASSIGN => op_codes::VAR_ASSIGN,
                    _ => 0,
                };

                match reference_type {
                    op_codes::VAR_ASSIGN => {
                        let token_after_equal = tokens[token_n + 2].clone();

                        let assignment = get_assignment_token(token_after_equal.value.clone());

                        let ast_token = ast_operations::VarAssignment::new(
                            current_token.value.clone(),
                            assignment,
                        );

                        ast_tree.body.push(Box::new(ast_token));

                        token_n += 3;
                    }
                    op_codes::FN_CALL => {
                        let mut ast_token =
                            ast_operations::FnCall::new(current_token.value.clone());

                        // Ignore itself and the (
                        let starting_token = token_n + 2;

                        let arguments: Vec<ast_operations::Argument> =
                            get_tokens_from_to(starting_token, op_codes::CLOSE_PARENT)
                                .iter()
                                .map(|token| ast_operations::Argument::new(token.value.clone()))
                                .collect();

                        token_n += 3 + arguments.len();

                        ast_token.arguments = arguments;
                        ast_tree.body.push(Box::new(ast_token));
                    }
                    _ => {
                        token_n += 1;
                        ()
                    }
                }
            }
            _ => {
                token_n += 1;
            }
        }
    }
}

#[derive(Clone)]
struct VariableDef {
    name: String,
    val_type: op_codes::Val,
    value: Box<dyn primitive_values::PrimitiveValueBase>,
    expr_id: String,
}

#[derive(Clone)]
struct FunctionDef {
    name: String,
    body: Vec<Box<dyn AstBase>>,
    cb: fn(arg: Vec<String>, body: Vec<Box<dyn AstBase>>, stack: &Mutex<Stack>),
    expr_id: String,
}

#[derive(Clone)]
pub struct Stack {
    functions: Vec<FunctionDef>,
    variables: Vec<VariableDef>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            functions: Vec::new(),
            variables: Vec::new(),
        }
    }
    pub fn drop_ops_from_id(&mut self, id: String) {
        for (index, op_var) in self.variables.clone().iter().enumerate() {
            if op_var.expr_id == id {
                self.variables.remove(index);
            }
        }
    }
}

pub fn run_ast(ast: &Mutex<ast_operations::Expression>, stack: &Mutex<Stack>) {
    let ast = ast.lock().unwrap();

    // Search variables in the stack by its name
    let get_var_reference = |var_name: String| -> Result<VariableDef, ()> {
        let stack = stack.lock().unwrap();

        for op_var in &stack.variables {
            if op_var.name == var_name {
                return Ok(op_var.clone());
            }
        }

        errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
        Err(())
    };

    // Search functions in the stack by its name
    let get_fn = |fn_name: String| -> Result<FunctionDef, ()> {
        let stack = stack.lock().unwrap();
        for op_fn in &stack.functions {
            if op_fn.name == fn_name {
                return Ok(op_fn.clone());
            }
        }

        errors::raise_error(errors::FUNCTION_NOT_FOUND, vec![fn_name.clone()]);
        Err(())
    };

    // Resolve values
    let resolve_val = |val_type: op_codes::Val, value: String| -> Result<String, ()> {
        let res: String = match val_type {
            // If the value is type String, Number or boolean then return it self
            op_codes::STRING => value,
            op_codes::BOOLEAN => value,
            op_codes::NUMBER => value,

            // If the value is a reference to a variable then returns the variable's current value
            op_codes::REFERENCE => {
                let ref_name = value.clone();
                let ref_value = get_var_reference(ref_name.clone());

                let mut val = String::from("");
                if ref_value.is_ok() {
                    let ref_value = ref_value.unwrap();
                    val = match ref_value.val_type {
                        op_codes::BOOLEAN => {
                            downcast_val::<primitive_values::Boolean>(ref_value.value.as_self())
                                .0
                                .to_string()
                        }
                        op_codes::STRING => {
                            downcast_val::<primitive_values::StringVal>(ref_value.value.as_self())
                                .0
                                .clone()
                        }
                        op_codes::NUMBER => {
                            downcast_val::<primitive_values::Number>(ref_value.value.as_self())
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

    /*
     * For static values it will just return the input but for references it will resolve its value
     * and then return it
     */
    let resolve_def = |val_type: op_codes::Val,
                       ref_val: Box<dyn primitive_values::PrimitiveValueBase>|
     -> (op_codes::Val, Box<dyn primitive_values::PrimitiveValueBase>) {
        match val_type {
            op_codes::STRING => (val_type, ref_val),
            op_codes::BOOLEAN => (val_type, ref_val),
            op_codes::NUMBER => (val_type, ref_val),
            op_codes::REFERENCE => {
                let val = downcast_val::<primitive_values::Reference>(ref_val.as_self())
                    .0
                    .clone();

                let ref_def = get_var_reference(val);

                if ref_def.is_ok() {
                    let ref_assign = ref_def.unwrap();
                    (ref_assign.val_type, ref_assign.value)
                } else {
                    (val_type, ref_val)
                }
            }
            _ => (0, Box::new(primitive_values::Number(0))),
        }
    };

    // Modify a variable
    let modify_var = |var_name: String, value: Box<dyn primitive_values::PrimitiveValueBase>| {
        let mut stack = stack.lock().unwrap();

        for mut op_var in stack.variables.iter_mut() {
            if op_var.name == var_name {
                op_var.value = value.clone();
                return ();
            }
        }

        errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
    };

    /*
     * print() function
     */
    stack.lock().unwrap().functions.push(FunctionDef {
        name: String::from("print"),
        body: vec![],
        cb: |args, _, _| {
            print!("{}", args.join(""));
        },
        expr_id: ast.expr_id.clone(),
    });

    /*
     * println() function
     */
    stack.lock().unwrap().functions.push(FunctionDef {
        name: String::from("println"),
        body: vec![],
        cb: |args, _, _| {
            println!("{}", args.join(""));
        },
        expr_id: ast.expr_id.clone(),
    });

    for op in &ast.body {
        match op.get_type() {
            op_codes::FN_DEF => {
                let function = downcast_val::<ast_operations::FnDefinition>(op.as_self());

                stack.lock().unwrap().functions.push(FunctionDef {
                    name: String::from(function.def_name.clone()),
                    body: function.body.clone(),
                    cb: |_, body: Vec<Box<dyn self::AstBase>>, stack| {
                        let expr = ast_operations::Expression::from_body(body.clone());
                        let expr_id = expr.expr_id.clone();

                        run_ast(&Mutex::new(expr), stack);

                        stack.lock().unwrap().drop_ops_from_id(expr_id.clone());
                    },
                    expr_id: ast.expr_id.clone(),
                });
            }

            /*
             * Handle variables definitions
             */
            op_codes::VAR_DEF => {
                let variable = downcast_val::<ast_operations::VarDefinition>(op.as_self());

                let val_type = variable.assignment.interface;
                let ref_val = variable.assignment.value.clone();

                let (ref_type, ref_val) = resolve_def(val_type, ref_val);

                if !op_codes::is_valid(ref_type) {
                    /*
                     * If value code is not valid then raise an error
                     */
                    errors::raise_error(
                        errors::UNHANDLED_VALUE_TYPE_CODE,
                        vec![val_type.to_string()],
                    )
                } else {
                    stack.lock().unwrap().variables.push(VariableDef {
                        name: variable.def_name.clone(),
                        val_type: ref_type,
                        value: ref_val,
                        expr_id: ast.expr_id.clone(),
                    });
                }
            }

            /*
             * Handle variable assignments
             */
            op_codes::VAR_ASSIGN => {
                let variable = downcast_val::<ast_operations::VarAssignment>(op.as_self());

                modify_var(variable.var_name.clone(), variable.assignment.value.clone());
            }

            /*
             * Handle function calls
             */
            op_codes::FN_CALL => {
                let fn_call = downcast_val::<ast_operations::FnCall>(op.as_self());
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

                    let func = ref_fn.unwrap();

                    (func.cb)(arguments, func.body, &stack);
                }
            }
            _ => {
                println!("IDK")
            }
        }
    }
}
