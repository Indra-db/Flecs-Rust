use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

fn main() {
    let mut world = World::new();

    // Register the Position component
    world
        .component::<Position>()
        .member::<f32>("x", 1 /* count */, offset_of!(Position, x))
        .member::<f32>("y", 1, offset_of!(Position, y));

    // Create a new entity
    let e = world.entity().set(Position { x: 2.0, y: 4.0 });

    // Convert position component to flecs expression string
    e.get::<&Position>(|p| {
        let expr: String = world.to_expr(p);
        println!("Position: {}", expr);
    });

    // Output:
    //  Position: {x: 2, y: 4}
}
