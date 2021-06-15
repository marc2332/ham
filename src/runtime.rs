use std::any::Any;
use std::sync::{Mutex, MutexGuard};

use crate::ast::ast_operations;
use crate::ast::ast_operations::{FnCallBase, ResultExpressionBase};
use crate::stack::{FunctionDef, Stack, VariableDef};
use crate::types::{IndexedTokenList, Token, TokensList};
use crate::utils::op_codes::Directions;
use crate::utils::primitive_values::{
    BooleanValueBase, NumberValueBase, PrimitiveValueBase, ReferenceValueBase, StringValueBase,
};
use crate::utils::{errors, op_codes, primitive_values};

pub fn downcast_val<T: 'static>(val: &dyn Any) -> &T {
    val.downcast_ref::<T>().unwrap()
}

// Search variables in the stack by its name
pub fn get_var_reference_fn(stack: &Mutex<Stack>, var_name: String) -> Result<VariableDef, ()> {
    let stack = stack.lock().unwrap();

    for op_var in &stack.variables {
        if op_var.name == var_name {
            return Ok(op_var.clone());
        }
    }

    errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
    Err(())
}

// Search functions in the stack by its name
pub fn get_func_fn(fn_name: String, stack: &Mutex<Stack>) -> Result<FunctionDef, ()> {
    let stack = stack.lock().unwrap();
    for op_fn in &stack.functions {
        if op_fn.name == fn_name {
            return Ok(op_fn.clone());
        }
    }

    errors::raise_error(errors::FUNCTION_NOT_FOUND, vec![fn_name.clone()]);
    Err(())
}

// Get tokens with index starting on `from` until a token matches its type to `to`
pub fn get_tokens_from_to_fn(
    from: usize,
    to: op_codes::Val,
    tokens: TokensList,
    direction: Directions,
) -> IndexedTokenList {
    let mut found_tokens = Vec::new();

    let mut token_n = from;

    match direction {
        // Get tokens from left to right
        Directions::LeftToRight => {
            while token_n < tokens.len() {
                if tokens[token_n].ast_type == to {
                    break;
                } else {
                    found_tokens.push((token_n, tokens[token_n].clone()))
                }
                token_n += 1;
            }
        }

        // Get tokens from right to left
        Directions::RightToLeft => {
            while token_n > 0 {
                if tokens[token_n - 1].ast_type == to {
                    break;
                } else {
                    found_tokens.push((token_n - 1, tokens[token_n - 1].clone()))
                }
                token_n -= 1
            }

            found_tokens.reverse();
        }
    }

    found_tokens
}

pub fn get_assignment_token_fn(
    val: String,
    token_n: usize,
    tokens: TokensList,
    direction: Directions,
) -> ast_operations::BoxedValue {
    match val.as_str() {
        // True boolean
        "true" => ast_operations::BoxedValue {
            interface: op_codes::BOOLEAN,
            value: Box::new(primitive_values::Boolean::new(true)),
        },
        // False boolean
        "false" => ast_operations::BoxedValue {
            interface: op_codes::BOOLEAN,
            value: Box::new(primitive_values::Boolean::new(false)),
        },
        // Numeric values
        val if val.parse::<usize>().is_ok() => ast_operations::BoxedValue {
            interface: op_codes::NUMBER,
            value: Box::new(primitive_values::Number::new(val.parse::<usize>().unwrap())),
        },
        // String values
        val if val.chars().nth(0).unwrap() == '"'
            && val.chars().nth(val.len() - 1).unwrap() == '"' =>
        {
            ast_operations::BoxedValue {
                interface: op_codes::STRING,
                value: Box::new(primitive_values::StringVal::new(String::from(val))),
            }
        }
        // References to other values (ej: referencing to a variable)
        val => {
            if token_n < tokens.len() - 1 {
                let next_token = {
                    match direction {
                        Directions::LeftToRight => tokens[token_n + 1].clone(),
                        _ => tokens[token_n - 1].clone(),
                    }
                };

                let reference_type = match next_token.ast_type {
                    op_codes::OPEN_PARENT => op_codes::FN_CALL,
                    op_codes::CLOSE_PARENT => op_codes::FN_CALL,
                    _ => 0,
                };

                match reference_type {
                    op_codes::FN_CALL => {
                        // Position where it will be starting getting the argument tokens
                        let starting_token: usize = {
                            match direction {
                                Directions::LeftToRight => token_n + 2,
                                _ => token_n,
                            }
                        };

                        // Get argument tokens
                        let mut arguments_tokens: Vec<(usize, Token)> = {
                            match direction {
                                Directions::LeftToRight => get_tokens_from_to_fn(
                                    starting_token,
                                    op_codes::CLOSE_PARENT,
                                    tokens.clone(),
                                    direction.clone(),
                                ),
                                // WIP
                                Directions::RightToLeft => get_tokens_from_to_fn(
                                    starting_token,
                                    op_codes::IF_CONDITIONAL,
                                    tokens.clone(),
                                    direction.clone(),
                                ),
                            }
                        };

                        let mut ast_token = ast_operations::FnCall::new({
                            match direction {
                                // When reading from left to right, we know current token.value is it's name
                                Directions::LeftToRight => String::from(val),

                                // But when reading from right to left we need to first get all the tokens which are part of the function
                                Directions::RightToLeft => {
                                    let fn_name = String::from(arguments_tokens[0].1.value.clone());

                                    // Now we can remove thefunction name from the arguments token
                                    arguments_tokens.remove(0);
                                    fn_name
                                }
                            }
                        });

                        // Transfrom the tokens into arguments
                        ast_token.arguments = convert_tokens_into_arguments(
                            arguments_tokens
                                .clone()
                                .iter()
                                .map(|(_, token)| token.clone())
                                .collect(),
                        );

                        ast_operations::BoxedValue {
                            interface: op_codes::FN_CALL,
                            value: Box::new(ast_token.clone()),
                        }
                    }
                    _ => ast_operations::BoxedValue {
                        interface: op_codes::REFERENCE,
                        value: Box::new(primitive_values::Reference::new(String::from(val))),
                    },
                }
            } else {
                ast_operations::BoxedValue {
                    interface: op_codes::REFERENCE,
                    value: Box::new(primitive_values::Reference::new(String::from(val))),
                }
            }
        }
    }
}

