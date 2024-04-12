mod common;
use std::ffi::c_void;

use common::*;
use flecs_ecs_sys::ecs_search;

// callbacks need to be extern "C" to be callable from C
extern "C" fn callback_group_by_relationship(
    world: *mut WorldT,
    table: *mut TableT,
    id: u64,
    _group_by_ctx: *mut c_void,
) -> u64 {
    // Use ecs_search to find the target for the relationship in the table
    let mut match_id: IdT = Default::default();
    let world = unsafe { WorldRef::from_ptr(world) };
    let id = IdView::new_from(world, (id, flecs::Wildcard::ID)).id();
    if unsafe { ecs_search(world.world_ptr_mut(), table, *id, &mut match_id) } != -1 {
        *IdView::new_from(world, match_id).second().id() // First, Second or Third
    } else {
        0
    }
}

fn main() {
    let world = World::new();

    // Register components in order so that id for First is lower than Third
    world.component::<First>();
    world.component::<Second>();
    world.component::<Third>();

    // Grouped query
    let query = world
        .query::<&Position>()
        .group_by_fn::<Group>(Some(callback_group_by_relationship))
        .build();

    // Create entities in 6 different tables with 3 group ids
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

    // The query cache now looks like this:
    //  - group First:
    //     - table [Position, (Group, First)]
    //     - table [Position, Tag, (Group, First)]
    //
    //  - group Second:
    //     - table [Position, (Group, Second)]
    //     - table [Position, Tag, (Group, Second)]
    //
    //  - group Third:
    //     - table [Position, (Group, Third)]
    //     - table [Position, Tag, (Group, Third)]
    //

    query.iter(|it, pos| {
        let group = world.new_entity_from_id(it.group_id());
        println!(
            "Group: {:?} - Table: [{:?}]",
            group.path().unwrap(),
            it.archetype(),
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
