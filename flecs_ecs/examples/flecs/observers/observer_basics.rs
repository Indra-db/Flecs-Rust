use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

fn main() {
    let world = World::new();

    world
        .observer::<flecs::OnAdd, ()>()
        // We use .with(Position::id()) because we cannot safely access the component
        // value here. The component value is uninitialized when the OnAdd event
        // is emitted, which is UB in Rust. To work around this, we use .with::<T>
        .with(Position::id())
        .run(|mut it| {
            while it.next() {
                for i in it.iter() {
                    println!(" - OnAdd: {}: {}", it.event_id().to_str(), it.entity_id(i));
                }
            }
        });

    // Create an observer for three events
    world
        .observer::<flecs::OnSet, &Position>()
        .add_event(flecs::OnRemove)
        .run(|mut it| {
            while it.next() {
                let pos = it.field::<Position>(0);
                for i in it.iter() {
                    println!(
                        " - {}: {}: with {:?}",
                        it.event().name(),
                        it.entity_id(i),
                        pos[i]
                    );
                }
            }
        });

    // Create entity, set Position (emits EcsOnAdd and EcsOnSet)
    let entity = world.entity_named("e1").set(Position { x: 10.0, y: 20.0 });

    // Remove Position (emits EcsOnRemove)
    entity.remove(Position::id());

    // Remove Position again (no event emitted)
    entity.remove(Position::id());

    // Output:
    //  - OnAdd: Position: e1
    //  - OnSet: Position: e1: with Position { x: 10.0, y: 20.0 }
    //  - OnRemove: Position: e1: with Position { x: 10.0, y: 20.0 }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("observer_basics".to_string());
}
