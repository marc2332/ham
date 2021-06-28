use crate::ast::ast_operations;
use crate::ast::ast_operations::{
    convert_tokens_into_arguments, convert_tokens_into_res_expressions, get_assignment_token_fn,
    get_tokens_from_to_fn, BoxedValue, ExpressionBase, FnCallBase, FnDefinitionBase,
    IfConditionalBase, VarAssignmentBase, VarDefinitionBase, WhileBase,
};
use crate::runtime::{
    downcast_val, get_methods_in_type, resolve_reference, value_to_string, values_to_strings,
};
use crate::stack::{FunctionDef, FunctionsContainer, Stack, VariableDef};
use crate::types::{IndexedTokenList, LinesList, Token, TokensList};
use crate::utils::op_codes::Directions;
use crate::utils::primitive_values::{Boolean, StringVal};
use crate::utils::{errors, op_codes, primitive_values};
use regex::Regex;
use std::sync::Mutex;
use uuid::Uuid;

pub mod ast;
pub mod runtime;
pub mod stack;
mod types;
mod utils;

/*
 * Split the text by the passed regex but also keep these words which are removed when splitting
 */
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

/*
 * Transform the code into lines
 */
fn get_lines(code: String) -> LinesList {
    let mut lines = Vec::new();

    // Every line
    for line in code.split('\n') {
        // Ignore // comments
        if line.starts_with("//") {
            continue;
        }

        let mut line_ast = Vec::new();

        let re = Regex::new(r#"([\s+,.:])|("(.*?)")|([()])"#).unwrap();

        // Every detected word
        for word in split(&re, line) {
            // Prevent empty words
            if word.trim() != "" {
                line_ast.push(String::from(word.trim()));
            }
        }
        lines.push(line_ast);
    }

    lines
}

/*
 * Trasnform a list of lines into a tokens list
 */
fn transform_into_tokens(lines: LinesList) -> TokensList {
    let mut tokens = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        for word in line {
            let token_type: op_codes::Val = match word.as_str() {
                "let" => op_codes::VAR_DEF,
                "=" => op_codes::LEFT_ASSIGN,
                "(" => op_codes::OPEN_PARENT,
                ")" => op_codes::CLOSE_PARENT,
                "fn" => op_codes::FN_DEF,
                "{" => op_codes::OPEN_BLOCK,
                "}" => op_codes::CLOSE_BLOCK,
                "if" => op_codes::IF_CONDITIONAL,
                "==" => op_codes::EQUAL_CONDITION,
                "return" => op_codes::RETURN,
                "." => op_codes::PROP_ACCESS,
                "," => op_codes::COMMA_DELIMITER,
                "while" => op_codes::WHILE_DEF,
                "!=" => op_codes::NOT_EQUAL_CONDITION,
                _ => op_codes::REFERENCE,
            };

            let ast_token = Token {
                ast_type: token_type,
                value: word.clone(),
                line: i + 1,
            };

            tokens.push(ast_token);
        }
    }

    tokens
}

/*
 * Transform the code into a list of tokens
 */
pub fn get_tokens(code: String) -> TokensList {
    let lines = self::get_lines(code);
    self::transform_into_tokens(lines)
}

