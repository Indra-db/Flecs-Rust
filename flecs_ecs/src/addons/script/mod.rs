//! The script addon enables loading and executing Flecs scripts.
//!
//! Flecs script is a declarative language for defining entities, components, systems,
//! and other ECS constructs. Scripts provide a convenient way to initialize worlds,
//! configure entities, and set up game data without recompiling.
//!
//! # Features
//!
//! - **Declarative Syntax**: Define entities and components using a simple text format
//! - **Runtime Loading**: Load and execute scripts at runtime
//! - **Template Support**: Use templates and prefabs in scripts
//! - **Expression Evaluation**: Evaluate expressions within scripts
//!
//! # Example
//!
//! ```no_run
//! use flecs_ecs::prelude::*;
//!
//! let world = World::new();
//!
//! // Load a Flecs script
//! world.script()
//!     .build_from_code(
//!         r#"
//!         Position {
//!             x: f32,
//!             y: f32
//!         }
//!
//!         entity {
//!             Position: {10, 20}
//!         }
//!         "#
//!     );
//! ```
//!
//! For more comprehensive script examples, see the [`examples/flecs/script/`] directory
//! in the repository, which includes:
//! - `hello_world.flecs` - Basic script example
//! - `prefabs.flecs` - Using prefabs in scripts
//! - `reflection.flecs` - Component reflection examples
//! - `expressions.flecs` - Expression evaluation examples
//!
//! [`examples/flecs/script/`]: https://github.com/Indra-db/Flecs-Rust/tree/main/flecs_ecs/examples/flecs/flecs/script
//!
//! # See also
//!
//! - [`World::script()`](crate::core::World::script) - Execute a Flecs script
//! - [Flecs Script Manual](https://www.flecs.dev/flecs/md_docs_2FlecsScript.html)
//! - [Flecs Script Tutorial](https://www.flecs.dev/flecs/flecsscripttutorial.html)

mod script_builder;
mod script_entity_view;
mod unmanaged_script;
mod world;

pub use script_builder::*;
pub use script_entity_view::*;
pub use unmanaged_script::*;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
