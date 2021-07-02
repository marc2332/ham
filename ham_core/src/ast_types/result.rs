use crate::{
    ast_types::{
        ast_base::AstBase,
        boxed_val::BoxedValue,
    },
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/* RESULT EXPRESSION */
pub trait ResultExpressionBase {
    fn new(relation: Ops, left: BoxedValue, right: BoxedValue) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct ResultExpression {
    pub left: BoxedValue,
    pub relation: Ops,
    pub right: BoxedValue,
}

impl ResultExpressionBase for ResultExpression {
    fn new(relation: Ops, left: BoxedValue, right: BoxedValue) -> Self {
        Self {
            left,
            relation,
            right,
        }
    }
}

impl AstBase for ResultExpression {
    fn get_type(&self) -> Ops {
        Ops::ResExpression
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}
