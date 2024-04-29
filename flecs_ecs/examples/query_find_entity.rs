include!("common");

#[allow(dead_code)]
pub fn main() -> Result<World, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();
    // Create a few test entities for a Position query
    world.entity_named(c"e1").set(Position { x: 10.0, y: 20.0 });

    world.entity_named(c"e2").set(Position { x: 20.0, y: 30.0 });

    // Create a simple query for component Position
    let query = world.new_query::<&Position>();

    let entity: Option<EntityView> = query.find(|pos| (pos.x - 20.0).abs() < f32::EPSILON);

    if let Some(entity) = entity {
        fprintln!(&world, "Entity found: {:?}", entity.path().unwrap());
    } else {
        fprintln!(&world, "Entity not found");
    }

    drop(query);

    Ok(world)

    // Output:
    //  Entity found: "::e2"
}
