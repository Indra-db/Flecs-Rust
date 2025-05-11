use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
// This example shows how to use union relationships. Union relationships behave
// much like exclusive relationships in that entities can have only one instance
// and that adding an instance removes the previous instance.
//
// What makes union relationships stand out is that changing the relationship
// target doesn't change the archetype of an entity. This allows for quick
// switching of tags, which can be useful when encoding state machines in ECS.
//
// There is a tradeoff, and that is that because a single archetype can contain
// entities with multiple targets, queries need to do a bit of extra work to
// only return the requested target.
//
// This code uses enumeration relationships. See the enum_relations example for
// more details.

#[derive(Component, Debug, PartialEq)]
#[repr(C)]
enum Movement {
    Walking,
    Running,
}

#[derive(Component, Debug, PartialEq)]
#[repr(C)]
enum Direction {
    Front,
    Back,
    Left,
    Right,
}

fn main() {
    let world = World::new();

    // Register Movement and Direction as union relationships. This ensures that
    // an entity can only have one Movement and one Direction.
    world.component::<Movement>().add(id::<flecs::Union>());
    world.component::<Direction>().add(id::<flecs::Union>());

    // Create a query that subscribes for all entities that have a Direction
    // and that are walking.
    let q = world
        .query::<()>()
        .with_enum(Movement::Walking)
        .with_enum_wildcard::<Direction>()
        .build();

    // Create a few entities with various state combinations
    world
        .entity_named("e1")
        .add_enum(Movement::Walking)
        .add_enum(Direction::Front);

    world
        .entity_named("e2")
        .add_enum(Movement::Running)
        .add_enum(Direction::Left);

    let e3 = world
        .entity_named("e3")
        .add_enum(Movement::Running)
        .add_enum(Direction::Back);

    // Add Walking to e3. This will remove the Running case
    e3.add_enum(Movement::Walking);

    // Iterate the query
    q.each_iter(|it, index, ()| {
        let entity = it.entity(index).unwrap();

        // Movement will always be Walking, Direction can be any state
        println!(
            "{}: Movement: {:?}, Direction: {:?}",
            entity.name(),
            it.pair(0).unwrap().second_id().name(),
            it.pair(1).unwrap().second_id().name()
        );
    });

    // Output:
    //   e3: Movement: Walking, Direction: Back
    //   e1: Movement: Walking, Direction: Front
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("relationships_union".to_string());
}
