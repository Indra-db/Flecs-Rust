use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
struct MyEvent;

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create an observer for the custom event
    world
        .observer::<MyEvent, &Position>()
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
        .event()
        .add::<Position>()
        .target(entity)
        .emit(&MyEvent);

    world
        .get::<Snap>()
        .test("observer_custom_event".to_string());

    // Output:
    //  - MyEvent: Position: e1
}
