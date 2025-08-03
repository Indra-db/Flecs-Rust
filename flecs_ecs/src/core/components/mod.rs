//! Contains types that represents components and a fast mechanism for caching access to them from an entity.

mod cached_ref;
mod component;
mod component_binding;
mod component_untyped;
#[doc(hidden)]
pub mod lifecycle_traits;

pub use cached_ref::*;
pub use component::*;
pub(crate) use component_binding::*;
pub use component_untyped::*;
#[doc(hidden)]
pub use lifecycle_traits::*;

#[cfg(feature = "flecs_safety_readwrite_locks")]
#[derive(Clone, Copy)]
#[doc(hidden)]
pub enum ComponentTypeRWLock {
    Dense((i16, super::ReadWriteId)),
    Sparse(
        (
            *mut flecs_ecs::sys::ecs_component_record_t,
            super::ReadWriteId,
        ),
    ),
}
