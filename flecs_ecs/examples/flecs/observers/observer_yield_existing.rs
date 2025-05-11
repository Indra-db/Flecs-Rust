use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// Observers can enable a "yield_existing" feature that upon creation of the
// observer produces events for all entities that match the observer query. The
// feature is only implemented for the builtin EcsOnAdd and EcsOnSet events.

fn main() {
    let world = World::new();

    // Create existing entities with Position component
    world.entity_named("e1").set(Position { x: 10.0, y: 20.0 });
    world.entity_named("e2").set(Position { x: 20.0, y: 30.0 });

    // Create an observer for three events
    world
        .observer::<flecs::OnSet, &Position>()
        .yield_existing()
        .each_iter(|it, index, pos| {
            println!(
                " - {}: {}: {}: {{ {}, {} }}",
                it.event().name(),
                it.event_id().to_str(),
                it.entity(index).unwrap(),
                pos.x,
                pos.y
            );
        });

    // Output:
    //  - OnSet: Position: e1: { 10, 20 }
    //  - OnSet: Position: e2: { 20, 30 }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("observer_yield_existing".to_string());
}
