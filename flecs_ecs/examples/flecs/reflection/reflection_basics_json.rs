use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
#[meta]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

fn main() {
    let mut world = World::new();

    // Register the Position component with reflection data
    world.component::<Position>().meta();

    /* Alternatively, you can do it manually like so (without the derive macro)
    .member::<f32>("x", 1 /* count */, core::mem::offset_of!(Position, x))
    .member::<f32>("y", 1, core::mem::offset_of!(Position, y));
    */

    // Create a new entity with the Position component
    let e = world.entity().set(Position { x: 2.0, y: 4.0 });

    // Convert position component to JSON string
    e.get::<&Position>(|p| {
        let expr: String = world.to_json::<Position>(p);
        println!("Position: {}", expr);
    });

    // Output:
    //  Position: {x: 2, y: 4}

    // Convert entity to JSON string
    println!("Entity: {}", e.to_json(None));

    // Output:
    // Entity: {"name":"#547", "components":{"Position":{"x":2, "y":4}}}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_basics_json".to_string());
}