pub fn move_tokens_into_ast(tokens: TokensList, ast_tree: &Mutex<ast_operations::Expression>) {
    let mut ast_tree = ast_tree.lock().unwrap();

    // Closure version of above
    let get_tokens_from_to = |from: usize, to: op_codes::Val| -> IndexedTokenList {
        get_tokens_from_to_fn(from, to, tokens.clone(), Directions::LeftToRight)
    };

    // Get all the tokens in a group (expression blocks, arguments)
    let get_tokens_in_group_of =
        |from: usize, open_tok: op_codes::Val, close_tok: op_codes::Val| -> TokensList {
            let mut found_tokens = Vec::new();

            let mut count = 0;

            let mut token_n = from;

            while token_n < tokens.len() {
                let token = tokens[token_n].clone();

                if token.ast_type == open_tok {
                    count += 1;
                } else if token.ast_type == close_tok {
                    count -= 1;
                }

                if count == 0 {
                    break;
                } else if token_n > from {
                    found_tokens.push(token.clone());
                }
                token_n += 1;
            }

            found_tokens
        };

    let get_assignment_token =
        |val: String, token_n: usize| -> (usize, ast_operations::BoxedValue) {
            get_assignment_token_fn(val, token_n, tokens.clone(), Directions::LeftToRight)
        };

    let mut token_n = 0;

    while token_n < tokens.len() {
        let current_token = &tokens[token_n];
        match current_token.ast_type {
            // While block
            op_codes::WHILE_DEF => {
                // Get the if condition tokens
                let condition_tokens = get_tokens_from_to(token_n + 1, op_codes::OPEN_BLOCK);

                // Transform those tokens into result expressions
                let exprs = convert_tokens_into_res_expressions(
                    condition_tokens
                        .clone()
                        .iter()
                        .map(|(_, token)| token.clone())
                        .collect(),
                );

                // Scope tree
                let scope_tree = Mutex::new(ast_operations::Expression::new());

                // Ignore the if conditions and {
                let open_block_index = token_n + condition_tokens.len() + 1;

                // Get all tokens inside the if block
                let block_tokens = get_tokens_in_group_of(
                    open_block_index,
                    op_codes::OPEN_BLOCK,
                    op_codes::CLOSE_BLOCK,
                );

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree);

                // Ignore the whilte body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a while definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token = ast_operations::While::new(exprs.clone(), body.to_vec());
                ast_tree.body.push(Box::new(ast_token));
            }

            // Return statement
            op_codes::RETURN => {
                let next_token = tokens[token_n + 1].clone();

                let (size, return_val) =
                    get_assignment_token(next_token.value.clone(), token_n + 1);

                let ast_token = ast_operations::ReturnStatement { value: return_val };
                ast_tree.body.push(Box::new(ast_token));

                token_n += 2 + size;
            }

            // If statement
            op_codes::IF_CONDITIONAL => {
                // Get the if condition tokens
                let condition_tokens = get_tokens_from_to(token_n + 1, op_codes::OPEN_BLOCK);

                // Transform those tokens into result expressions
                let exprs = convert_tokens_into_res_expressions(
                    condition_tokens
                        .clone()
                        .iter()
                        .map(|(_, token)| token.clone())
                        .collect(),
                );

                // Scope tree
                let scope_tree = Mutex::new(ast_operations::Expression::new());

                // Ignore the if conditions and {
                let open_block_index = token_n + condition_tokens.len() + 1;

                // Get all tokens inside the if block
                let block_tokens = get_tokens_in_group_of(
                    open_block_index,
                    op_codes::OPEN_BLOCK,
                    op_codes::CLOSE_BLOCK,
                );

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree);

                // Ignore the block body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a if block definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token = ast_operations::IfConditional::new(exprs.clone(), body.to_vec());
                ast_tree.body.push(Box::new(ast_token));
            }

            op_codes::PROP_ACCESS => {
                let previous_token = tokens[token_n - 1].clone();
                let next_token = tokens[token_n + 1].clone();

                if next_token.ast_type == op_codes::REFERENCE {
                    let after_next_token = tokens[token_n + 2].clone();

                    let reference_type = match after_next_token.ast_type {
                        op_codes::OPEN_PARENT => op_codes::FN_CALL,
                        _ => 0,
                    };

                    match reference_type {
                        /*
                         * Call to functions from variables
                         */
                        op_codes::FN_CALL => {
                            let mut ast_token = ast_operations::FnCall::new(
                                next_token.value.clone(),
                                Some(previous_token.value),
                            );

                            // Ignore itself and the (
                            let starting_token = token_n + 2;

                            let arguments_tokens = get_tokens_in_group_of(
                                starting_token,
                                op_codes::OPEN_PARENT,
                                op_codes::CLOSE_PARENT,
                            );
                            let arguments = convert_tokens_into_arguments(arguments_tokens.clone());

                            token_n += 2 + arguments_tokens.len();

                            ast_token.arguments = arguments;
                            ast_tree.body.push(Box::new(ast_token));
                        }
                        /*
                         * TODO: Access properties from varibles
                         */
                        op_codes::REFERENCE => {}
                        _ => (),
                    };
                }
            }

            // Function definition
            op_codes::FN_DEF => {
                let def_name = String::from(&tokens[token_n + 1].value.clone());

                // Scope tree
                let scope_tree = Mutex::new(ast_operations::Expression::new());

                // Ignore function name and the (
                let starting_token = token_n + 2;

                // Get function arguments, WIP
                let arguments: Vec<String> = get_tokens_in_group_of(
                    starting_token,
                    op_codes::OPEN_PARENT,
                    op_codes::CLOSE_PARENT,
                )
                .iter()
                .map(|token| token.value.clone())
                .collect();

                // Ignore function name, (, arguments and )
                let open_block_index = starting_token + arguments.len() + 2;

                // Get all tokens inside the function block

                let block_tokens = get_tokens_in_group_of(
                    open_block_index,
                    op_codes::OPEN_BLOCK,
                    op_codes::CLOSE_BLOCK,
                );

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree);

                // Ignore the function body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a function definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token =
                    ast_operations::FnDefinition::new(def_name, body.to_vec(), arguments);
                ast_tree.body.push(Box::new(ast_token));
            }
            // Variable definition
            op_codes::VAR_DEF => {
                let next_token = tokens[token_n + 1].clone();

                // Variable name
                let def_name = next_token.value.clone();

                // Value token position
                let val_index = token_n + 3;

                // Stringified value
                let def_value = String::from(&tokens[val_index].value.clone());

                let (size, assignment) = get_assignment_token(def_value, val_index);

                let ast_token = ast_operations::VarDefinition::new(def_name, assignment);
                ast_tree.body.push(Box::new(ast_token));

                token_n += 3 + size;
            }
            // References (fn calls, variable reassignation...)
            op_codes::REFERENCE => {
                let next_token = &tokens[token_n + 1];

                let reference_type = match next_token.ast_type {
                    op_codes::OPEN_PARENT => op_codes::FN_CALL,
                    op_codes::LEFT_ASSIGN => op_codes::VAR_ASSIGN,
                    _ => 0,
                };

                match reference_type {
                    op_codes::VAR_ASSIGN => {
                        let token_after_equal = tokens[token_n + 2].clone();

                        let (size, assignment) =
                            get_assignment_token(token_after_equal.value.clone(), token_n + 2);

                        let ast_token = ast_operations::VarAssignment::new(
                            current_token.value.clone(),
                            assignment,
                        );

                        ast_tree.body.push(Box::new(ast_token));

                        token_n += 2 + size;
                    }
                    op_codes::FN_CALL => {
                        let mut ast_token =
                            ast_operations::FnCall::new(current_token.value.clone(), None);

                        // Ignore itself and the (
                        let starting_token = token_n + 1;

                        let arguments_tokens = get_tokens_in_group_of(
                            starting_token,
                            op_codes::OPEN_PARENT,
                            op_codes::CLOSE_PARENT,
                        );
                        let arguments = convert_tokens_into_arguments(arguments_tokens.clone());

                        token_n += 3 + arguments_tokens.len();

                        ast_token.arguments = arguments;

                        ast_tree.body.push(Box::new(ast_token));
                    }
                    _ => {
                        token_n += 1;
                    }
                }
            }
            _ => {
                token_n += 1;
            }
        }
    }
}

