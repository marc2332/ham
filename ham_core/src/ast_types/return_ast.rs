use crate::{
    ast_types::{
        ast_base::AstBase,
        boxed_val::BoxedValue,
    },
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* RETURN STATEMENT */

#[derive(Clone, Debug, Serialize)]
pub struct ReturnStatement {
    pub value: BoxedValue,
}

impl AstBase for ReturnStatement {
    fn get_type(&self) -> Ops {
        Ops::Return
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}
