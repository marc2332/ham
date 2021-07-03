use crate::{
    ast_types::{
        boxed_val::BoxedValue,
        expression::Expression,
        fn_call::FnCall,
        reference::Reference,
    },
    primitive_values::{
        boolean::Boolean,
        number::{
            Number,
            NumberValueBase,
        },
        pointer::Pointer,
        string::StringVal,
    },
    run_ast,
    stack::{
        FunctionDef,
        FunctionsContainer,
        Stack,
    },
    types::BoxedPrimitiveValue,
    utils::{
        errors,
        errors::raise_error,
        Ops,
    },
};
use std::{
    any::Any,
    collections::HashMap,
    sync::{
        Mutex,
        MutexGuard,
    },
};

/*
 * Shorthand and *unsafe* way to downcast values
 */
pub fn downcast_val<T: 'static>(val: &dyn Any) -> &T {
    val.downcast_ref::<T>().unwrap()
}

/*
 * Force the transformation from a primitive type into a String
 */
pub fn value_to_string(value: BoxedValue, stack: &Mutex<Stack>) -> Result<String, Ops> {
    match value.interface {
        Ops::Boolean => Ok(downcast_val::<Boolean>(value.value.as_self()).0.to_string()),
        Ops::String => Ok(downcast_val::<StringVal>(value.value.as_self()).0.clone()),
        Ops::Number => Ok(downcast_val::<Number>(value.value.as_self()).0.to_string()),
        Ops::Pointer => {
            let pointer = downcast_val::<Pointer>(value.value.as_self()).0;
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
pub fn get_methods_in_type(val_type: Ops) -> HashMap<String, FunctionDef> {
    let mut res = HashMap::new();

    /*
     * map a value type to it's implemented functions
     */
    match val_type {
        Ops::Number => {
            res.insert(
                "sum".to_string(),
                FunctionDef {
                    name: String::from("sum"),
                    body: vec![],
                    cb: Number::sum,
                    expr_id: "".to_string(),
                    arguments: vec![],
                },
            );

            res.insert(
                "mut_sum".to_string(),
                FunctionDef {
                    name: String::from("mut_sum"),
                    body: vec![],
                    cb: Number::mut_sum,
                    expr_id: "".to_string(),
                    arguments: vec![],
                },
            );
        }

        /*
         * TODO: Methods for strings
         */
        Ops::String => {}

        /*
         * TODO: Methods for booleans
         */
        Ops::Boolean => {}
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
    val_type: Ops,
    ref_val: BoxedPrimitiveValue,
    ast: &MutexGuard<Expression>,
) -> Option<BoxedValue> {
    match val_type {
        Ops::Expression => {
            let expr = downcast_val::<Expression>(ref_val.as_self());
            let res = run_ast(&Mutex::new(expr.clone()), stack);

            stack.lock().unwrap().drop_ops_from_id(expr.expr_id.clone());

            res
        }

        Ops::Pointer => {
            let pointer = downcast_val::<Pointer>(ref_val.as_self()).0;
            let variable = stack.lock().unwrap().get_variable_by_id(pointer);

            if let Some(variable) = variable {
                Some(BoxedValue {
                    value: variable.value,
                    interface: variable.val_type,
                })
            } else {
                raise_error(errors::CODES::BrokenPointer, vec![pointer.to_string()]);
                None
            }
        }
        Ops::String => Some(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        Ops::Boolean => Some(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        Ops::Number => Some(BoxedValue {
            interface: val_type,
            value: ref_val,
        }),
        Ops::Reference => {
            let mut referenced_variable = downcast_val::<Reference>(ref_val.as_self()).clone();

            let is_pointer = referenced_variable.0.starts_with('&');

            if is_pointer {
                // Remove & from it's name
                referenced_variable.0.remove(0);
            }

            let variable = stack
                .lock()
                .unwrap()
                .get_variable_by_name(referenced_variable.0.as_str());

            if let Some(variable) = variable {
                if is_pointer {
                    // Return a pointer
                    Some(BoxedValue {
                        interface: Ops::Pointer,
                        value: Box::new(Pointer(variable.var_id)),
                    })
                } else {
                    // Return a copy of it's value
                    Some(BoxedValue {
                        interface: variable.val_type,
                        value: variable.value,
                    })
                }
            } else {
                raise_error(errors::CODES::VariableNotFound, vec![referenced_variable.0]);
                None
            }
        }
        Ops::FnCall => {
            let fn_call = downcast_val::<FnCall>(ref_val.as_self());

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

                // Pass the reference name as first argument
                if is_referenced {
                    let reference_to = fn_call.reference_to.as_ref().unwrap();
                    arguments.push(BoxedValue {
                        interface: Ops::String,
                        value: Box::new(StringVal(reference_to.to_string())),
                    });
                }

                for argument in &fn_call.arguments {
                    let arg_ref =
                        resolve_reference(stack, argument.interface, argument.value.clone(), &ast);

                    if let Some(arg_ref) = arg_ref {
                        arguments.push(arg_ref);
                    } else {
                        // Broken argument
                    }
                }

                // Call the function and return it's result
                (function.cb)(function.arguments, arguments, function.body, &stack, &ast)
            } else {
                None
            }
        }
        _ => None,
    }
}
