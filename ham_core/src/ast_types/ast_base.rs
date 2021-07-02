use crate::utils::Ops;
use erased_serde::serialize_trait_object;
use std::any::Any;

/*
 * This is the base for every AST type
 */
pub trait AstBase: dyn_clone::DynClone + erased_serde::Serialize + std::fmt::Debug {
    fn get_type(&self) -> Ops;
    fn as_self(&self) -> &dyn Any;
}

dyn_clone::clone_trait_object!(AstBase);
serialize_trait_object!(AstBase);
