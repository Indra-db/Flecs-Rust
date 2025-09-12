use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

// A monitor observer triggers when an entity starts/stop matching the observer
// query. The observer communicates whether an entity is "entering/leaving" the
// monitor by setting ecs_iter_t::event to EcsOnAdd (for entering) or
// EcsOnRemove (for leaving).
//
// To specify that an observer is a monitor observer, the EcsMonitor tag must be
// provided as event. No additional event kinds should be provided for a monitor
// observer.

fn main() {
    let world = World::new();

    // Create observer for custom event
    world
        .observer::<flecs::Monitor, (&Position, &Velocity)>()
        .each_iter(|it, index, (_pos, _vel)| {
            if it.event() == flecs::OnAdd::ID {
                println!(
                    " - Enter: {}: {}",
                    it.event_id().to_str(),
                    it.entity(index).name()
                );
            } else if it.event() == flecs::OnRemove::ID {
                println!(
                    " - Leave: {}: {}",
                    it.event_id().to_str(),
                    it.entity(index).name()
                );
            }
        });

    // Create entity
    let entity = world.entity_named("e");

    // This does not yet trigger the monitor, as the entity does not yet match.
    entity.set(Position { x: 10.0, y: 20.0 });

    // This triggers the monitor with EcsOnAdd, as the entity now matches.
    entity.set(Velocity { x: 1.0, y: 2.0 });

    // This triggers the monitor with EcsOnRemove, as the entity no longer matches.
    entity.remove(Position::id());

    // Output:
    //  - Enter: Velocity: e
    //  - Leave: Position: e
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("observer_monitor".to_string());
}
