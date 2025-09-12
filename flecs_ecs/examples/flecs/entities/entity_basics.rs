use crate::z_ignore_test_common::*;

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

fn main() {
    let world = World::new();

    // Create an entity with name Bob
    let bob = world
        .entity_named("Bob")
        // The set operation finds or creates a component, and sets it.
        // Components are automatically registered with the world
        .set(Position { x: 10.0, y: 20.0 })
        // The add operation adds a component without setting a value. This is
        // useful for tags, or when adding a component with its default value.
        .add(Walking);

    // Get the value for the Position component
    // - get panics if the component is not present, use try_get for a non-panicking version which does not run the callback.
    // - or use Option to handle the individual component missing.
    bob.get::<Option<&Position>>(|pos| {
        if let Some(pos) = pos {
            println!("Bob's position: {pos:?}");
        }
    });

    // Overwrite the value of the Position component
    bob.set(Position { x: 20.0, y: 30.0 });

    // Create another named entity
    let alice = world
        .entity_named("Alice")
        .set(Position { x: 10.0, y: 20.0 });

    // Add a tag after entity is created
    alice.add(Walking);

    // Print all of the components the entity has. This will output:
    //    Position, Walking, (Identifier,Name)
    println!("[{}]", alice.archetype());

    // Remove tag
    alice.remove(Walking);

    // Iterate all entities with position
    world.each_entity::<&Position>(|entity, pos| {
        println!("{} has {:?}", entity.name(), pos);
    });

    // Output:
    //  Bob's position: Position { x: 10.0, y: 20.0 }
    //  [Position, Walking, (Identifier,Name)]
    //  Alice has Position { x: 10.0, y: 20.0 }
    //  Bob has Position { x: 20.0, y: 30.0 }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("entity_basics".to_string());
}
