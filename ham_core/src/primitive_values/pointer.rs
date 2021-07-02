use crate::primitive_values::primitive_base::PrimitiveValueBase;
use serde::Serialize;
use std::any::Any;

#[derive(Clone, Debug, Serialize)]
pub struct Pointer(pub u64);

// Implement base methods for Pointer
impl PrimitiveValueBase for Pointer {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/*
 * Pointer base
 */
pub trait PointerBase {
    fn get_state(&self) -> u64;
}

impl PointerBase for Pointer {
    fn get_state(&self) -> u64 {
        self.0
    }
}
