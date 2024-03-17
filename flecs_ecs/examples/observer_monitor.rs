mod common;
use common::*;

// A monitor observer triggers when an entity starts/stop matching the observer
// filter. The observer communicates whether an entity is "entering/leaving" the
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
        .observer_builder::<(&Position, &Velocity)>()
        .add_event(ECS_MONITOR)
        .on_each_iter(|it, index, (_pos, _vel)| {
            if it.get_event() == ECS_ON_ADD {
                println!(
                    " - Enter: {}: {}",
                    it.get_event_id().to_str(),
                    it.get_entity(index).get_name()
                );
            } else if it.get_event() == ECS_ON_REMOVE {
                println!(
                    " - Leave: {}: {}",
                    it.get_event_id().to_str(),
                    it.get_entity(index).get_name()
                );
            }
        });

    // Create entity
    let entity = world.new_entity_named(CStr::from_bytes_with_nul(b"e\0").unwrap());

    // This does not yet trigger the monitor, as the entity does not yet match.
    entity.set(Position { x: 10.0, y: 20.0 });

    // This triggers the monitor with EcsOnAdd, as the entity now matches.
    entity.set(Velocity { x: 1.0, y: 2.0 });

    // This triggers the monitor with EcsOnRemove, as the entity no longer matches.
    entity.remove::<Position>();

    // Output
    //  - Enter: Velocity: e
    //  - Leave: Position: e
}
