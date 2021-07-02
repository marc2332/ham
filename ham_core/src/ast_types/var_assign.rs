use crate::{
    ast_types::{
        ast_base::AstBase,
        boxed_val::BoxedValue,
    },
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* VARIABLE ASSIGNMENT */
pub trait VarAssignmentBase {
    fn get_def_name(&self) -> String;
    fn new(var_name: String, assignment: BoxedValue) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct VarAssignment {
    pub var_name: String,
    pub assignment: BoxedValue,
}

impl VarAssignmentBase for VarAssignment {
    fn get_def_name(&self) -> String {
        self.var_name.clone()
    }
    fn new(var_name: String, assignment: BoxedValue) -> Self {
        Self {
            var_name,
            assignment,
        }
    }
}

impl AstBase for VarAssignment {
    fn get_type(&self) -> Ops {
        Ops::VarAssign
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}
