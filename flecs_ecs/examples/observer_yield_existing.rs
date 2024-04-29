include!("common");

// Observers can enable a "yield_existing" feature that upon creation of the
// observer produces events for all entities that match the observer query. The
// feature is only implemented for the builtin EcsOnAdd and EcsOnSet events.
//
// Custom events can also implement behavior for yield_existing by adding the
// Iterable component to the event (see EcsIterable for more details).

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create existing entities with Position component
    world.entity_named(c"e1").set(Position { x: 10.0, y: 20.0 });
    world.entity_named(c"e2").set(Position { x: 20.0, y: 30.0 });

    // Create an observer for three events
    world
        .observer::<flecs::OnSet, &Position>()
        .yield_existing(true)
        .each_iter(|it, index, pos| {
            fprintln!(
                snap,
                " - {}: {}: {}: {{ {}, {} }}",
                it.event().name(),
                it.event_id().to_str(),
                it.entity(index),
                pos.x,
                pos.y
            );
        });

    Ok(snap)

    // Output:
    //  - OnSet: Position: e1: { 10, 20 }
    //  - OnSet: Position: e2: { 20, 30 }
}
