use crate::{
    ast_types::ast_base::AstBase,
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;
use uuid::Uuid;

/* EXPRESSION  */

#[derive(Clone, Debug, Serialize)]
pub struct Expression {
    pub body: Vec<Box<dyn self::AstBase>>,
    pub token_type: Ops,
    pub expr_id: String,
}

impl AstBase for Expression {
    fn get_type(&self) -> Ops {
        Ops::Expression
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait ExpressionBase {
    fn new() -> Self;
    fn from_body(body: Vec<Box<dyn self::AstBase>>) -> Self;
}

impl ExpressionBase for Expression {
    fn new() -> Self {
        Self {
            token_type: Ops::Expression,
            body: Vec::new(),
            expr_id: Uuid::new_v4().to_string(),
        }
    }
    /*
     * Create an expression statement from an existing AST body
     */
    fn from_body(body: Vec<Box<dyn self::AstBase>>) -> Self {
        Self {
            token_type: Ops::Expression,
            body,
            // TODO: Move away from Uuid
            expr_id: Uuid::new_v4().to_string(),
        }
    }
}
