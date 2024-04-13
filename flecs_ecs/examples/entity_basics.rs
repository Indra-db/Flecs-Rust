mod common;
use common::*;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create an entity with name Bob
    let bob = world
        .new_entity_named(c"Bob")
        // The set operation finds or creates a component, and sets it.
        // Components are automatically registered with the world
        .set(Position { x: 10.0, y: 20.0 })
        // The add operation adds a component without setting a value. This is
        // useful for tags, or when adding a component with its default value.
        .add::<Walking>();

    // Get the value for the Position component
    let pos = bob.get::<Position>().unwrap();
    fprintln!(snap, "Bob's position: {:?}", pos);

    // Overwrite the value of the Position component
    bob.set(Position { x: 20.0, y: 30.0 });

    // Create another named entity
    let alice = world
        .new_entity_named(c"Alice")
        .set(Position { x: 10.0, y: 20.0 });

    // Add a tag after entity is created
    alice.add::<Walking>();

    // Print all of the components the entity has. This will output:
    //    Position, Walking, (Identifier,Name)
    fprintln!(snap, "[{}]", alice.archetype());

    // Remove tag
    alice.remove::<Walking>();

    // Iterate all entities with position
    world.each_entity::<&Position>(|entity, pos| {
        fprintln!(snap, "{} has {:?}", entity.name(), pos);
    });

    snap.test();

    // Output:
    //  Bob's position: Position { x: 10.0, y: 20.0 }
    //  [Position, Walking, (Identifier,Name)]
    //  Alice has Position { x: 10.0, y: 20.0 }
    //  Bob has Position { x: 20.0, y: 30.0 }
}
