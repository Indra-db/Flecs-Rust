mod common;
use common::*;

// An observer can match multiple components/tags. Only entities that match the
// entire observer filter will be forwarded to the callback. For example, an
// observer for Position,Velocity won't match an entity that only has Position.

fn main() {
    let world = World::new();

    // Create observer for custom event
    world
        .observer_builder::<(&Position, &Velocity)>()
        .add_event(ECS_ON_SET)
        .on_each_iter(|it, index, (pos, vel)| {
            println!(
                " - {}: {}: {}: p: {{ {}, {} }}, v: {{ {}, {} }}",
                it.get_event().get_name(),
                it.get_event_id().to_str(),
                it.get_entity(index).get_name(),
                pos.x,
                pos.y,
                vel.x,
                vel.y
            );
        });

    // Create entity, set Position (emits EcsOnSet, does not yet match observer)
    let entity = world
        .new_entity_named(CStr::from_bytes_with_nul(b"e\0").unwrap())
        .set(Position { x: 10.0, y: 20.0 });

    // Set Velocity (emits EcsOnSet, matches observer)
    entity.set(Velocity { x: 1.0, y: 2.0 });

    // Output
    //  - OnSet: Velocity: e: p: { 10, 20 }, v: { 1, 2 }
}
