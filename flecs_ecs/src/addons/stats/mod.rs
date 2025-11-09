//! The stats addon tracks high-resolution statistics for the world, systems, and pipelines.
//!
//! This module provides detailed performance metrics and can be used in two ways:
//! 1. **As an API**: Applications directly call functions to obtain statistics
//! 2. **As a module**: Statistics are automatically tracked (required for explorer)
//!
//! # Features
//!
//! - **World Statistics**: Entity counts, component operations, memory usage
//! - **System Statistics**: Execution time, invocation count, entity processing
//! - **Pipeline Statistics**: Performance metrics for entire pipelines
//! - **Multi-tier Tracking**: Statistics tracked per frame, second, minute, hour, day, and week
//! - **High Resolution**: 60 datapoints per tier when imported as module
//!
//! # Usage
//!
//! When imported as a module, statistics are automatically tracked and made available
//! to tools like the Flecs Explorer:
//!
//! ```no_run
//! use flecs_ecs::prelude::*;
//!
//! let world = World::new();
//!
//! // Statistics are tracked automatically when using the app addon with stats enabled
//! world.app()
//!     .enable_stats(true)
//!     .run();
//! ```
//!
//! # Statistics Tiers
//!
//! When the addon is imported as a module, statistics are tracked across multiple time scales:
//! - **Frame**: Per-frame statistics (60 frames)
//! - **Second**: Per-second aggregation (60 seconds)
//! - **Minute**: Per-minute aggregation (60 minutes)
//! - **Hour**: Per-hour aggregation (24 hours)
//! - **Day**: Per-day aggregation (7 days)
//! - **Week**: Per-week aggregation (52 weeks)
//!
//! # See also
//!
//! - [`App::enable_stats()`](crate::addons::app::App::enable_stats) - Enable statistics tracking

mod stats;
pub use stats::*;
