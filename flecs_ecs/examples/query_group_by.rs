mod common;
use common::*;

fn main() {
    let world = World::new();

    world.component::<First>();
    world.component::<Second>();
    world.component::<Third>();

    let query = world
        .query_builder::<&Position>()
        .group_by::<Group>()
        .build();

    world
        .new_entity()
        .add::<(Group, Third)>()
        .set(Position { x: 1.0, y: 1.0 });
    world
        .new_entity()
        .add::<(Group, Second)>()
        .set(Position { x: 2.0, y: 2.0 });
    world
        .new_entity()
        .add::<(Group, First)>()
        .set(Position { x: 3.0, y: 3.0 });

    world
        .new_entity()
        .add::<(Group, Third)>()
        .set(Position { x: 4.0, y: 4.0 })
        .add::<Tag>();
    world
        .new_entity()
        .add::<(Group, Second)>()
        .set(Position { x: 5.0, y: 5.0 })
        .add::<Tag>();
    world
        .new_entity()
        .add::<(Group, First)>()
        .set(Position { x: 6.0, y: 6.0 })
        .add::<Tag>();

    println!();

    query.iter(|it: Iter, pos: &[Position]| {
        let group = world.new_entity_from_id(it.group_id());
        println!(
            "Group: {:?} - Table: [{:?}]",
            group.path().unwrap(),
            it.archetype()
        );

        for i in it.iter() {
            println!(" [{:?}]", pos[i]);
        }

        println!();
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
