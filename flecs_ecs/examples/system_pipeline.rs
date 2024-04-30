include!("common");

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create a system for moving an entity
    world
        .system::<(&mut Position, &Velocity)>()
        .kind::<flecs::pipeline::OnUpdate>()
        .each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    // Create a system for printing the entity position
    world
        .system::<&Position>()
        .kind::<flecs::pipeline::PostUpdate>()
        .each_entity(|e, p| {
            fprintln!(e, "{}: {{ {}, {} }}", e.name(), p.x, p.y);
        });

    // Create a few test entities for a Position, Velocity query
    world
        .entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    // Run the default pipeline. This will run all systems ordered by their
    // phase. Systems within the same phase are ran in declaration order. This
    // function is usually called in a loop.
    world.progress();

    Ok(Snap::from(&world))

    // Output:
    //  e1: { 11, 22 }
    //  e2: { 13, 24 }
}
