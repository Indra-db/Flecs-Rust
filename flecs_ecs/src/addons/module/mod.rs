//! The module addon allows for creating and importing modules.
//!
//! Flecs modules enable applications to organize components and systems into reusable
//! units of code that can be easily shared across projects. Modules provide automatic
//! namespacing, dependency management, and prevent identifier conflicts.
//!
//! # Key Features
//!
//! - **Organization**: Group related components, systems, and resources together
//! - **Reusability**: Share modules across different projects
//! - **Namespacing**: Module contents are automatically scoped to prevent conflicts
//! - **Lazy Loading**: Modules are only initialized once, even if imported multiple times
//!
//! # Usage
//!
//! To create a module:
//! 1. Define a struct that implements [`Module`]
//! 2. Implement the `module()` method to register components, systems, etc.
//! 3. Import the module using [`World::import()`](crate::core::World::import)
//!
//! # Example
//!
//! ```
//! use flecs_ecs::prelude::*;
//!
//! // Define components for the physics module
//! #[derive(Component)]
//! struct Velocity { x: f32, y: f32 }
//!
//! #[derive(Component)]
//! struct Position { x: f32, y: f32 }
//!
//! // Define the physics module
//! #[derive(Component)]
//! struct PhysicsModule;
//!
//! impl Module for PhysicsModule {
//!     fn module(world: &World) {
//!         // Name the module (optional but recommended)
//!         world.module::<PhysicsModule>("physics");
//!
//!         // Register components
//!         world.component::<Velocity>();
//!         world.component::<Position>();
//!
//!         // Create a movement system
//!         world.system::<(&Velocity, &mut Position)>()
//!             .each(|(vel, pos)| {
//!                 pos.x += vel.x;
//!                 pos.y += vel.y;
//!             });
//!     }
//! }
//!
//! // Use the module
//! let world = World::new();
//! world.import::<PhysicsModule>();
//!
//! // Create an entity using components from the module
//! world.entity()
//!     .set(Velocity { x: 1.0, y: 2.0 })
//!     .set(Position { x: 0.0, y: 0.0 });
//! ```
//!
//! # See also
//!
//! - [`Module`] - Trait for defining modules
//! - [`World::import()`](crate::core::World::import) - Import a module into the world
//! - [`World::module()`](crate::core::World::module) - Override the name of a module
//! - [Flecs Modules Manual](https://www.flecs.dev/flecs/md_docs_2Modules.html)

mod module;
mod world;

pub use module::*;
