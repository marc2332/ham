use crate::{
    ast_types::{
        ast_base::AstBase,
        boxed_val::BoxedValue,
    },
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* VARIABLE DEFINITION */
pub trait VarDefinitionBase {
    fn get_def_name(&self) -> String;
    fn new(def_name: String, assignment: BoxedValue) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct VarDefinition {
    pub def_name: String,
    pub assignment: BoxedValue,
}

impl VarDefinitionBase for VarDefinition {
    fn get_def_name(&self) -> String {
        self.def_name.clone()
    }
    fn new(def_name: String, assignment: BoxedValue) -> Self {
        Self {
            def_name,
            assignment,
        }
    }
}

impl AstBase for VarDefinition {
    fn get_type(&self) -> Ops {
        Ops::VarDef
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}
