//! The meta addon enables reflecting on component data.
//!
//! Types are stored as entities with components that store reflection data. This allows
//! for runtime introspection, serialization, and manipulation of component data without
//! compile-time knowledge of the types.
//!
//! # Type Representation
//!
//! A type entity has at least two components:
//! - **`EcsComponent`**: Core component containing size & alignment
//! - **`EcsType`**: Indicates what kind of type the entity represents
//!
//! Additionally, types may have reflection-specific components:
//!
//! ## Structs
//! - `EcsComponent`
//! - `EcsType`
//! - `EcsStruct`
//!
//! Struct members are represented as child entities with the `EcsMember` component.
//! Adding a member child automatically adds `EcsStruct` to the parent.
//!
//! ## Enums/Bitmasks
//! - `EcsComponent`
//! - `EcsType`  
//! - `EcsEnum` or `EcsBitmask`
//!
//! Constants are child entities with the `Constant` tag. Values are auto-assigned
//! by default, or can be manually set using `(Constant, i32)` for enums or
//! `(Constant, u32)` for bitmasks.
//!
//! # Usage
//!
//! The easiest way to add reflection data is using the `#[flecs(meta)]` attribute
//! with the `Component` derive macro:
//!
//! ```
//! use flecs_ecs::prelude::*;
//!
//! #[derive(Component)]
//! #[flecs(meta)]
//! struct Position {
//!     x: f32,
//!     y: f32,
//! }
//!
//! let world = World::new();
//! world.component::<Position>();
//!
//! // The Position type now has full reflection metadata
//! ```
//!
//! Types created with reflection metadata automatically receive `EcsComponent` and
//! `EcsType`, allowing them to be used as regular components:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # #[flecs(meta)]
//! # struct Position { x: f32, y: f32 }
//! # let world = World::new();
//! // Create entity with Position component
//! let entity = world.entity().set(Position { x: 10.0, y: 20.0 });
//! ```
//!
//! # Examples
//!
//! For comprehensive examples of the meta addon, see the [`examples/flecs/reflection/`]
//! directory, which includes:
//! - `reflection_basics.rs` - Basic reflection usage
//! - `reflection_nested_struct.rs` - Nested struct reflection
//! - `member_ranges.rs` - Member validation and ranges
//! - `reflection_runtime_component.rs` - Creating types at runtime
//!
//! [`examples/flecs/reflection/`]: https://github.com/Indra-db/Flecs-Rust/tree/main/flecs_ecs/examples/flecs/flecs/reflection
//!
//! # See also
//!
//! - [`Component` derive macro](crate::macros::Component) with `#[flecs(meta)]` attribute

#![doc(hidden)]
mod builtin;
mod component;
mod component_id_fetcher;
mod cursor;
mod declarations;
mod ecs_serializer;
mod entity_view;
mod impl_bindings;
mod impl_primitives;
pub mod macros;
mod meta_fn_types;
mod meta_functions;
mod meta_traits;
mod opaque;
mod untyped_component;
mod world;

pub use builtin::*;
pub use component_id_fetcher::*;
pub use cursor::*;
pub use declarations::*;
pub use ecs_serializer::*;
pub use macros::*;
pub use meta_fn_types::*;
pub use meta_traits::MetaMember;
pub use opaque::*;

use crate::sys;

//used for `.member` functions
pub struct Count(pub i32);

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(Debug, Clone, Component)]
    struct Int {
        _value: i32,
    }

    // //#[test]
    // fn test_opaque() {
    //     let world = World::new();
    //     world
    //         .component::<Int>()
    //         .opaque::<flecs::meta::I32>()
    //         .serialize(|s: &meta::Serializer, i: &Int| s.value::<i32>(&i.value));

    //     let int_type = Int { value: 10 };

    //     let json = world.to_json::<Int>(&int_type);

    //     println!("{}", json);
    //     assert_eq!("10", json);
    // }

    // #[derive(Component, Default)]
    // struct Position {
    //     x: f32,
    //     y: f32,
    // }

    // //#[test]
    // fn test_expr() {
    //     let world = World::new();

    //     world
    //         .component::<Position>()
    //         .member::<f32>("x", 1, core::mem::offset_of!(Position, x) as i32)
    //         .member::<f32>("y", 1, core::mem::offset_of!(Position, y) as i32);

    //     let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    //     let pos_id = <Position as ComponentId>::id(&world);

    //     // e.get::<&Position>(|pos| {
    //     //     let expr = world.to_expr(pos);
    //     //     println!("{}", expr);
    //     // });
    // }
}
