use crate::ast::ast_operations;
use crate::ast::ast_operations::{AstBase, BoxedValue};
use crate::runtime::{value_to_string, values_to_strings};
use crate::utils::primitive_values::StringVal;
use crate::utils::{errors, op_codes, primitive_values};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::{thread, time};

/*
 * Get a function by it's name
 */
fn get_function(
    fn_name: &str,
    functions: &HashMap<String, FunctionDef>,
) -> Result<FunctionDef, ()> {
    let op_fn: Option<&FunctionDef> = functions.get(fn_name);

    if op_fn.is_some() {
        Ok(op_fn.unwrap().clone())
    } else {
        errors::raise_error(errors::FUNCTION_NOT_FOUND, vec![fn_name.to_string()]);
        Err(())
    }
}

/*
 * Variable definition stored on the memory stack
 */
#[derive(Clone)]
pub struct VariableDef {
    pub name: String,
    pub val_type: op_codes::Val,
    pub value: Box<dyn primitive_values::PrimitiveValueBase>,
    pub expr_id: String,
    pub functions: HashMap<String, FunctionDef>,
}

impl FunctionsContainer for VariableDef {
    fn get_function(&self, fn_name: &str) -> Result<FunctionDef, ()> {
        get_function(fn_name, &self.functions)
    }
}

/*
 * Function definition stored on the memory stack
 */
#[derive(Clone)]
pub struct FunctionDef {
    pub name: String,
    pub body: Vec<Box<dyn AstBase>>,
    pub cb: fn(
        args: Vec<String>,
        args_vals: Vec<BoxedValue>,
        body: Vec<Box<dyn AstBase>>,
        stack: &Mutex<Stack>,
        ast: &MutexGuard<ast_operations::Expression>,
    ) -> Option<ast_operations::BoxedValue>,
    pub expr_id: String,
    pub arguments: Vec<String>,
}

/*
 * Returns a function by it's name
 */
pub trait FunctionsContainer {
    fn get_function(&self, fn_name: &str) -> Result<FunctionDef, ()>;
}

/*
 * Implementation of the stack
 */
#[derive(Clone)]
pub struct Stack {
    pub functions: HashMap<String, FunctionDef>,
    pub variables: HashMap<String, VariableDef>,
}

impl FunctionsContainer for Stack {
    fn get_function(&self, fn_name: &str) -> Result<FunctionDef, ()> {
        get_function(fn_name, &self.functions)
    }
}

impl Stack {
    pub fn new(expr_id: String) -> Stack {
        let mut functions = HashMap::new();

        /*
         * format() function
         */
        functions.insert(
            "format".to_string(),
            FunctionDef {
                name: "format".to_string(),
                body: vec![],
                arguments: vec![],
                cb: |_, args, _, _, _| {
                    let mut args = values_to_strings(args);

                    let mut template = args[0].clone();

                    args.remove(0);

                    for arg in args {
                        template = template.replacen("{}", arg.as_str(), 1);
                    }

                    Some(BoxedValue {
                        interface: op_codes::STRING,
                        value: Box::new(StringVal(template)),
                    })
                },
                expr_id: expr_id.clone(),
            },
        );

        /*
         * print() function
         */
        functions.insert(
            "print".to_string(),
            FunctionDef {
                name: "print".to_string(),
                body: vec![],
                arguments: vec![],
                cb: |_, args, _, _, _| {
                    print!("{}", values_to_strings(args).join(" "));
                    None
                },
                expr_id: expr_id.clone(),
            },
        );

        /*
         * println() function
         */
        functions.insert(
            "println".to_string(),
            FunctionDef {
                name: "println".to_string(),
                body: vec![],
                arguments: vec![],
                cb: |_, args, _, _, _| {
                    println!("{}", values_to_strings(args).join(""));
                    None
                },
                expr_id: expr_id.clone(),
            },
        );

        /*
         * wait() function
         */
        functions.insert(
            "wait".to_string(),
            FunctionDef {
                name: "wait".to_string(),
                body: vec![],
                arguments: vec![],
                cb: |_, args, _, _, _| {
                    let time = value_to_string(args[0].clone())
                        .unwrap()
                        .parse::<u64>()
                        .unwrap();
                    let time = time::Duration::from_millis(time);
                    thread::sleep(time);
                    None
                },
                expr_id: expr_id.clone(),
            },
        );

        Stack {
            variables: HashMap::new(),
            functions,
        }
    }

    /*
     * Drop from the stack all variables and functions which are port
     * of an specified expression ID
     *
     * This is mainly used in block expression (functions, if...)
     */
    pub fn drop_ops_from_id(&mut self, id: String) {
        self.variables.retain(|_, var| var.expr_id != id);
        self.functions.retain(|_, func| func.expr_id != id);
    }

    /*
     * Shorthand to push a function into the stack
     */
    pub fn push_function(&mut self, function: FunctionDef) {
        self.functions.insert(function.name.clone(), function);
    }

    /*
     * Search variables in the stack by its name
     *
     * IDEA: deprecate referencing variables by it's name but instead use a uuid
     */
    pub fn get_variable(&self, var_name: &str) -> Result<VariableDef, ()> {
        let op_var = self.variables.get(var_name);

        if op_var.is_some() {
            Ok(op_var.unwrap().clone())
        } else {
            errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.to_string()]);
            Err(())
        }
    }

    /*
     * Modify a variable value
     */
    pub fn modify_var(
        &mut self,
        var_name: String,
        value: Box<dyn primitive_values::PrimitiveValueBase>,
    ) {
        let mut_var = self.variables.get_mut(var_name.as_str());

        if mut_var.is_some() {
            let mut_var = mut_var.unwrap();
            mut_var.value = value;
        } else {
            errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
        }
    }
}
