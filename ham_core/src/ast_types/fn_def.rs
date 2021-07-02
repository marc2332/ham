use crate::{
    ast_types::ast_base::AstBase,
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* FUNCTION DEFINITION */
pub trait FnDefinitionBase {
    fn get_def_name(&self) -> String;
    fn new(def_name: String, body: Vec<Box<dyn self::AstBase>>, arguments: Vec<String>) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct FnDefinition {
    pub def_name: String,
    pub body: Vec<Box<dyn self::AstBase>>,
    pub arguments: Vec<String>,
}

impl FnDefinitionBase for FnDefinition {
    fn get_def_name(&self) -> String {
        self.def_name.clone()
    }
    fn new(def_name: String, body: Vec<Box<dyn self::AstBase>>, arguments: Vec<String>) -> Self {
        Self {
            def_name,
            body,
            arguments,
        }
    }
}

impl AstBase for FnDefinition {
    fn get_type(&self) -> Ops {
        Ops::FnDef
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}
