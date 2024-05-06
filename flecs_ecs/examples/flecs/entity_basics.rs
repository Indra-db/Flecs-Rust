use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Walking;

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create an entity with name Bob
    let bob = world
        .entity_named(c"Bob")
        // The set operation finds or creates a component, and sets it.
        // Components are automatically registered with the world
        .set(Position { x: 10.0, y: 20.0 })
        // The add operation adds a component without setting a value. This is
        // useful for tags, or when adding a component with its default value.
        .add::<Walking>();

    // Get the value for the Position component
    let pos = bob.try_get::<Position>().unwrap();
    fprintln!(&world, "Bob's position: {:?}", pos);

    // Overwrite the value of the Position component
    bob.set(Position { x: 20.0, y: 30.0 });

    // Create another named entity
    let alice = world
        .entity_named(c"Alice")
        .set(Position { x: 10.0, y: 20.0 });

    // Add a tag after entity is created
    alice.add::<Walking>();

    // Print all of the components the entity has. This will output:
    //    Position, Walking, (Identifier,Name)
    fprintln!(&world, "[{}]", alice.archetype());

    // Remove tag
    alice.remove::<Walking>();

    // Iterate all entities with position
    world.each_entity::<&Position>(|entity, pos| {
        fprintln!(entity, "{} has {:?}", entity.name(), pos);
    });

    world.get::<Snap>().test("entity_basics".to_string());

    // Output:
    //  Bob's position: Position { x: 10.0, y: 20.0 }
    //  [Position, Walking, (Identifier,Name)]
    //  Alice has Position { x: 10.0, y: 20.0 }
    //  Bob has Position { x: 20.0, y: 30.0 }
}
