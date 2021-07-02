use crate::{
    ast_types::{
        ast_base::AstBase,
        boxed_val::BoxedValue,
    },
    primitive_values::primitive_base::PrimitiveValueBase,
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* FUNCTION CALL  */

#[derive(Clone, Serialize, Debug)]
pub struct FnCall {
    pub token_type: Ops,
    pub fn_name: String,
    pub arguments: Vec<BoxedValue>,
    pub reference_to: Option<String>,
}

impl AstBase for FnCall {
    fn get_type(&self) -> Ops {
        Ops::FnCall
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait FnCallBase {
    fn new(fn_name: String, reference_to: Option<String>) -> Self;
}

impl FnCallBase for FnCall {
    fn new(fn_name: String, reference_to: Option<String>) -> Self {
        Self {
            token_type: Ops::FnCall,
            fn_name,
            arguments: Vec::new(),
            reference_to,
        }
    }
}

impl PrimitiveValueBase for FnCall {
    fn as_self(&self) -> &dyn Any {
        self
    }
}
