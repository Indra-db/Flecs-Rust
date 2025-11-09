//! The alerts module enables applications to register alerts for when certain
//! conditions are met. Alerts are registered as [queries](crate::core::Query),
//! and automatically become active when [entities](crate::core::EntityView) match the alert query.
//!
//! Alerts are useful for monitoring application state, validating entity configurations,
//! and detecting problematic conditions at runtime. Each alert can have a severity level
//! and custom messages.
//!
//! # Example
//!
//! ```no_run
//! use flecs_ecs::prelude::*;
//!
//! #[derive(Component)]
//! struct Health(f32);
//!
//! let world = World::new();
//!
//! // Create an alert builder for entities with low health
//! let alert = world.alert::<&Health>()
//!     .message("Low health detected")
//!     .build();
//! ```
//!
//! # See also
//!
//! - [`AlertBuilder`] - Builder for creating alerts
//! - [`World::alert`](crate::core::World::alert) - Create a new alert

mod alert_builder;
pub use alert_builder::*;
mod module;
pub use module::*;
mod types;
pub use types::*;
mod alerts;
pub use alerts::*;
mod entity_view;
mod world;
