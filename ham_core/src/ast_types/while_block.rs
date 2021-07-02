use crate::{
    ast_types::{
        ast_base::AstBase,
        result::ResultExpression,
    },
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* WHILE BLOCK  */

#[derive(Clone, Debug, Serialize)]
pub struct While {
    pub body: Vec<Box<dyn self::AstBase>>,
    pub conditions: Vec<ResultExpression>,
}

impl AstBase for While {
    fn get_type(&self) -> Ops {
        Ops::WhileDef
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait WhileBase {
    fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self;
}

impl WhileBase for While {
    fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self {
        Self { body, conditions }
    }
}
