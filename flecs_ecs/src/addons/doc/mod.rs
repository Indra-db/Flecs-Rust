//! The doc addon allows documenting entities with brief and detailed descriptions.
//!
//! This module enables adding human-readable documentation to entities (including
//! components, systems, and other ECS constructs) as components. Documentation can
//! be retrieved at runtime and used by tooling such as UIs, documentation frameworks,
//! or introspection tools.
//!
//! # Features
//!
//! - **Brief Descriptions**: Short, one-line summaries of entities
//! - **Detailed Descriptions**: Longer, comprehensive documentation
//! - **Human-Readable Names**: Friendly display names separate from entity IDs
//! - **External Links**: URLs to external documentation
//! - **Runtime Access**: Query documentation at runtime for tooling
//!
//! # Usage
//!
//! Documentation can be added to any entity using the [`Doc`] trait methods:
//!
//! ```
//! use flecs_ecs::prelude::*;
//! use flecs_ecs::addons::doc::Doc;
//!
//! #[derive(Component)]
//! struct Position { x: f32, y: f32 }
//!
//! let world = World::new();
//!
//! // Add documentation to a component
//! world.component::<Position>()
//!     .set_doc_name("Position")
//!     .set_doc_brief("2D position in world space")
//!     .set_doc_detail("Represents the X and Y coordinates of an entity in the game world.");
//!
//! // Add documentation to an entity
//! world.entity()
//!     .set_doc_name("Player")
//!     .set_doc_brief("The player character")
//!     .set_doc_detail("Main player entity controlled by user input.");
//!
//! // Retrieve documentation at runtime
//! let pos_component = world.component::<Position>();
//! if let Some(brief) = pos_component.doc_brief() {
//!     println!("Position component: {}", brief);
//! }
//! ```
//!
//! # See also
//!
//! - [`Doc`] - Trait providing documentation methods for entities

mod doc;
mod world;
pub use doc::*;
