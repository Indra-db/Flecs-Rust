include!("common");

#[derive(Component)]
struct MyEvent;

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create an observer for three events
    world
        .observer::<&Position>()
        .add_event::<MyEvent>()
        .each_iter(|it, index, _pos| {
            fprintln!(
                it,
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
        .event::<MyEvent>()
        .add::<Position>()
        .set_entity_to_emit(entity)
        .emit();

    Ok(Snap::from(&world))

    // Output:
    //  - MyEvent: Position: e1
}
