use std::any::Any;
use std::sync::{Mutex, MutexGuard};

use crate::ast::ast_operations;
use crate::ast::ast_operations::{BoxedValue, FnCallBase, ResultExpressionBase};
use crate::stack::{FunctionDef, Stack, VariableDef};
use crate::types::{IndexedTokenList, Token, TokensList};
use crate::utils::op_codes::Directions;
use crate::utils::primitive_values::{
    BooleanValueBase, Number, NumberValueBase, ReferenceValueBase, StringVal, StringValueBase,
};
use crate::utils::{errors, op_codes, primitive_values};

pub fn downcast_val<T: 'static>(val: &dyn Any) -> &T {
    val.downcast_ref::<T>().unwrap()
}

/*
 * Search variables in the stack by its name
 *
 * IDEA: deprecate referencing variables by it's name but instead use a uuid
 */
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
pub fn get_func_fn(fn_name: String, functions: &Vec<FunctionDef>) -> Result<FunctionDef, ()> {
    for op_fn in functions {
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

/*
 * Get the correct token
 */
pub fn get_assignment_token_fn(
    val: String,
    token_n: usize,
    tokens: TokensList,
    direction: Directions,
) -> (usize, ast_operations::BoxedValue) {
    match val.as_str() {
        // True boolean
        "true" => (
            1,
            ast_operations::BoxedValue {
                interface: op_codes::BOOLEAN,
                value: Box::new(primitive_values::Boolean::new(true)),
            },
        ),
        // False boolean
        "false" => (
            1,
            ast_operations::BoxedValue {
                interface: op_codes::BOOLEAN,
                value: Box::new(primitive_values::Boolean::new(false)),
            },
        ),
        // Numeric values
        val if val.parse::<usize>().is_ok() => (
            1,
            ast_operations::BoxedValue {
                interface: op_codes::NUMBER,
                value: Box::new(primitive_values::Number::new(val.parse::<usize>().unwrap())),
            },
        ),
        // String values
        val if val.chars().nth(0).unwrap() == '"'
            && val.chars().nth(val.len() - 1).unwrap() == '"' =>
        {
            (
                1,
                ast_operations::BoxedValue {
                    interface: op_codes::STRING,
                    value: Box::new(primitive_values::StringVal::new(String::from(val))),
                },
            )
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
                    op_codes::PROP_ACCESS => op_codes::PROP_ACCESS,
                    _ => 0,
                };

                match reference_type {
                    op_codes::PROP_ACCESS => {
                        let after_next_token = tokens[token_n + 2].clone();
                        let (size, val) = get_assignment_token_fn(
                            after_next_token.value.clone(),
                            token_n + 2,
                            tokens.clone(),
                            Directions::LeftToRight,
                        );

                        (size + 2, val)
                    }

                    op_codes::FN_CALL => {
                        // Position where it will be starting getting the argument tokens
                        let starting_token: usize = {
                            match direction {
                                Directions::LeftToRight => token_n + 2,
                                _ => token_n,
                            }
                        };

                        let previous_token = tokens[token_n - 1].clone();

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

                        let mut ast_token = ast_operations::FnCall::new(
                            {
                                match direction {
                                    // When reading from left to right, we know current token.value is it's name
                                    Directions::LeftToRight => String::from(val),

                                    // But when reading from right to left we need to first get all the tokens which are part of the function
                                    Directions::RightToLeft => {
                                        let fn_name =
                                            String::from(arguments_tokens[0].1.value.clone());

                                        // Now we can remove thefunction name from the arguments token
                                        arguments_tokens.remove(0);
                                        fn_name
                                    }
                                }
                            },
                            {
                                match previous_token.ast_type {
                                    op_codes::PROP_ACCESS => tokens[token_n - 2].value.clone(),
                                    _ => String::new(),
                                }
                            },
                        );

                        // Transfrom the tokens into arguments
                        ast_token.arguments = convert_tokens_into_arguments(
                            arguments_tokens
                                .clone()
                                .iter()
                                .map(|(_, token)| token.clone())
                                .collect(),
                        );

                        (
                            arguments_tokens.len() + 3,
                            ast_operations::BoxedValue {
                                interface: op_codes::FN_CALL,
                                value: Box::new(ast_token.clone()),
                            },
                        )
                    }
                    _ => (
                        1,
                        ast_operations::BoxedValue {
                            interface: op_codes::REFERENCE,
                            value: Box::new(primitive_values::Reference::new(String::from(val))),
                        },
                    ),
                }
            } else {
                (
                    1,
                    ast_operations::BoxedValue {
                        interface: op_codes::REFERENCE,
                        value: Box::new(primitive_values::Reference::new(String::from(val))),
                    },
                )
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
                let assigned_token = get_assignment_token_fn(
                    token.value.clone(),
                    token_n,
                    tokens.clone(),
                    Directions::LeftToRight,
                );

                match assigned_token.1.interface {
                    op_codes::FN_CALL => token_n += assigned_token.0 + 1,
                    _ => token_n += 1,
                }

                args.push(assigned_token.1);
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
                    left_token.1.clone(),
                    right_token.1.clone(),
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
 * Force the transformation from a primitive type into a String
 */
pub fn value_to_string(value: BoxedValue) -> Result<String, usize> {
    match value.interface {
        op_codes::BOOLEAN => Ok(
            downcast_val::<primitive_values::Boolean>(value.value.as_self())
                .0
                .to_string(),
        ),
        op_codes::STRING => Ok(
            downcast_val::<primitive_values::StringVal>(value.value.as_self())
                .0
                .clone(),
        ),
        op_codes::NUMBER => Ok(
            downcast_val::<primitive_values::Number>(value.value.as_self())
                .0
                .to_string(),
        ),
        _ => Err(value.interface),
    }
}

/*
 * Transform a group of boxed values into strings
 */
pub fn values_to_strings(values: Vec<BoxedValue>) -> Vec<String> {
    return values
        .iter()
        .map(|arg| value_to_string(arg.clone()).unwrap())
        .collect();
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
) -> Result<BoxedValue, ()> {
    match val_type {
        op_codes::STRING => Ok(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        op_codes::BOOLEAN => Ok(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        op_codes::NUMBER => Ok(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        op_codes::REFERENCE => {
            let val = downcast_val::<primitive_values::Reference>(ref_val.as_self())
                .0
                .clone();

            let ref_def = get_var_reference_fn(stack, val);

            if ref_def.is_ok() {
                let ref_assign = ref_def.unwrap();
                Ok(BoxedValue {
                    interface: ref_assign.val_type,
                    value: ref_assign.value,
                })
            } else {
                Err(())
            }
        }
        op_codes::FN_CALL => {
            let fn_call = downcast_val::<ast_operations::FnCall>(ref_val.as_self());

            let is_referenced = fn_call.reference_to != "";

            let ref_fn = if is_referenced {
                let ref_var = get_var_reference_fn(stack, fn_call.reference_to.clone());
                get_func_fn(fn_call.fn_name.clone(), &ref_var.unwrap().methods)
            } else {
                get_func_fn(
                    fn_call.fn_name.clone(),
                    &stack.lock().unwrap().clone().get_functions(),
                )
            };

            // If the calling function is found
            if ref_fn.is_ok() {
                let mut arguments = Vec::new();

                if is_referenced {
                    arguments.push(BoxedValue {
                        interface: op_codes::STRING,
                        value: Box::new(StringVal(fn_call.reference_to.clone())),
                    });
                }

                for argument in &fn_call.arguments {
                    let arg_ref =
                        resolve_reference(argument.interface, argument.value.clone(), stack, &ast);

                    if arg_ref.is_ok() {
                        arguments.push(arg_ref.unwrap());
                    }
                }

                if ref_fn.is_ok() {
                    let func = ref_fn.unwrap();
                    let return_val = (func.cb)(func.arguments, arguments, func.body, &stack, &ast);
                    return return_val;
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

/*
 * Modify a variable value
 */
pub fn modify_var(
    stack: &Mutex<Stack>,
    var_name: String,
    value: Box<dyn primitive_values::PrimitiveValueBase>,
) {
    let mut stack = stack.lock().unwrap();

    for mut op_var in stack.variables.iter_mut() {
        if op_var.name == var_name {
            op_var.value = value.clone();
            return ();
        }
    }

    errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
}

/*
 * Returns the methods for the specified primitive type
 */
pub fn get_methods_in_type(val_type: op_codes::Val) -> Vec<FunctionDef> {
    let mut res = Vec::new();

    match val_type {
        op_codes::NUMBER => {
            res.push(FunctionDef {
                name: String::from("sum"),
                body: vec![],
                cb: |_, args, _, stack, _| {
                    let var_name = value_to_string(args[0].clone()).unwrap();
                    let new_val = downcast_val::<Number>(args[1].value.as_self()).0;

                    let var_ref = get_var_reference_fn(stack, var_name.clone());

                    if var_ref.is_ok() {
                        let current_var = var_ref.unwrap();
                        let current_var = downcast_val::<Number>(current_var.value.as_self());
                        let current_val = current_var.get_state();

                        let new_val = Number::new(current_val + new_val);

                        return Ok(BoxedValue {
                            interface: op_codes::NUMBER,
                            value: Box::new(new_val),
                        });
                    }

                    Err(())
                },
                expr_id: "".to_string(),
                arguments: vec![],
            });
            res.push(FunctionDef {
                name: String::from("mut_sum"),
                body: vec![],
                cb: |_, args, _, stack, _| {
                    let var_name = value_to_string(args[0].clone()).unwrap();
                    let new_val = downcast_val::<Number>(args[1].value.as_self()).0;

                    let var_ref = get_var_reference_fn(stack, var_name.clone());

                    if var_ref.is_ok() {
                        let current_var = var_ref.unwrap();
                        let current_val = downcast_val::<Number>(current_var.value.as_self());
                        let current_num = current_val.get_state();

                        let new_val = Number::new(current_num + new_val);

                        modify_var(stack, var_name, Box::new(new_val));
                    }

                    Err(())
                },
                expr_id: "".to_string(),
                arguments: vec![],
            });
        }
        _ => (),
    }

    res
}
