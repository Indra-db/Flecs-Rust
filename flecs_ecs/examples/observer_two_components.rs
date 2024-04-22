include!("common");

// An observer can match multiple components/tags. Only entities that match the
// entire observer filter will be forwarded to the callback. For example, an
// observer for Position,Velocity won't match an entity that only has Position.

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create observer for custom event
    world
        .observer::<(&Position, &Velocity)>()
        .add_event::<flecs::OnSet>()
        .each_iter(|it, index, (pos, vel)| {
            fprintln!(
                snap,
                " - {}: {}: {}: p: {{ {}, {} }}, v: {{ {}, {} }}",
                it.event().name(),
                it.event_id().to_str(),
                it.entity(index).name(),
                pos.x,
                pos.y,
                vel.x,
                vel.y
            );
        });

    // Create entity, set Position (emits EcsOnSet, does not yet match observer)
    let entity = world.entity_named(c"e").set(Position { x: 10.0, y: 20.0 });

    // Set Velocity (emits EcsOnSet, matches observer)
    entity.set(Velocity { x: 1.0, y: 2.0 });

    Ok(snap)

    // Output:
    //  - OnSet: Velocity: e: p: { 10, 20 }, v: { 1, 2 }
}
