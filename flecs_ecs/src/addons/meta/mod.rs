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

use core::ffi::c_void;

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
        value: i32,
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
