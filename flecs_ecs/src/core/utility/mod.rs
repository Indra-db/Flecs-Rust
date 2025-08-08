//! contains traits that define what a component is and also the API's for [`Query`][super::Query], [`Observer`][super::Observer] and [`System`][crate::addons::system::System].
//! Also contains lower level utility functions on ECS IDs. This is mostly used internally by the library.

mod errors;
mod functions;
pub mod id;
pub(crate) mod id_map;
mod log;
pub mod traits;
pub mod types;

pub use errors::*;
pub use functions::*;
pub use id::id;
pub(crate) use id_map::*;
pub use log::*;

#[doc(hidden)]
pub use traits::*;
#[doc(hidden)]
pub use types::*;

use crate::sys;

/// Type alias for extern function pointers that adapts to target platform
#[cfg(target_family = "wasm")]
pub(crate) type ExternIterFn = unsafe extern "C" fn(*mut sys::ecs_iter_t);
#[cfg(not(target_family = "wasm"))]
pub(crate) type ExternIterFn = unsafe extern "C-unwind" fn(*mut sys::ecs_iter_t);

#[cfg(target_family = "wasm")]
pub(crate) type ExternIterNextFn = unsafe extern "C" fn(*mut sys::ecs_iter_t) -> bool;
#[cfg(not(target_family = "wasm"))]
pub(crate) type ExternIterNextFn = unsafe extern "C-unwind" fn(*mut sys::ecs_iter_t) -> bool;
