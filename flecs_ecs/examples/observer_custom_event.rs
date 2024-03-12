mod common;

use common::*;

#[derive(Default, Clone, Component)]
struct MyEvent;

impl EventData for MyEvent {}

//TODO: `.each` signature with it, index and EcsId not yet supported in flecs_ecs, this example needs to be updated when it is supported.
fn main() {
    let world = World::new();

    // Create an observer for three events
    world
        .observer_builder::<(&Position,)>()
        .add_event_type::<MyEvent>()
        .on_iter(|_it, (_pos,)| {
            println!("OnEvent");
        });

    // The observer filter can be matched against the entity, so make sure it
    // has the Position component before emitting the event. This does not
    // trigger the observer yet.
    let entity = world
        .new_entity_named(CStr::from_bytes_with_nul(b"e1\0").unwrap())
        .set(Position { x: 10.0, y: 20.0 });

    // Emit the custom event. This triggers the observer.
    world
        .event::<MyEvent>()
        .add_type_to_emit::<Position>()
        .set_entity_to_emit(&entity)
        .emit();

    // Output:
    //  OnEvent
}
