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
