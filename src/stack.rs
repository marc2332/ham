use crate::ast::ast_operations;
use crate::ast::ast_operations::{AstBase, BoxedValue};
use crate::runtime::values_to_strings;
use crate::utils::{op_codes, primitive_values};
use std::sync::{Mutex, MutexGuard};

/*
 * Variable definition stored on the memory stack
 */
#[derive(Clone)]
pub struct VariableDef {
    pub name: String,
    pub val_type: op_codes::Val,
    pub value: Box<dyn primitive_values::PrimitiveValueBase>,
    pub expr_id: String,
    pub methods: Vec<FunctionDef>,
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
    ) -> Result<ast_operations::BoxedValue, ()>,
    pub expr_id: String,
    pub arguments: Vec<String>,
}

/*
 * Implementation of the stack
 */
#[derive(Clone)]
pub struct Stack {
    pub functions: Box<[FunctionDef]>,
    pub variables: Vec<VariableDef>,
}

impl Stack {
    pub fn new(expr_id: String) -> Stack {
        let mut functions = Vec::new();

        /*
         * print() function
         */
        functions.push(FunctionDef {
            name: String::from("print"),
            body: vec![],
            arguments: vec![],
            cb: |_, args, _, _, _| {
                print!("{}", values_to_strings(args).join(" "));
                return Err(());
            },
            expr_id: expr_id.clone(),
        });

        /*
         * println() function
         */
        functions.push(FunctionDef {
            name: String::from("println"),
            body: vec![],
            arguments: vec![],
            cb: |_, args, _, _, _| {
                println!("{}", values_to_strings(args).join(""));
                return Err(());
            },
            expr_id: expr_id.clone(),
        });

        Stack {
            variables: Vec::new(),
            functions: functions.into_boxed_slice(),
        }
    }
    pub fn drop_ops_from_id(&mut self, id: String) {
        self.variables.retain(|var| var.expr_id != id);
    }

    pub fn get_functions(self) -> Vec<FunctionDef> {
        self.functions.to_vec().clone()
    }

    pub fn push_function(&mut self, function: FunctionDef) {
        let mut functions = self.functions.to_vec();

        functions.push(function);

        self.functions = functions.into_boxed_slice();
    }
}
