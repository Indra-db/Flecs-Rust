mod common;
use common::*;

//TODO: `.each` signature with it, index and EcsId not yet supported in flecs_ecs, this example needs to be updated when it is supported.
fn main() {
    let world = World::new();

    // Create an observer for three events
    world
        .observer_builder::<(&Position,)>()
        .add_event(ECS_ON_ADD)
        .add_event(ECS_ON_REMOVE)
        .add_event(ECS_ON_SET)
        .on_iter(|it, (_pos,)| {
            if it.get_event_as_entity() == ECS_ON_ADD {
                println!("OnAdd");
            } else if it.get_event_as_entity() == ECS_ON_REMOVE {
                println!("OnRemove");
            } else if it.get_event_as_entity() == ECS_ON_SET {
                println!("OnSet");
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
    //  OnAdd
    //  OnSet
    //  OnRemove
}
