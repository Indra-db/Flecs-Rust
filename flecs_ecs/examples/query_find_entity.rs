include!("common");

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create a few test entities for a Position query
    world.entity_named(c"e1").set(Position { x: 10.0, y: 20.0 });

    world.entity_named(c"e2").set(Position { x: 20.0, y: 30.0 });

    // Create a simple query for component Position
    let query = world.new_query::<&Position>();

    let entity: Option<EntityView> = query.find(|pos| (pos.x - 20.0).abs() < f32::EPSILON);

    if let Some(entity) = entity {
        fprintln!(snap, "Entity found: {:?}", entity.path().unwrap());
    } else {
        fprintln!(snap, "Entity not found");
    }

    Ok(snap)

    // Output:
    //  Entity found: "::e2"
}
