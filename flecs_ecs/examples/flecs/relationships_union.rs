use crate::z_snapshot_test::*;
snapshot_test!();
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

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // disabled v4 not yet supported
    /*
        // Register Movement and Direction as union relationships. This ensures that
        // an entity can only have one Movement and one Direction.
        world.component::<Movement>().add::<flecs::Union>();
        world.component::<Direction>().add::<flecs::Union>();

        // Create a query that subscribes for all entities that have a Direction
        // and that are walking.
        // with<T>() requests no data by  so we must specify what we want.
        // in() requests Read-Only
        let q = world
        .query::<()>()
        .with_enum(Movement::Walking)
        .in_()
        .with_enum_wildcard::<Direction>()
        .in_()
        .build();

    // Create a few entities with various state combinations
    world
    .entity_named(c"e1")
    .add_enum(Movement::Walking)
    .add_enum(Direction::Front);

    world
    .entity_named(c"e2")
    .add_enum(Movement::Running)
    .add_enum(Direction::Left);

    let e3 = world
    .entity_named(c"e3")
    .add_enum(Movement::Running)
    .add_enum(Direction::Back);

    // Add Walking to e3. This will remove the Running case
    e3.add_enum(Movement::Walking);

    // Iterate the query
    q.iter_only(|it| {
        // Get the column with direction states. This is stored as an array
        // with identifiers to the individual states
        //since it's an union, we need to get the entity id for safety
        let movement = it.field::<Entity>(0).unwrap();
        let direction = it.field::<Entity>(1).unwrap();

        for i in 0..it.count() {
            fprintln!(
                snap,
                "{}: Movement: {:?}, Direction: {:?}",
                it.entity(i).name(),
                movement[i]
                .entity_view(it.world())
                .to_constant::<Movement>()
                .unwrap(),
                direction[i]
                .entity_view(it.world())
                .to_constant::<Direction>()
                .unwrap()
            );
        }
    });

    world.get::<&Snap>(|snap| snap.test("relationships_union".to_string()));

    */

    // Output:
    //   e3: Movement: Walking, Direction: Back
    //   e1: Movement: Walking, Direction: Front
}
