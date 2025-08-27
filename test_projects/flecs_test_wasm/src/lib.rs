use flecs_ecs::prelude::*;

#[derive(Debug, Component, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[no_mangle]
pub extern "C" fn example_pos_x() -> i32 {
    let world = World::new();

    world.component::<Position>();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let pos = e.cloned::<&Position>();
    pos.x
}
