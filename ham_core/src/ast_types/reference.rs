use crate::primitive_values::primitive_base::PrimitiveValueBase;
use serde::Serialize;
use std::any::Any;

/*
 * Reference by name
 *
 * This happens when a variable instead of having an static value,
 * it references to another variable by it's name
 *
 */

#[derive(Clone, Debug, Serialize)]
pub struct Reference(pub String);

// Implement base methods for REFERENCE
impl PrimitiveValueBase for Reference {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait ReferenceValueBase {
    fn new(val: String) -> Self;
}

impl ReferenceValueBase for Reference {
    fn new(val: String) -> Self {
        Self(val)
    }
}
