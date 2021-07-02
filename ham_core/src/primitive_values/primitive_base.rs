use erased_serde::serialize_trait_object;
use std::any::Any;

/*
 * Primitive value base
 */
pub trait PrimitiveValueBase:
    dyn_clone::DynClone + erased_serde::Serialize + std::fmt::Debug
{
    fn as_self(&self) -> &dyn Any;
}

dyn_clone::clone_trait_object!(PrimitiveValueBase);
serialize_trait_object!(PrimitiveValueBase);
