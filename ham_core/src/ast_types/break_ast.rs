use crate::{
    ast_types::ast_base::AstBase,
    utils::Ops,
};
use serde::Serialize;
use std::any::Any;

/*
 * Break statement
 *
 * Used to break from while loops
 *
 */

#[derive(Clone, Debug, Serialize)]
pub struct Break();

impl AstBase for Break {
    fn get_type(&self) -> Ops {
        Ops::Break
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait BreakBase {
    fn new() -> Self;
}

impl BreakBase for Break {
    fn new() -> Self {
        Self()
    }
}
