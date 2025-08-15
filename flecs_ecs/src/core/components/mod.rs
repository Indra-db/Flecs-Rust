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
