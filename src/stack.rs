use crate::ast::ast_operations;
use crate::ast::ast_operations::{AstBase, BoxedValue};
use crate::runtime::{value_to_string, values_to_strings};
use crate::utils::primitive_values::StringVal;
use crate::utils::{errors, op_codes, primitive_values};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::{thread, time};

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
    pub var_id: String,
}

impl FunctionsContainer for VariableDef {
    fn get_function_by_name(&self, fn_name: &str) -> Option<FunctionDef> {
        let op_fn: Option<&FunctionDef> = self.functions.get(fn_name);

        if op_fn.is_some() {
            Some(op_fn.unwrap().clone())
        } else {
            errors::raise_error(errors::FUNCTION_NOT_FOUND, vec![fn_name.to_string()]);
            None
        }
    }
    fn push_function(&mut self, function: FunctionDef) {
        self.functions.insert(function.name.clone(), function);
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
 * Common layer for functions container (ex: variables, memory stack)
 */
pub trait FunctionsContainer {
    /*
     * Return the requested function if found
     */
    fn get_function_by_name(&self, fn_name: &str) -> Option<FunctionDef>;
    /*
     * Push a function into the container
     */
    fn push_function(&mut self, function: FunctionDef);
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
    fn get_function_by_name(&self, fn_name: &str) -> Option<FunctionDef> {
        for (_, function) in &self.functions {
            if function.name == fn_name.to_string() {
                return Some(function.clone());
            }
        }
        return None;
    }
    fn push_function(&mut self, function: FunctionDef) {
        self.functions.insert(function.expr_id.clone(), function);
    }
}

impl Stack {
    pub fn new(expr_id: String) -> Stack {
        let mut functions = HashMap::new();

        /*
         * format() function
         *
         * Example:
         *
         * let msg = format("Hello {} from {}", "people", "world")
         *
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
     * Print variables and functions stored on stack
     */
    #[allow(dead_code)]
    pub fn debug_print(&self) {
        let functions: String = self
            .functions
            .iter()
            .map(|func| {
                format!(
                    "fn {}({}); is {} \n",
                    func.0,
                    func.1.arguments.join(", "),
                    func.1.expr_id
                )
            })
            .collect();
        let variables: String = self
            .variables
            .iter()
            .map(|var| {
                format!(
                    "let {} = {};  in {} \n",
                    var.0,
                    value_to_string(BoxedValue {
                        value: var.1.value.clone(),
                        interface: var.1.val_type.clone()
                    })
                    .unwrap(),
                    var.1.expr_id
                )
            })
            .collect();
        println!(
            "DEBUG:: \n | functions | \n{} \n | variables | \n{}",
            functions, variables
        );
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

    pub fn push_variable(&mut self, var: VariableDef) {
        self.variables.insert(var.var_id.clone(), var.clone());
    }

    /*
     * Get a variable in the stack by its name
     *
     */
    pub fn get_variable_by_name(&self, var_name: &str) -> Option<VariableDef> {
        for (_, variable) in &self.variables {
            if variable.name == var_name.to_string() {
                return Some(variable.clone());
            }
        }
        return None;
    }

    /*
     * Get a mutable variable in the stack by its name
     *
     */
    pub fn get_mut_variable_by_name(&mut self, var_name: &str) -> Option<&mut VariableDef> {
        for (_, variable) in &mut self.variables {
            if variable.name == var_name.to_string() {
                return Some(variable);
            }
        }
        return None;
    }

    /*
     * Modify a variable value
     */
    pub fn modify_var(
        &mut self,
        var_name: String,
        value: Box<dyn primitive_values::PrimitiveValueBase>,
    ) {
        let mut_var = self.get_mut_variable_by_name(var_name.as_str());

        if mut_var.is_some() {
            let mut_var = mut_var.unwrap();
            mut_var.value = value;
        } else {
            errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
        }
    }
}
