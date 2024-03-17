mod common;
use common::*;

// Observers can enable a "yield_existing" feature that upon creation of the
// observer produces events for all entities that match the observer query. The
// feature is only implemented for the builtin EcsOnAdd and EcsOnSet events.
//
// Custom events can also implement behavior for yield_existing by adding the
// Iterable component to the event (see EcsIterable for more details).

fn main() {
    let world = World::new();

    // Create existing entities with Position component
    world
        .new_entity_named(CStr::from_bytes_with_nul(b"e1\0").unwrap())
        .set(Position { x: 10.0, y: 20.0 });
    world
        .new_entity_named(CStr::from_bytes_with_nul(b"e2\0").unwrap())
        .set(Position { x: 20.0, y: 30.0 });

    // Create an observer for three events
    world
        .observer_builder::<(&Position,)>()
        .add_event(ECS_ON_SET)
        .yield_existing(true)
        .on_each_iter(|it, index, (pos,)| {
            println!(
                " - {}: {}: {}: {{ {}, {} }}",
                it.get_event().get_name(),
                it.get_event_id().to_str(),
                it.get_entity(index),
                pos.x,
                pos.y
            );
        });

    // Output
    //  - OnSet: Position: e1: { 10, 20 }
    //  - OnSet: Position: e2: { 20, 30 }
}
