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

    // Create a query for Position, Velocity. Queries are the fastest way to
    // iterate entities as they cache results.
    let query = world.new_query::<(&mut Position, &Velocity)>();

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

    // The next lines show the different ways in which a query can be iterated.

    // `The each_entity()` function iterates each entity individually and accepts an
    // entity argument plus arguments for each query component:
    query.each_entity(|e, (pos, vel)| {
        pos.x += vel.x;
        pos.y += vel.y;
        println!("{}: [{:?}]", e.name(), pos);
    });

    // There's an equivalent function that does not include the entity argument
    query.each(|(pos, vel)| {
        pos.x += vel.x;
        pos.y += vel.y;
        println!("[{:?}]", pos);
    });

    // Iter is a bit more verbose, but allows for more control over how entities
    // are iterated as it provides multiple entities in the same callback.
    // There's also an `iter_only` function that only provides the iterator.
    query.iter(|it, (pos, vel)| {
        for i in it.iter() {
            pos[i].x += vel[i].x;
            pos[i].y += vel[i].y;
            println!("[{:?}]", pos[i]);
        }
    });

    // Output:
    //  e1: [Position { x: 11.0, y: 22.0 }]
    //  e2: [Position { x: 13.0, y: 24.0 }]
    //  [Position { x: 12.0, y: 24.0 }]
    //  [Position { x: 16.0, y: 28.0 }]
    //  [Position { x: 13.0, y: 26.0 }]
    //  [Position { x: 19.0, y: 32.0 }]
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_basics".to_string());
}
