use crate::primitive_values::primitive_base::PrimitiveValueBase;
use serde::Serialize;
use std::any::Any;

/*
 * String
 */

#[derive(Clone, Debug, Serialize)]
pub struct StringVal(pub String);

// Implement base methods for String
impl PrimitiveValueBase for StringVal {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/*
 * String base
 */
pub trait StringValueBase {
    fn new(val: String) -> Self;
    fn get_state(&self) -> String;
}

impl StringValueBase for StringVal {
    fn new(val: String) -> Self {
        Self(val)
    }

    fn get_state(&self) -> String {
        self.0.clone()
    }
}
