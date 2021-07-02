use crate::{
    ast_types::ast_base::AstBase,
    primitive_values::primitive_base::PrimitiveValueBase,
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* BOXED VALUE */

#[derive(Clone, Debug, Serialize)]
pub struct BoxedValue {
    pub interface: Ops,
    pub value: Box<dyn PrimitiveValueBase>,
}

impl AstBase for BoxedValue {
    fn get_type(&self) -> Ops {
        Ops::LeftAssign
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

impl PrimitiveValueBase for BoxedValue {
    fn as_self(&self) -> &dyn Any {
        self
    }
}
