include!("common");

#[derive(Component)]
struct MyEvent;

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create an observer for the custom event
    world
        .observer::<MyEvent, &Position>()
        .each_iter(|it, index, _pos| {
            fprintln!(
                snap,
                " - {}: {}: {}",
                it.event().name(),
                it.event_id().to_str(),
                it.entity(index)
            );
        });

    // The observer filter can be matched against the entity, so make sure it
    // has the Position component before emitting the event. This does not
    // trigger the observer yet.
    let entity = world.entity_named(c"e1").set(Position { x: 10.0, y: 20.0 });

    // Emit the custom event. This triggers the observer.
    world
        .event()
        .add::<Position>()
        .target(entity)
        .emit(&mut MyEvent);

    Ok(snap)

    // Output:
    //  - MyEvent: Position: e1
}
