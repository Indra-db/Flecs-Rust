use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Default, Component)]
pub struct PortableType {
    pub i32: i32,
    pub usize: usize,
}

#[test]
fn main() {
    let world = World::new();

    // Register component. Do not use core i32 or core usize for the
    // member type as this will resolve to a different integer type depending on
    // the platform, which can cause unexpected issues when type information is
    // shared between platforms.
    world
        .component::<PortableType>()
        .member::<flecs::meta::UPtr>("usize", 1, offset_of!(PortableType, usize))
        .member::<flecs::meta::I32>("i32", 1, offset_of!(PortableType, i32));

    let e = world.entity().set(PortableType { i32: 10, usize: 20 });

    e.get::<&mut PortableType>(|portable_type| {
        println!("{}", world.to_expr(portable_type));
    });

    // Output:
    //  {i32: 10, usize: 20}
}
