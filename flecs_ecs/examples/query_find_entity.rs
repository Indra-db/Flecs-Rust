#[allow(unused_variables)]
mod common;
use common::*;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create a few test entities for a Position query
    world
        .new_entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 });

    world
        .new_entity_named(c"e2")
        .set(Position { x: 20.0, y: 30.0 });

    // Create a simple query for component Position
    let query = world.new_query::<&Position>();

    let entity: Option<EntityView> = query.find(|pos| (pos.x - 20.0).abs() < f32::EPSILON);

    if let Some(entity) = entity {
        fprintln!(snap, "Entity found: {:?}", entity.path().unwrap());
    } else {
        fprintln!(snap, "Entity not found");
    }

    snap.test();

    // Output:
    //  Entity found: "::e2"
}
