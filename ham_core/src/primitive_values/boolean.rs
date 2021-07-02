use crate::primitive_values::primitive_base::PrimitiveValueBase;
use serde::Serialize;
use std::any::Any;

/*
 * Boolean
 */

#[derive(Clone, Debug, Serialize)]
pub struct Boolean(pub bool);

// Implement base methods for Boolean
impl PrimitiveValueBase for Boolean {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/*
 * Boolean base
 */
pub trait BooleanValueBase {
    fn new(val: bool) -> Self;
    fn get_state(&self) -> bool;
}

impl BooleanValueBase for Boolean {
    fn new(val: bool) -> Self {
        Self(val)
    }

    fn get_state(&self) -> bool {
        self.0
    }
}
