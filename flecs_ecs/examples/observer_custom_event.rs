mod common;

use common::*;

#[derive(Default, Clone, Component)]
struct MyEvent;

impl EventData for MyEvent {}

fn main() {
    let world = World::new();

    // Create an observer for three events
    world
        .observer_builder::<(&Position,)>()
        .add_event_type::<MyEvent>()
        .on_each_iter(|it, index, (_pos,)| {
            println!(
                " - {}: {}: {}",
                it.get_event().get_name(),
                it.get_event_id().to_str(),
                it.get_entity(index)
            );
        });

    // The observer filter can be matched against the entity, so make sure it
    // has the Position component before emitting the event. This does not
    // trigger the observer yet.
    let entity = world
        .new_entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 });

    // Emit the custom event. This triggers the observer.
    world
        .event::<MyEvent>()
        .add_type_to_emit::<Position>()
        .set_entity_to_emit(entity)
        .emit();

    // Output:
    //  - MyEvent: Position: e1
}
