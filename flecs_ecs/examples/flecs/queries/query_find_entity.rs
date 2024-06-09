use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

fn main() {
    let world = World::new();
    // Create a few test entities for a Position query
    world.entity_named("e1").set(Position { x: 10.0, y: 20.0 });

    world.entity_named("e2").set(Position { x: 20.0, y: 30.0 });

    // Create a simple query for component Position
    let query = world.new_query::<&Position>();

    let entity: Option<EntityView> = query.find(|pos| (pos.x - 20.0).abs() < f32::EPSILON);

    if let Some(entity) = entity {
        println!("Entity found: {:?}", entity.path().unwrap());
    } else {
        println!("Entity not found");
    }

    // Output:
    //  Entity found: "::e2"
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_find_entity".to_string());
}
