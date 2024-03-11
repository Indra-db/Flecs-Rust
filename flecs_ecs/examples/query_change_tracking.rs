mod common;
use common::*;

// Queries have a builtin mechanism for tracking changes per matched table. This
// is a cheap way of eliminating redundant work, as many entities can be skipped
// with a single check.
//
// This example shows how to use change tracking in combination with a few other
// techniques, like using prefabs to store a single dirty state for multiple
// entities and instanced queries.

#[derive(Debug, Default, Clone, Component)]
struct Dirty {
    value: bool,
}

pub fn main() {
    let world = World::new();

    // Create a query that just reads a component. We'll use this query for
    // change tracking. Change tracking for a query is automatically enabled
    // when query::changed() is called.
    // Each query has its own private dirty state which is reset only when the
    // query is iterated.

    let query_read = world.query::<(&Position,)>();

    // Create a query that writes the component based on a Dirty state.
    let query_write = world
        .query_builder::<(&Dirty, &mut Position)>()
        .term_at(1)
        .up()
        .instanced()
        .build();

    // Create two prefabs with a Dirty component. We can use this to share a
    // single Dirty value for all entities in a table.
    let prefab_dirty_false = world
        .prefab_named(CStr::from_bytes_with_nul(b"prefab_dirty_false\0").unwrap())
        .set(Dirty { value: false });

    let prefab_dirty_true = world
        .prefab_named(CStr::from_bytes_with_nul(b"prefab_dirty_true\0").unwrap())
        .set(Dirty { value: true });

    // Create instances of p1 and p2. Because the entities have different
    // prefabs, they end up in different tables.
    world
        .new_entity_named(CStr::from_bytes_with_nul(b"e1_dirty_false\0").unwrap())
        .is_a(&prefab_dirty_false)
        .set(Position { x: 10.0, y: 20.0 });

    world
        .new_entity_named(CStr::from_bytes_with_nul(b"e2_dirty_false\0").unwrap())
        .is_a(&prefab_dirty_false)
        .set(Position { x: 30.0, y: 40.0 });

    world
        .new_entity_named(CStr::from_bytes_with_nul(b"e3_dirty_true\0").unwrap())
        .is_a(&prefab_dirty_true)
        .set(Position { x: 40.0, y: 50.0 });

    world
        .new_entity_named(CStr::from_bytes_with_nul(b"e4_dirty_true\0").unwrap())
        .is_a(&prefab_dirty_true)
        .set(Position { x: 50.0, y: 60.0 });

    // We can use the changed() function on the query to check if any of the
    // tables it is matched with has changed. Since this is the first time that
    // we check this and the query is matched with the tables we just created,
    // the function will return true.
    println!();
    println!("query_read.is_changed(): {}", query_read.is_changed());
    println!();

    // The changed state will remain true until we have iterated each table.
    query_read.iter_only(|iter| {
        // With the it.changed() function we can check if the table we're
        // currently iterating has changed since last iteration.
        // Because this is the first time the query is iterated, all tables
        // will show up as changed.
        println!(
            "iiter.is_changed() for table [{}]: {}",
            iter.get_archetype(),
            iter.is_changed()
        )
    });

    // Now that we have iterated all tables, the dirty state is reset.
    println!();
    println!("query_read.is_changed(): {:?}", query_read.is_changed());
    println!();

    // Iterate the write query. Because the Position term is InOut (default)
    // iterating the query will write to the dirty state of iterated tables.
    query_write.iter(|iter, (dirty, pos)| {
        println!("iterate table [{}]", iter.get_archetype());

        // Because we enforced that Dirty is a shared component, we can check
        // a single value for the entire table.
        if !dirty[0].value {
            // If the dirty flag is false, skip the table. This way the table's
            // dirty state is not updated by the query.
            iter.skip();
            println!("iter.skip() for table [{}]", iter.get_archetype());
            return;
        }

        // For all other tables the dirty state will be set.
        for i in iter {
            pos[i].x += 1.0;
            pos[i].y += 1.0;
        }
    });

    // One of the tables has changed, so q_read.changed() will return true
    println!();
    println!("query_read.is_changed(): {}", query_read.is_changed());
    println!();

    // When we iterate the read query, we'll see that one table has changed.
    query_read.iter_only(|iter| {
        println!(
            "iter.is_changed() for table [{}]: {}",
            iter.get_archetype(),
            iter.is_changed()
        )
    });
    println!();

    // Output:
    //  query_read.is_changed(): true
    //
    //  iiter.is_changed() for table [Position, (Identifier,Name), (IsA,prefab_dirty_false)]: true
    //  iiter.is_changed() for table [Position, (Identifier,Name), (IsA,prefab_dirty_true)]: true
    //
    //  query_read.is_changed(): false
    //
    //  iterate table [Position, (Identifier,Name), (IsA,prefab_dirty_false)]
    //  iter.skip() for table [Position, (Identifier,Name), (IsA,prefab_dirty_false)]
    //  iterate table [Position, (Identifier,Name), (IsA,prefab_dirty_true)]
    //
    //  query_read.is_changed(): true
    //
    //  iter.is_changed() for table [Position, (Identifier,Name), (IsA,prefab_dirty_false)]: false
    //  iter.is_changed() for table [Position, (Identifier,Name), (IsA,prefab_dirty_true)]: true
}
