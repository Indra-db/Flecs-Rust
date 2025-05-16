use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Tag;

#[derive(Component)]
pub struct First;

#[derive(Component)]
pub struct Second;

#[derive(Component)]
pub struct Third;

#[derive(Component)]
pub struct Group;

fn main() {
    let world = World::new();

    world.component::<First>();
    world.component::<Second>();
    world.component::<Third>();

    let query = world.query::<&Position>().group_by(id::<Group>()).build();

    world
        .entity()
        .add((id::<Group>(), id::<Third>()))
        .set(Position { x: 1.0, y: 1.0 });
    world
        .entity()
        .add((id::<Group>(), id::<Second>()))
        .set(Position { x: 2.0, y: 2.0 });
    world
        .entity()
        .add((id::<Group>(), id::<First>()))
        .set(Position { x: 3.0, y: 3.0 });

    world
        .entity()
        .add((id::<Group>(), id::<Third>()))
        .set(Position { x: 4.0, y: 4.0 })
        .add(id::<Tag>());
    world
        .entity()
        .add((id::<Group>(), id::<Second>()))
        .set(Position { x: 5.0, y: 5.0 })
        .add(id::<Tag>());
    world
        .entity()
        .add((id::<Group>(), id::<First>()))
        .set(Position { x: 6.0, y: 6.0 })
        .add(id::<Tag>());

    println!();

    query.run(|mut it| {
        while it.next() {
            let pos = it.field_mut::<Position>(0).unwrap();
            let group = world.entity_from_id(it.group_id());
            println!(
                "Group: {:?} - Table: [{:?}]",
                group.path().unwrap(),
                it.archetype()
            );

            for i in it.iter() {
                println!(" [{:?}]", pos[i]);
            }

            println!();
        }
    });

    // Output:
    //  Group: "::First" - Table: [Position, (Group,First)]
    //  [Position { x: 3.0, y: 3.0 }]
    //
    //  Group: "::First" - Table: [Position, Tag, (Group,First)]
    //  [Position { x: 6.0, y: 6.0 }]
    //
    //  Group: "::Second" - Table: [Position, (Group,Second)]
    //  [Position { x: 2.0, y: 2.0 }]
    //
    //  Group: "::Second" - Table: [Position, Tag, (Group,Second)]
    //  [Position { x: 5.0, y: 5.0 }]
    //
    //  Group: "::Third" - Table: [Position, (Group,Third)]
    //  [Position { x: 1.0, y: 1.0 }]
    //
    //  Group: "::Third" - Table: [Position, Tag, (Group,Third)]
    //  [Position { x: 4.0, y: 4.0 }]
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_group_by".to_string());
}
