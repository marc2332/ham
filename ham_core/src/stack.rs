use crate::ast::ast_operations;
use crate::ast::ast_operations::{AstBase, BoxedValue};
use crate::runtime::{downcast_val, value_to_string, values_to_strings};
use crate::utils::primitive_values::StringVal;
use crate::utils::{errors, primitive_values, Ops};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::{thread, time};

/*
 * Variable definition stored on the memory stack
 */
#[derive(Clone)]
pub struct VariableDef {
    pub name: String,
    pub val_type: Ops,
    pub value: Box<dyn primitive_values::PrimitiveValueBase>,
    pub expr_id: String,
    pub functions: HashMap<String, FunctionDef>,
    pub var_id: u64,
}

impl FunctionsContainer for VariableDef {
    fn get_function_by_name(&self, fn_name: &str) -> Option<FunctionDef> {
        let op_fn: Option<&FunctionDef> = self.functions.get(fn_name);

        if let Some(op_fn) = op_fn {
            Some(op_fn.clone())
        } else {
            errors::raise_error(errors::CODES::FunctionNotFound, vec![fn_name.to_string()]);
            None
        }
    }
    fn push_function(&mut self, function: FunctionDef) {
        self.functions.insert(function.name.clone(), function);
    }
}

type FunctionAction = fn(
    args: Vec<String>,
    args_vals: Vec<BoxedValue>,
    body: Vec<Box<dyn AstBase>>,
    stack: &Mutex<Stack>,
    ast: &MutexGuard<ast_operations::Expression>,
) -> Option<ast_operations::BoxedValue>;

/*
 * Function definition stored on the memory stack
 */
#[derive(Clone)]
pub struct FunctionDef {
    pub name: String,
    pub body: Vec<Box<dyn AstBase>>,
    pub cb: FunctionAction,
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
    pub variables: Vec<VariableDef>,
    pub item_index: u64,
}

impl FunctionsContainer for Stack {
    fn get_function_by_name(&self, fn_name: &str) -> Option<FunctionDef> {
        for function in self.functions.values() {
            if function.name == *fn_name {
                return Some(function.clone());
            }
        }
        errors::raise_error(errors::CODES::FunctionNotFound, vec![fn_name.to_string()]);
        None
    }
    fn push_function(&mut self, function: FunctionDef) {
        self.functions.insert(function.expr_id.clone(), function);
    }
}

impl Stack {
    pub fn reseve_index(&mut self) -> u64 {
        self.item_index += 1;
        self.item_index
    }

    pub fn new(expr_id: String) -> Stack {
        let mut functions = HashMap::new();

        functions.insert(
            "clear".to_string(),
            FunctionDef {
                name: "clear".to_string(),
                body: vec![],
                arguments: vec![],
                cb: |_, _, _, _, _| {
                    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                    None
                },
                expr_id: expr_id.clone(),
            },
        );

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
                cb: |_, args, _, stack, _| {
                    let mut args = values_to_strings(args, stack);

                    let mut template = args[0].clone();

                    args.remove(0);

                    for arg in args {
                        template = template.replacen("{}", arg.as_str(), 1);
                    }

                    Some(BoxedValue {
                        interface: Ops::String,
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
                cb: |_, args, _, stack, _| {
                    print!("{}", values_to_strings(args, stack).join(" "));
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
                cb: |_, args, _, stack, _| {
                    println!("{}", values_to_strings(args, stack).join(""));
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
                cb: |_, args, _, stack, _| {
                    let time = value_to_string(args[0].clone(), stack)
                        .unwrap()
                        .parse::<u64>()
                        .unwrap();
                    let time = time::Duration::from_millis(time);
                    thread::sleep(time);
                    None
                },
                expr_id,
            },
        );

        Stack {
            variables: Vec::new(),
            functions,
            item_index: 0,
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
                    var.name,
                    value_to_string(
                        BoxedValue {
                            value: var.value.clone(),
                            interface: var.val_type
                        },
                        &Mutex::new(self.clone())
                    )
                    .unwrap(),
                    var.expr_id
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
        self.variables.retain(|var| var.expr_id != id);
        self.functions.retain(|_, func| func.expr_id != id);
    }

    pub fn push_variable(&mut self, var: VariableDef) {
        self.variables.push(var);
    }

    /*
     * Get a variable from the stack by its ID
     */
    pub fn get_variable_by_id(&self, var_id: u64) -> Option<VariableDef> {
        for variable in &self.variables {
            if variable.var_id == var_id {
                return Some(variable.clone());
            }
        }
        None
    }

    /*
     * Get a variable from the stack by its name
     */
    pub fn get_variable_by_name(&self, var_name: &str) -> Option<VariableDef> {
        for variable in self.variables.iter().rev() {
            if variable.name == *var_name {
                return Some(variable.clone());
            }
        }
        None
    }

    /*
     * Get a mutable variable from the stack by its ID
     */
    pub fn get_mut_variable_by_id(&mut self, var_id: u64) -> Option<&mut VariableDef> {
        for variable in &mut self.variables {
            if variable.var_id == var_id {
                return Some(variable);
            }
        }
        None
    }

    /*
     * Get a mutable variable from the stack by its name
     */
    pub fn get_mut_variable_by_name(&mut self, var_name: &str) -> Option<&mut VariableDef> {
        for variable in &mut self.variables.iter_mut().rev() {
            if variable.name == *var_name {
                return Some(variable);
            }
        }
        None
    }

    /*
     * Modify a variable value
     */
    pub fn modify_var(&mut self, var_name: String, value: BoxedValue) {
        let variable = self.get_mut_variable_by_name(var_name.as_str());

        // If variable exists
        if let Some(variable) = variable {
            // Is pointer
            if variable.val_type == Ops::Pointer {
                let variable = variable.clone();

                let pointer = downcast_val::<primitive_values::Pointer>(variable.value.as_self());

                let variable_origin = self.get_mut_variable_by_id(pointer.0);

                if let Some(variable_origin) = variable_origin {
                    variable_origin.value = value.value;
                    variable_origin.val_type = value.interface;
                } else {
                    // Broken pointer
                }
            } else {
                variable.value = value.value;
            }
        } else {
            errors::raise_error(errors::CODES::VariableNotFound, vec![var_name.clone()]);
        }
    }
}
