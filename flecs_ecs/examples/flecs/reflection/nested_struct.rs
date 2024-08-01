use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Default, Component)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Component)]
pub struct Line {
    pub start: Point,
    pub stop: Point,
}

#[test]
fn main() {
    let world = World::new();

    world
        .component::<Point>()
        .member::<f32>("x", 1, offset_of!(Point, x))
        .member::<f32>("y", 1, offset_of!(Point, y));

    world
        .component::<Line>()
        .member::<Point>("start", 1, offset_of!(Line, start))
        .member::<Point>("stop", 1, offset_of!(Line, stop));

    // Create entity, set Line component
    let e = world.entity().set(Line {
        start: Point { x: 10.0, y: 20.0 },
        stop: Point { x: 30.0, y: 40.0 },
    });

    // Convert Line component to flecs expression string
    e.get::<&mut Line>(|line| {
        // Convert component to string
        println!("{}", world.to_expr(line));
    });

    // Output:
    //  {start: {x: 10, y: 20}, stop: {x: 30, y: 40}}
}
