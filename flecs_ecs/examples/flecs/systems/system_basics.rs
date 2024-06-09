use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

fn main() {
    let world = World::new();

    // Create a system for Position, Velocity. Systems are like queries (see
    // queries) with a function that can be ran or scheduled (see pipeline).

    let s = world
        .system::<(&mut Position, &Velocity)>()
        .each_entity(|e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
            println!("{}: {{ {}, {} }}", e.name(), p.x, p.y);
        });

    // Create a few test entities for a Position, Velocity query
    world
        .entity_named("e1")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .entity_named("e2")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    // This entity will not match as it does not have Position, Velocity
    world.entity_named("e3").set(Position { x: 10.0, y: 20.0 });

    // Run the system
    s.run();

    // Output:
    //  e1: { 11, 22 }
    //  e2: { 13, 24 }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("system_basics".to_string());
}
