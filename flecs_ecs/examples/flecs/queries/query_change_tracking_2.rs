#![allow(unused_imports)]
#![allow(warnings)]
use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
// Queries have a builtin mechanism for tracking changes per matched table. This
// is a cheap way of eliminating redundant work, as many entities can be skipped
// with a single check.
//
// This example shows how to use change tracking to skip tables that have not
// changed. This is useful when you have a query that writes to a component, but
// only want to write to tables that have changed.

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
}

// this is to make sure independent entity is in a different archetype
#[derive(Component)]
pub struct DummyTag;

// This system updates the WorldPosition component based on the Position and
// Parent components. If the Parent component is not present, the WorldPosition
// component is set to the Position component. If the Parent component is
// present, the WorldPosition component is set to the sum of the Position and
// Parent components.
fn update_transforms(mut it: TableIter<true, ()>) {
    while it.next() {
        // With the it.changed() function we can check if the table we're
        // currently iterating has changed since last iteration. If the table
        // has not changed, we can skip it.
        if !it.is_changed() {
            println!("skip archetype: {:?}", it.archetype().unwrap());
            it.skip();
            continue;
        }

        println!("non-skip archetype: {:?}", it.archetype().unwrap());

        let pos = it.field::<&Position>(0).unwrap();
        let parent_pos = it.field::<&Position>(1); // Optional
        let mut world_pos = it.field::<&mut WorldPosition>(2).unwrap();

        match parent_pos {
            None => {
                for i in it.iter() {
                    world_pos[i].x = pos[i].x;
                    world_pos[i].y = pos[i].y;
                }
            }
            Some(parent_pos) => {
                for i in it.iter() {
                    world_pos[i].x = pos[i].x + parent_pos[i].x;
                    world_pos[i].y = pos[i].y + parent_pos[i].y;
                }
            }
        }
    }
}

fn print_world_positions(entity: EntityView, world_pos: &WorldPosition) {
    println!("{}: {:?}", entity.name(), world_pos);
}
#[test]
fn main() {
    let world = World::new();

    let transform_query = world
        .query_named::<(&Position, Option<&Position>, &mut WorldPosition)>("update_transforms")
        .term_at(1)
        .parent()
        .build();

    let print_world_pos = world.query::<&WorldPosition>().build();

    // create entities with and without parent
    let parent = world
        .entity_named("parent")
        .set(Position { x: 100.0, y: 200.0 })
        .set(WorldPosition { x: 0.0, y: 0.0 });

    let child = world
        .entity_named("child")
        .set(Position { x: 10.0, y: 20.0 })
        .set(WorldPosition { x: 0.0, y: 0.0 })
        .child_of_id(parent);

    let independent = world
        .entity_named("independent")
        .set(Position { x: 50.0, y: 30.0 })
        .set(WorldPosition { x: 0.0, y: 0.0 })
        // this is to make sure independent entity is in a different archetype
        .add::<DummyTag>();

    // We can use the changed() function on the query to check if any of the
    // tables it is matched with has changed. Since this is the first time that
    // we check this and the query is matched with the tables we just created,
    // the function will return true.
    println!();
    println!(
        "transform_query.is_changed(): {}",
        transform_query.is_changed() // true
    );
    println!();

    // The changed state will remain true until we have iterated each table.
    // Because this is the first time the query is iterated, all tables
    // will show up as changed.
    transform_query.run(update_transforms);

    // Output:
    //  non-skip archetype: Position, WorldPosition
    //  non-skip archetype: Position, WorldPosition, (ChildOf,parent)
    //  non-skip archetype: Position, WorldPosition, DummyTag

    // Now that we have iterated all tables, the state is reset.
    println!();
    println!(
        "transform_query.is_changed(): {:?}",
        transform_query.is_changed() // false
    );
    println!();

    // Set the child position to a new value. This will change the table that
    // the child entity is in, which will cause the query to return true when
    // we call changed().
    child.set(Position { x: 110.0, y: 210.0 });

    // One of the tables has changed, so q_read.changed() will return true
    println!();
    println!(
        "transform_query.is_changed(): {}",
        transform_query.is_changed() // true
    );
    println!();

    // When we iterate the read query, we'll see that one table has changed.
    transform_query.run(update_transforms);

    println!();

    // Output:
    //  skip archetype: Position, WorldPosition
    //  non-skip archetype: Position, WorldPosition, (ChildOf,parent)
    //  skip archetype: Position, WorldPosition, DummyTag

    print_world_pos.each_entity(print_world_positions);

    // Output:
    //  parent: WorldPosition { x: 110.0, y: 210.0 }
    //  independent: WorldPosition { x: 50.0, y: 30.0 }
    //  child: WorldPosition { x: 120.0, y: 230.0 }

    // now the same, but for independent entity

    independent.set(Position { x: 60.0, y: 40.0 });

    println!();
    println!(
        "transform_query.is_changed(): {}",
        transform_query.is_changed() // true
    );
    println!();

    transform_query.run(update_transforms);

    println!();

    // Output:
    //  skip archetype: Position, WorldPosition
    //  skip archetype: Position, WorldPosition, (ChildOf,parent)
    //  non-skip archetype: Position, WorldPosition, DummyTag

    print_world_pos.each_entity(print_world_positions);

    // Output:
    //  parent: WorldPosition { x: 110.0, y: 210.0 }
    //  independent: WorldPosition { x: 60.0, y: 40.0 }
    //  child: WorldPosition { x: 120.0, y: 230.0 }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_change_tracking_2".to_string());
}
