include!("common");

// A monitor observer triggers when an entity starts/stop matching the observer
// filter. The observer communicates whether an entity is "entering/leaving" the
// monitor by setting ecs_iter_t::event to EcsOnAdd (for entering) or
// EcsOnRemove (for leaving).
//
// To specify that an observer is a monitor observer, the EcsMonitor tag must be
// provided as event. No additional event kinds should be provided for a monitor
// observer.

#[allow(dead_code)]
pub fn main() -> Result<World, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create observer for custom event
    world
        .observer::<(&Position, &Velocity)>()
        .add_event::<flecs::Monitor>()
        .each_iter(|it, index, (_pos, _vel)| {
            if it.event() == flecs::OnAdd::ID {
                fprintln!(
                    it,
                    " - Enter: {}: {}",
                    it.event_id().to_str(),
                    it.entity(index).name()
                );
            } else if it.event() == flecs::OnRemove::ID {
                fprintln!(
                    it,
                    " - Leave: {}: {}",
                    it.event_id().to_str(),
                    it.entity(index).name()
                );
            }
        });

    // Create entity
    let entity = world.entity_named(c"e");

    // This does not yet trigger the monitor, as the entity does not yet match.
    entity.set(Position { x: 10.0, y: 20.0 });

    // This triggers the monitor with EcsOnAdd, as the entity now matches.
    entity.set(Velocity { x: 1.0, y: 2.0 });

    // This triggers the monitor with EcsOnRemove, as the entity no longer matches.
    entity.remove::<Position>();

    Ok(world)

    // Output:
    //  - Enter: Velocity: e
    //  - Leave: Position: e
}
