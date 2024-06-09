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

// Systems can be created with a custom run function that takes control over the
// entire iteration. By  a system is invoked once per matched table,
// which means the function can be called multiple times per frame. In some
// cases that's inconvenient, like when a system has things it needs to do only
// once per frame. For these use cases, the run callback can be used which is
// called once per frame per system.

fn main() {
    let world = World::new();

    let system = world
        .system::<(&mut Position, &Velocity)>()
        // Forward each result from the run callback to the each callback.
        .run_each_entity(
            |mut iter| {
                println!("Move begin");

                while iter.next_iter() {
                    iter.each();
                }

                println!("Move end");
            },
            |e, (pos, vel)| {
                pos.x += vel.x;
                pos.y += vel.y;
                println!("{}: {{ {}, {} }}", e.name(), pos.x, pos.y);
            },
        );

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
    system.run();

    // Output:
    //  Move begin
    //  e1: {11, 22}
    //  e2: {13, 24}
    //  Move end
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("system_custom_runner".to_string());
}
