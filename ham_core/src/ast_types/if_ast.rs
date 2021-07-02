use crate::{
    ast_types::{
        ast_base::AstBase,
        result::ResultExpression,
    },
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* IF STATEMENT */
pub trait IfConditionalBase {
    fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct IfConditional {
    pub conditions: Vec<ResultExpression>,
    pub body: Vec<Box<dyn self::AstBase>>,
}

impl IfConditionalBase for IfConditional {
    fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self {
        Self { conditions, body }
    }
}

impl AstBase for IfConditional {
    fn get_type(&self) -> Ops {
        Ops::IfConditional
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}
