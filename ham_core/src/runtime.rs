use crate::ast::ast_operations;
use crate::ast::ast_operations::BoxedValue;
use crate::stack::{FunctionDef, FunctionsContainer, Stack};
use crate::utils::primitive_values::{Number, NumberValueBase, StringVal};
use crate::utils::{op_codes, primitive_values};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

/*
 * Shorthand and *unsafe* way to downcast values
 */
pub fn downcast_val<T: 'static>(val: &dyn Any) -> &T {
    val.downcast_ref::<T>().unwrap()
}

/*
 * Force the transformation from a primitive type into a String
 */
pub fn value_to_string(value: BoxedValue, stack: &Mutex<Stack>) -> Result<String, usize> {
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
        op_codes::POINTER => {
            let pointer = downcast_val::<primitive_values::Pointer>(value.value.as_self()).0;
            let variable = stack.lock().unwrap().get_variable_by_id(pointer).unwrap();

            value_to_string(
                BoxedValue {
                    value: variable.value,
                    interface: variable.val_type,
                },
                stack,
            )
        }
        _ => Err(value.interface),
    }
}

/*
 * Transform a group of boxed values into strings
 */
pub fn values_to_strings(values: Vec<BoxedValue>, stack: &Mutex<Stack>) -> Vec<String> {
    return values
        .iter()
        .map(|arg| value_to_string(arg.clone(), stack).unwrap())
        .collect();
}

/*
 * Returns the methods for the specified primitive type
 */
pub fn get_methods_in_type(val_type: op_codes::Val) -> HashMap<String, FunctionDef> {
    let mut res = HashMap::new();

    match val_type {
        op_codes::NUMBER => {
            /*
             * function: sum()
             *
             * Returns the variable value plus the argument
             */
            res.insert(
                "sum".to_string(),
                FunctionDef {
                    name: String::from("sum"),
                    body: vec![],
                    cb: |_, args, _, stack, _| {
                        let var_name = value_to_string(args[0].clone(), stack).unwrap();
                        let new_val = downcast_val::<Number>(args[1].value.as_self()).0;

                        // Get the variable from the stack
                        let variable = stack
                            .lock()
                            .unwrap()
                            .get_variable_by_name(var_name.as_str());

                        match variable {
                            Some(current_var) => {
                                let current_var =
                                    downcast_val::<Number>(current_var.value.as_self());
                                let current_val = current_var.get_state();

                                let new_val = Number::new(current_val + new_val);

                                Some(BoxedValue {
                                    interface: op_codes::NUMBER,
                                    value: Box::new(new_val),
                                })
                            }
                            _ => None,
                        }
                    },
                    expr_id: "".to_string(),
                    arguments: vec![],
                },
            );
            /*
             * function: mut_sum()
             *
             * Assigns to the variable value it's value plus the argument
             */
            res.insert(
                "mut_sum".to_string(),
                FunctionDef {
                    name: String::from("mut_sum"),
                    body: vec![],
                    cb: |_, args, _, stack, _| {
                        let var_name = value_to_string(args[0].clone(), stack).unwrap();
                        let new_val = downcast_val::<Number>(args[1].value.as_self()).0;

                        // Get the variable from the stack
                        let var_ref = stack
                            .lock()
                            .unwrap()
                            .get_variable_by_name(var_name.as_str());

                        if let Some(current_var) = var_ref {
                            let current_val = downcast_val::<Number>(current_var.value.as_self());
                            let current_num = current_val.get_state();

                            let new_val = Number::new(current_num + new_val);

                            stack.lock().unwrap().modify_var(
                                var_name,
                                BoxedValue {
                                    interface: op_codes::NUMBER,
                                    value: Box::new(new_val),
                                },
                            );
                        }

                        None
                    },
                    expr_id: "".to_string(),
                    arguments: vec![],
                },
            );
        }
        _ => (),
    }

    res
}

/*
 * For static values it will just return the input but for references it will resolve its value
 * and then return it
 */
pub fn resolve_reference(
    stack: &Mutex<Stack>,
    val_type: op_codes::Val,
    ref_val: Box<dyn primitive_values::PrimitiveValueBase>,
    ast: &MutexGuard<ast_operations::Expression>,
) -> Option<BoxedValue> {
    match val_type {
        op_codes::POINTER => {
            let pointer = downcast_val::<primitive_values::Pointer>(ref_val.as_self()).0;
            let variable = stack
                .lock()
                .unwrap()
                .get_variable_by_id(pointer)
                .unwrap()
                .clone();

            Some(BoxedValue {
                value: variable.value,
                interface: variable.val_type,
            })
        }
        op_codes::STRING => Some(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        op_codes::BOOLEAN => Some(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        op_codes::NUMBER => Some(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        op_codes::REFERENCE => {
            let mut variable_name = downcast_val::<ast_operations::Reference>(ref_val.as_self())
                .0
                .clone();

            let is_pointer = variable_name.starts_with('&');

            let variable_name = if is_pointer {
                // Remove & from it's name
                variable_name.remove(0);
                variable_name
            } else {
                variable_name
            };

            let variable = stack
                .lock()
                .unwrap()
                .get_variable_by_name(variable_name.as_str());

            if let Some(variable) = variable {
                if is_pointer {
                    // Return a pointer
                    Some(BoxedValue {
                        interface: op_codes::POINTER,
                        value: Box::new(primitive_values::Pointer(variable.var_id)),
                    })
                } else {
                    // Return a copy of it's value
                    Some(BoxedValue {
                        interface: variable.val_type,
                        value: variable.value,
                    })
                }
            } else {
                None
            }
        }
        op_codes::FN_CALL => {
            let fn_call = downcast_val::<ast_operations::FnCall>(ref_val.as_self());

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
            if function.is_some() {
                let function = function.unwrap();
                let mut arguments = Vec::new();

                if is_referenced {
                    let reference_to = fn_call.reference_to.as_ref().unwrap();
                    arguments.push(BoxedValue {
                        interface: op_codes::STRING,
                        value: Box::new(StringVal(reference_to.to_string())),
                    });
                }

                for argument in &fn_call.arguments {
                    let arg_ref =
                        resolve_reference(stack, argument.interface, argument.value.clone(), &ast);

                    if arg_ref.is_some() {
                        arguments.push(arg_ref.unwrap());
                    }
                }
                let function_result =
                    (function.cb)(function.arguments, arguments, function.body, &stack, &ast);

                return function_result;
            } else {
                None
            }
        }
        _ => None,
    }
}
