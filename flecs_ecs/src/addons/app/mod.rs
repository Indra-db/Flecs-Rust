//! The app addon is a wrapper around the application's main loop.
//!
//! Its main purpose is to provide a hook to modules that need to take control of the
//! main loop, as is for example the case with native applications that use
//! emscripten with WebGL.
//!
//! The app addon provides a structured way to initialize, run, and manage your
//! application's lifecycle, including integration with platform-specific requirements
//! like browser-based environments.
//!
//! # Example
//!
//! ```no_run
//! use flecs_ecs::prelude::*;
//!
//! #[derive(Component)]
//! struct Position { x: f32, y: f32 }
//!
//! let world = World::new();
//!
//! // Create a system that runs each frame
//! world.system::<&mut Position>()
//!     .each(|pos| {
//!         pos.x += 1.0;
//!     });
//!
//! // Run the application main loop
//! world.app()
//!     .set_target_fps(60.0)
//!     .run();
//! ```
//!
//! # See also
//!
//! - [`App`] - Main application controller
//! - [`World::app`](crate::core::World::app) - Create a new app instance

mod app;
mod world;
pub use app::*;
