use crate::{
    ast_types::{
        ast_base::AstBase,
        fn_def::FnDefinition,
    },
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* MODULE STATEMENT */

#[derive(Clone, Debug, Serialize)]
pub struct Module {
    pub name: String,
    pub functions: Vec<FnDefinition>,
}

impl AstBase for Module {
    fn get_type(&self) -> Ops {
        Ops::Module
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}
