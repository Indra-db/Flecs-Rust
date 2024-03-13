mod common;
use common::*;

fn main() {
    let world = World::new();

    // Create an observer for three events
    world
        .observer_builder::<(&Position,)>()
        .add_event(ECS_ON_ADD)
        .add_event(ECS_ON_REMOVE)
        .add_event(ECS_ON_SET)
        .on_each_iter(|it, index, (pos,)| {
            if it.get_event() == ECS_ON_ADD {
                // No assumptions about the component value should be made here. If
                // a ctor for the component was registered it will be called before
                // the EcsOnAdd event, but a value assigned by set won't be visible.
                println!(
                    " - OnAdd: {}: {}",
                    it.get_event_id().to_str(),
                    it.get_entity(index)
                );
            } else {
                println!(
                    " - {}: {}: {}: with {:?}",
                    it.get_event().get_name(),
                    it.get_event_id().to_str(),
                    it.get_entity(index),
                    pos
                );
            }
        });

    // Create entity, set Position (emits EcsOnAdd and EcsOnSet)
    let entity = world
        .new_entity_named(CStr::from_bytes_with_nul(b"e1\0").unwrap())
        .set(Position { x: 10.0, y: 20.0 });

    // Remove Position (emits EcsOnRemove)
    entity.remove::<Position>();

    // Remove Position again (no event emitted)
    entity.remove::<Position>();

    // Output:
    //  - OnAdd: Position: e1
    //  - OnSet: Position: e1: with Position { x: 10.0, y: 20.0 }
    //  - OnRemove: Position: e1: with Position { x: 10.0, y: 20.0 }
}
