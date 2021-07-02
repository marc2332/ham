use crate::{
    ast_types::{
        ast_base::AstBase,
        boxed_val::BoxedValue,
        expression::Expression,
    },
    primitive_values::primitive_base::PrimitiveValueBase,
    runtime::{
        downcast_val,
        value_to_string,
    },
    stack::Stack,
    utils::Ops,
};
use serde::Serialize;
use std::{
    any::Any,
    sync::{
        Mutex,
        MutexGuard,
    },
};

/*
 * Number
 */

#[derive(Clone, Debug, Serialize)]
pub struct Number(pub usize);

impl PrimitiveValueBase for Number {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/*
 * Number base
 */
pub trait NumberValueBase {
    fn new(value: usize) -> Self;
    fn get_state(&self) -> usize;

    fn mut_sum(
        args: Vec<String>,
        args_vals: Vec<BoxedValue>,
        body: Vec<Box<dyn AstBase>>,
        stack: &Mutex<Stack>,
        ast: &MutexGuard<Expression>,
    ) -> Option<BoxedValue>;

    fn sum(
        args: Vec<String>,
        args_vals: Vec<BoxedValue>,
        body: Vec<Box<dyn AstBase>>,
        stack: &Mutex<Stack>,
        ast: &MutexGuard<Expression>,
    ) -> Option<BoxedValue>;
}

impl NumberValueBase for Number {
    fn new(value: usize) -> Self {
        Self(value)
    }

    fn get_state(&self) -> usize {
        self.0
    }

    /*
     * function: sum()
     *
     * Returns the variable's value plus the argument
     */
    fn sum(
        _: Vec<String>,
        args_vals: Vec<BoxedValue>,
        _: Vec<Box<dyn AstBase>>,
        stack: &Mutex<Stack>,
        _: &MutexGuard<Expression>,
    ) -> Option<BoxedValue> {
        let var_name = value_to_string(args_vals[0].clone(), stack).unwrap();
        let new_val = downcast_val::<Number>(args_vals[1].value.as_self()).0;

        // Get the variable from the stack
        let variable = stack
            .lock()
            .unwrap()
            .get_variable_by_name(var_name.as_str());

        match variable {
            Some(current_var) => {
                let current_var = downcast_val::<Number>(current_var.value.as_self());
                let current_val = current_var.get_state();

                let new_val = Number::new(current_val + new_val);

                Some(BoxedValue {
                    interface: Ops::Number,
                    value: Box::new(new_val),
                })
            }
            _ => None,
        }
    }

    /*
     * function: mut_sum()
     *
     * Assigns to the variable value it's value plus the argument
     */
    fn mut_sum(
        _: Vec<String>,
        args_vals: Vec<BoxedValue>,
        _: Vec<Box<dyn AstBase>>,
        stack: &Mutex<Stack>,
        _: &MutexGuard<Expression>,
    ) -> Option<BoxedValue> {
        let var_name = value_to_string(args_vals[0].clone(), stack).unwrap();
        let new_val = downcast_val::<Number>(args_vals[1].value.as_self()).0;

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
                    interface: Ops::Number,
                    value: Box::new(new_val),
                },
            );
        }

        None
    }
}