// Get function arguments
pub fn convert_tokens_into_arguments(tokens: TokensList) -> Vec<ast_operations::BoxedValue> {
    let mut args = Vec::new();

    let mut token_n = 0;

    while token_n < tokens.len() {
        let token = tokens[token_n].clone();

        match token.ast_type {
            // Ignore ( and )
            op_codes::OPEN_PARENT => token_n += 1,
            op_codes::CLOSE_PARENT => token_n += 1,
            _ => {
                let arguments_tokens: Vec<(usize, Token)> = get_tokens_from_to_fn(
                    token_n,
                    op_codes::CLOSE_PARENT,
                    tokens.clone(),
                    Directions::LeftToRight,
                );

                let assigned_token = get_assignment_token_fn(
                    token.value.clone(),
                    token_n,
                    tokens.clone(),
                    Directions::LeftToRight,
                );

                match assigned_token.interface {
                    op_codes::FN_CALL => token_n += arguments_tokens.len() + 1,
                    _ => token_n += 1,
                }

                args.push(assigned_token);
            }
        }
    }

    args
}

pub fn convert_tokens_into_res_expressions(
    tokens: TokensList,
) -> Vec<ast_operations::ResultExpression> {
    let mut exprs = Vec::new();

    let mut token_n = 1;

    while token_n < tokens.len() {
        let left_token = tokens[token_n - 1].clone();
        let token = tokens[token_n].clone();

        match token.ast_type {
            op_codes::EQUAL_CONDITION => {
                let right_token = tokens[token_n + 1].clone();

                let left_token = get_assignment_token_fn(
                    left_token.value.clone(),
                    token_n,
                    tokens.clone(),
                    Directions::RightToLeft,
                );

                let right_token = get_assignment_token_fn(
                    right_token.value.clone(),
                    token_n + 1,
                    tokens.clone(),
                    Directions::LeftToRight,
                );

                exprs.push(ast_operations::ResultExpression::new(
                    op_codes::EQUAL_CONDITION,
                    left_token.clone(),
                    right_token.clone(),
                ));

                token_n += 2;
            }
            _ => {
                token_n += 1;
            }
        }
    }

    exprs
}

/*
 * Transform a primitive type into a String
 */
pub fn value_to_string(value: (usize, Box<dyn PrimitiveValueBase>)) -> Result<String, usize> {
    match value.0 {
        op_codes::BOOLEAN => Ok(downcast_val::<primitive_values::Boolean>(value.1.as_self())
            .0
            .to_string()),
        op_codes::STRING => Ok(
            downcast_val::<primitive_values::StringVal>(value.1.as_self())
                .0
                .clone(),
        ),
        op_codes::NUMBER => Ok(downcast_val::<primitive_values::Number>(value.1.as_self())
            .0
            .to_string()),
        _ => Err(value.0),
    }
}

/*
 * For static values it will just return the input but for references it will resolve its value
 * and then return it
 */
pub fn resolve_reference(
    val_type: op_codes::Val,
    ref_val: Box<dyn primitive_values::PrimitiveValueBase>,
    stack: &Mutex<Stack>,
    ast: &MutexGuard<ast_operations::Expression>,
) -> Result<(op_codes::Val, Box<dyn primitive_values::PrimitiveValueBase>), ()> {
    match val_type {
        op_codes::STRING => Ok((val_type, ref_val)),
        op_codes::BOOLEAN => Ok((val_type, ref_val)),
        op_codes::NUMBER => Ok((val_type, ref_val)),
        op_codes::REFERENCE => {
            let val = downcast_val::<primitive_values::Reference>(ref_val.as_self())
                .0
                .clone();

            let ref_def = get_var_reference_fn(stack, val);

            if ref_def.is_ok() {
                let ref_assign = ref_def.unwrap();
                Ok((ref_assign.val_type, ref_assign.value))
            } else {
                Err(())
            }
        }
        op_codes::FN_CALL => {
            let fn_call = downcast_val::<ast_operations::FnCall>(ref_val.as_self());
            let ref_fn = get_func_fn(fn_call.fn_name.clone(), stack);

            // If the calling function is found
            if ref_fn.is_ok() {
                let mut arguments = Vec::new();

                for argument in &fn_call.arguments {
                    let arg_ref =
                        resolve_reference(argument.interface, argument.value.clone(), stack, &ast);

                    let arg_stringified = value_to_string(arg_ref.unwrap());

                    if arg_stringified.is_ok() {
                        arguments.push(arg_stringified.unwrap());
                    }
                }

                if ref_fn.is_ok() {
                    let func = ref_fn.unwrap();
                    let return_val = (func.cb)(func.arguments, arguments, func.body, &stack, &ast);
                    if return_val.is_ok() {
                        let boxed_val = return_val.unwrap();
                        Ok((boxed_val.interface, boxed_val.value))
                    } else {
                        Err(())
                    }
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
        }
        _ => Err(()),
    }
}
