//! The [`World`] type: the container for all entities, components and queries.
//!
//! Start with [`World::new()`] (or [`World::new_mini()`] for a world without any builtin
//! modules registered) to create a world, then use it to spawn entities, register components
//! and build queries.

use crate::core::*;
use crate::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub(crate) type FlecsArray = Vec<u64>;

mod component;
mod entity_view;
mod event;
mod id;
mod observer;
mod operations;
#[cfg(feature = "flecs_pipeline")]
mod pipeline;
mod query;
mod singleton;
#[cfg(feature = "flecs_system")]
mod system;
mod world;

pub use singleton::*;
pub use world::*;
