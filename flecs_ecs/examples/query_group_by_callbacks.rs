mod common;
use std::ffi::c_void;

use common::*;

use std::sync::Mutex;

static GROUP_COUNTER: Mutex<i32> = Mutex::new(0);

struct GroupCtx {
    counter: i32,
}

// callbacks need to be extern "C" to be callable from C
extern "C" fn callback_group_create(
    world: *mut WorldT,
    group_id: u64,
    _group_by_ctx: *mut c_void,
) -> *mut c_void {
    let world = World::new_wrap_raw_world(world);
    println!(
        "Group created: {:?}",
        world.new_entity_from_id(group_id).get_name()
    );

    println!();

    let mut counter = GROUP_COUNTER.lock().unwrap();
    *counter += 1;

    // Return data that will be associated with the group
    let ctx = Box::new(GroupCtx { counter: *counter });

    Box::into_raw(ctx) as *mut std::ffi::c_void // Cast to make sure function type matches
}

// callbacks need to be extern "C" to be callable from C
extern "C" fn callback_group_delete(
    world: *mut WorldT,
    group_id: u64,
    ctx: *mut c_void,
    _group_by_arg: *mut c_void,
) {
    let world = World::new_wrap_raw_world(world);
    println!(
        "Group deleted: {:?}",
        world.new_entity_from_id(group_id).get_name()
    );

    //Free data associated with group
    unsafe { drop(Box::from_raw(ctx as *mut GroupCtx)) };
}

fn main() {
    let world = World::new();

    // Register components in order so that id for First is lower than Third
    world.component::<First>();
    world.component::<Second>();
    world.component::<Third>();

    // Grouped query
    let query = world
        .query_builder::<(&Position,)>()
        .group_by::<Group>()
        // Callback invoked when a new group is created
        .on_group_create(Some(callback_group_create))
        // Callback invoked when a group is deleted
        .on_group_delete(Some(callback_group_delete))
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

    query.iter(|it, (pos,)| {
        let group = world.new_entity_from_id(it.get_group_id());
        let ctx = unsafe { &*(query.get_group_context(group) as *mut GroupCtx) };
        println!(
            "Group: {:?} - Table: [{:?}] - Counter: {}",
            group.get_path().unwrap(),
            it.get_archetype(),
            ctx.counter
        );

        for i in it.iter() {
            println!(" [{:?}]", pos[i]);
        }

        println!();
    });

    // Deleting the query will call the on_group_deleted callback

    query.destruct();

    // Output:
    //  Group created: "Third"
    //  Group created: "Second"
    //  Group created: "First"
    //
    //  Group: "::First" - Table: [Position, (Group,First)] - Counter: 3
    //   [Position { x: 3.0, y: 3.0 }]
    //
    //  Group: "::First" - Table: [Position, Tag, (Group,First)] - Counter: 3
    //   [Position { x: 6.0, y: 6.0 }]
    //
    //  Group: "::Second" - Table: [Position, (Group,Second)] - Counter: 2
    //   [Position { x: 2.0, y: 2.0 }]
    //
    //  Group: "::Second" - Table: [Position, Tag, (Group,Second)] - Counter: 2
    //   [Position { x: 5.0, y: 5.0 }]
    //
    //  Group: "::Third" - Table: [Position, (Group,Third)] - Counter: 1
    //   [Position { x: 1.0, y: 1.0 }]
    //
    //  Group: "::Third" - Table: [Position, Tag, (Group,Third)] - Counter: 1
    //   [Position { x: 4.0, y: 4.0 }]
    //
    //  Group deleted: "Second"
    //  Group deleted: "First"
    //  Group deleted: "Third"
}
