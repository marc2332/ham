use crate::primitive_values::primitive_base::PrimitiveValueBase;
use serde::Serialize;
use std::any::Any;

/*
 * Number
 */

#[derive(Clone, Debug, Serialize)]
pub struct Number(pub usize);

// Implement base methods for Number
impl PrimitiveValueBase for Number {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/*
 * Number base
 */
pub trait NumberValueBase {
    fn new(val: usize) -> Self;
    fn get_state(&self) -> usize;
}

impl NumberValueBase for Number {
    fn new(val: usize) -> Self {
        Number(val)
    }

    fn get_state(&self) -> usize {
        self.0
    }
}
