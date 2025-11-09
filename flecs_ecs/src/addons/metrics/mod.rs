//! The metrics module extracts metrics from components and makes them available
//! through a unified component interface.
//!
//! Metrics allow you to track and monitor specific values from your ECS components,
//! making them accessible for debugging, profiling, and real-time monitoring tools.
//! The module supports different metric types like counters, gauges, and rates.
//!
//! # Metric Types
//!
//! - [`Counter`] - Monotonically increasing value
//! - [`CounterIncrement`] - Auto-incremented counter by source value
//! - [`CounterId`] - Counts the number of entities with an id
//! - [`Gauge`] - Represents current value (can increase or decrease)
//!
//! # Example
//!
//! ```no_run
//! use flecs_ecs::prelude::*;
//! use flecs_ecs::addons::metrics::*;
//!
//! #[derive(Component)]
//! struct Health {
//!     current: f32,
//!     max: f32,
//! }
//!
//! let world = World::new();
//!
//! // Create a metric to track the health value
//! let entity = world.entity();
//! world.metric(entity)
//!     .member_named("current")
//!     .kind(Gauge);
//! ```
//!
//! # See also
//!
//! - [`MetricBuilder`] - Builder for creating metrics
//! - [`World::metric`](crate::core::World::metric) - Create a new metric
//! - [`UntypedComponent::metric`](crate::core::UntypedComponent::metric) - Register a component member as a metric
//! - [Flecs Metrics Manual](https://www.flecs.dev/flecs/md_docs_2Metrics.html)

mod module;
pub use module::*;
mod metric_builder;
pub use metric_builder::*;
mod types;
pub use types::*;

mod untyped_component;
mod world;
