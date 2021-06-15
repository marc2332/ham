use crate::ast::ast_operations;
use crate::ast::ast_operations::AstBase;
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
        args_vals: Vec<String>,
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
    pub functions: Vec<FunctionDef>,
    pub variables: Vec<VariableDef>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            functions: Vec::new(),
            variables: Vec::new(),
        }
    }
    pub fn drop_ops_from_id(&mut self, id: String) {
        self.variables.retain(|var| var.expr_id != id);
    }
}
