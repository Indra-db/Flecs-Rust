//! Contains flecs traits and pre-registered components.

use crate::core::{ComponentInfo, InternalComponentHooks, OnComponentRegistration, c_types::*};
use crate::sys;
use core::ops::Deref;

// Internal macros
mod macros;
pub(crate) use macros::*;

// Core components and traits
mod component_traits;
pub use component_traits::*;

mod builtin;
pub use builtin::*;

mod events;
pub use events::*;

// Flags
pub mod id_flags;
pub mod query_flags;
pub mod term_flags;

// Components
pub mod unit_component;

// Feature-gated addons
mod addons;
pub use addons::*;

pub trait FlecsTrait {}