pub fn run_ast(
    ast: &Mutex<ast_operations::Expression>,
    stack: &Mutex<Stack>,
) -> Option<ast_operations::BoxedValue> {
    let ast = ast.lock().unwrap();

    // Closure version of resolve_reference
    let resolve_ref =
        |val_type: op_codes::Val,
         ref_val: Box<dyn primitive_values::PrimitiveValueBase>|
         -> Option<BoxedValue> { resolve_reference(stack, val_type, ref_val, &ast) };

    // Check if a conditional is true or not
    let eval_condition = |condition_code: op_codes::Val,
                          left_val: ast_operations::BoxedValue,
                          right_val: ast_operations::BoxedValue|
     -> bool {
        let left_val = resolve_ref(left_val.interface, left_val.value.clone());
        let right_val = resolve_ref(right_val.interface, right_val.value.clone());

        if let (Some(left_val), Some(right_val)) = (left_val, right_val) {
            match condition_code {
                // Handle !=
                op_codes::NOT_EQUAL_CONDITION => {
                    let left_val = value_to_string(left_val, stack).unwrap();
                    let right_val = value_to_string(right_val, stack).unwrap();

                    left_val != right_val
                }
                // Handle ==
                op_codes::EQUAL_CONDITION => {
                    let left_val = value_to_string(left_val, stack).unwrap();
                    let right_val = value_to_string(right_val, stack).unwrap();

                    left_val == right_val
                }
                _ => false,
            }
        } else {
            false
        }
    };

    for operation in &ast.body {
        match operation.get_type() {
            /*
             * Handle if block
             */
            op_codes::WHILE_DEF => {
                let while_block = downcast_val::<ast_operations::While>(operation.as_self());

                let check_while = |while_block: &ast_operations::While| -> Option<BoxedValue> {
                    /*
                     * Evaluate all conditions,
                     * If all they return true then execute the IF's expression block
                     */
                    let mut true_count = 0;

                    for condition in while_block.conditions.clone() {
                        let res =
                            eval_condition(condition.relation, condition.left, condition.right);

                        if res {
                            true_count += 1;
                        }
                    }

                    if true_count == while_block.conditions.len() {
                        let expr = ast_operations::Expression::from_body(while_block.body.clone());
                        let expr_id = expr.expr_id.clone();

                        // Execute the expression block
                        let if_block_return = run_ast(&Mutex::new(expr), stack);

                        if let Some(if_block_return) = if_block_return {
                            return Some(if_block_return);
                        }

                        // Clean the expression definitions from the stack
                        stack.lock().unwrap().drop_ops_from_id(expr_id);
                        Some(BoxedValue {
                            value: Box::new(Boolean(false)),
                            interface: op_codes::WHILE_DEF,
                        })
                    } else {
                        None
                    }
                };

                let mut stopped = false;

                while !stopped {
                    let res = check_while(while_block);

                    if let Some(res) = res {
                        if res.interface != op_codes::WHILE_DEF {
                            return Some(res);
                        }
                    } else {
                        stopped = true;
                    }
                }
            }

            /*
             * Handle return statements
             */
            op_codes::RETURN => {
                let statement =
                    downcast_val::<ast_operations::ReturnStatement>(operation.as_self());

                // Type of return
                let return_type = statement.value.interface;

                // Value returning
                let return_val = statement.value.value.clone();

                // Pimitive value to return
                let return_val = resolve_ref(return_type, return_val);

                return return_val;
            }

            /*
             * Handle if statements
             */
            op_codes::IF_CONDITIONAL => {
                let if_statement =
                    downcast_val::<ast_operations::IfConditional>(operation.as_self());

                /*
                 * Evaluate all conditions,
                 * If all they return true then execute the IF's expression block
                 */
                let mut true_count = 0;

                for condition in if_statement.conditions.clone() {
                    let res = eval_condition(condition.relation, condition.left, condition.right);

                    if res {
                        true_count += 1;
                    }
                }
                if true_count == if_statement.conditions.len() {
                    let expr = ast_operations::Expression::from_body(if_statement.body.clone());
                    let expr_id = expr.expr_id.clone();

                    // Execute the expression block
                    let if_block_return = run_ast(&Mutex::new(expr), stack);

                    if let Some(if_block_return) = if_block_return {
                        return Some(if_block_return);
                    }

                    // Clean the expression definitions from the stack
                    stack.lock().unwrap().drop_ops_from_id(expr_id.clone());
                }
            }

            /*
             * Handle function definitions
             */
            op_codes::FN_DEF => {
                let function = downcast_val::<ast_operations::FnDefinition>(operation.as_self());

                stack.lock().unwrap().push_function(FunctionDef {
                    name: function.def_name.clone(),
                    body: function.body.clone(),
                    arguments: function.arguments.clone(),
                    cb: |args, args_vals, body, stack, ast| {
                        let expr = ast_operations::Expression::from_body(body.clone());
                        let expr_id = expr.expr_id.clone();

                        for (i, arg) in args_vals.iter().enumerate() {
                            let arg_name = args[i].clone();
                            let var_id = stack.lock().unwrap().reseve_index();
                            stack.lock().unwrap().push_variable(VariableDef {
                                name: arg_name,
                                value: arg.value.clone(),
                                val_type: arg.interface,
                                expr_id: expr_id.clone(),
                                functions: get_methods_in_type(arg.interface),
                                var_id,
                            })
                        }

                        let return_val = run_ast(&Mutex::new(expr), stack);

                        stack.lock().unwrap().drop_ops_from_id(expr_id);

                        if let Some(return_val) = return_val {
                            resolve_reference(stack, return_val.interface, return_val.value, &ast)
                        } else {
                            return_val
                        }
                    },
                    // TODO: Move away from Uuid
                    expr_id: Uuid::new_v4().to_string(),
                });
            }

            /*
             * Handle variables definitions
             */
            op_codes::VAR_DEF => {
                let variable = downcast_val::<ast_operations::VarDefinition>(operation.as_self());

                let val_type = variable.assignment.interface;
                let ref_val = variable.assignment.value.clone();

                let var_ref = resolve_ref(val_type, ref_val);

                if let Some(var_ref) = var_ref {
                    // Take a id for the stack
                    let var_id = stack.lock().unwrap().reseve_index();

                    // Push the variable into the stack
                    stack.lock().unwrap().push_variable(VariableDef {
                        name: variable.def_name.clone(),
                        val_type: var_ref.interface,
                        value: var_ref.value,
                        expr_id: ast.expr_id.clone(),
                        functions: get_methods_in_type(var_ref.interface),
                        var_id,
                    });
                }
            }

            /*
             * Handle variable assignments
             */
            op_codes::VAR_ASSIGN => {
                let variable = downcast_val::<ast_operations::VarAssignment>(operation.as_self());

                let is_pointer = variable.var_name.starts_with('&');

                let variable_name = if is_pointer {
                    // Remove & from it's name
                    let mut variable_name = variable.var_name.clone();
                    variable_name.remove(0);
                    variable_name
                } else {
                    variable.var_name.clone()
                };

                let ref_val = resolve_ref(
                    variable.assignment.interface,
                    variable.assignment.value.clone(),
                );

                if let Some(ref_val) = ref_val {
                    stack.lock().unwrap().modify_var(variable_name, ref_val);
                }
            }

            /*
             * Handle function calls
             */
            op_codes::FN_CALL => {
                let fn_call = downcast_val::<ast_operations::FnCall>(operation.as_self());

                let is_referenced = fn_call.reference_to.is_some();

                let function = if is_referenced {
                    let reference_to = fn_call.reference_to.as_ref().unwrap();
                    let variable = stack
                        .lock()
                        .unwrap()
                        .get_variable_by_name(reference_to.as_str());
                    variable
                        .unwrap()
                        .get_function_by_name(fn_call.fn_name.as_str())
                } else {
                    stack
                        .lock()
                        .unwrap()
                        .get_function_by_name(fn_call.fn_name.as_str())
                };

                // If the calling function is found
                if let Some(function) = function {
                    let mut arguments = Vec::new();

                    if is_referenced {
                        let reference_to = fn_call.reference_to.as_ref().unwrap();
                        arguments.push(BoxedValue {
                            interface: op_codes::STRING,
                            value: Box::new(StringVal(reference_to.to_string())),
                        });
                    }

                    for argument in &fn_call.arguments {
                        let arg_ref = resolve_ref(argument.interface, argument.value.clone());
                        if let Some(arg_ref) = arg_ref {
                            arguments.push(arg_ref);
                        } else {
                            // Broken argument
                        }
                    }

                    let res_func = (function.cb)(
                        function.arguments,
                        arguments.clone(),
                        function.body,
                        &stack,
                        &ast,
                    );

                    if let Some(ret_val) = res_func {
                        let val_stringified = value_to_string(ret_val, stack);

                        if let Ok(val_stringified) = val_stringified {
                            // The function returned something that ends up not being used, throw error

                            errors::raise_error(
                                errors::RETURNED_VALUE_NOT_USED,
                                vec![
                                    val_stringified,
                                    fn_call.fn_name.clone(),
                                    values_to_strings(arguments, stack).join(" "),
                                ],
                            )
                        }
                    } else {
                        // No value returned, OK
                    }
                }
            }
            _ => {
                panic!("Unhandled code operation")
            }
        }
    }
    None
}
