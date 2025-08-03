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

// An observer can match multiple components/tags. Only entities that match the
// entire observer query will be forwarded to the callback. For example, an
// observer for Position,Velocity won't match an entity that only has Position.

fn main() {
    let world = World::new();

    // Create observer for custom event
    world
        .observer::<flecs::OnSet, (&Position, &Velocity)>()
        .run(|mut it| {
            // The observer will iterate over all entities that match the query.
            while it.next() {
                let pos = it.field::<Position>(0);
                let vel = it.field::<Velocity>(1);
                for i in it.iter() {
                    println!(
                        " - {}: {}: {}: p: {{ {}, {} }}, v: {{ {}, {} }}",
                        it.event().name(),
                        it.event_id().to_str(),
                        it.get_entity(i).unwrap().name(),
                        pos[i].x,
                        pos[i].y,
                        vel[i].x,
                        vel[i].y
                    );
                }
            }
        });

    // Create entity, set Position (emits EcsOnSet, does not yet match observer)
    let entity = world.entity_named("e").set(Position { x: 10.0, y: 20.0 });

    // Set Velocity (emits EcsOnSet, matches observer)
    entity.set(Velocity { x: 1.0, y: 2.0 });

    // Output:
    //  - OnSet: Velocity: e: p: { 10, 20 }, v: { 1, 2 }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("observer_two_components".to_string());
}
