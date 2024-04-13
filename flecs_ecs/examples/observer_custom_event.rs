mod common;

use common::*;

#[derive(Component)]
struct MyEvent;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create an observer for three events
    world
        .observer_builder::<&Position>()
        .add_event::<MyEvent>()
        .on_each_iter(|it, index, _pos| {
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
    let entity = world
        .new_entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 });

    // Emit the custom event. This triggers the observer.
    world
        .event::<MyEvent>()
        .add::<Position>()
        .set_entity_to_emit(entity)
        .emit();

    snap.test();

    // Output:
    //  - MyEvent: Position: e1
}
