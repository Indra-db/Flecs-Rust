include!("common");

#[allow(dead_code)]
pub fn main() -> Result<World, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create an observer for three events
    world
        .observer::<&Position>()
        .add_event::<flecs::OnAdd>() //or .add_event_id(OnAdd::ID)
        .add_event::<flecs::OnRemove>()
        .add_event::<flecs::OnSet>()
        .each_iter(|it, index, pos| {
            if it.event() == flecs::OnAdd::ID {
                // No assumptions about the component value should be made here. If
                // a ctor for the component was registered it will be called before
                // the EcsOnAdd event, but a value assigned by set won't be visible.
                fprintln!(
                    it,
                    " - OnAdd: {}: {}",
                    it.event_id().to_str(),
                    it.entity(index)
                );
            } else {
                fprintln!(
                    it,
                    " - {}: {}: {}: with {:?}",
                    it.event().name(),
                    it.event_id().to_str(),
                    it.entity(index),
                    pos
                );
            }
        });

    // Create entity, set Position (emits EcsOnAdd and EcsOnSet)
    let entity = world.entity_named(c"e1").set(Position { x: 10.0, y: 20.0 });

    // Remove Position (emits EcsOnRemove)
    entity.remove::<Position>();

    // Remove Position again (no event emitted)
    entity.remove::<Position>();

    Ok(world)

    // Output:
    //  - OnAdd: Position: e1
    //  - OnSet: Position: e1: with Position { x: 10.0, y: 20.0 }
    //  - OnRemove: Position: e1: with Position { x: 10.0, y: 20.0 }
}
