mod common;
use common::*;

pub use flecs_ecs::{core::*, macros::Component};

#[derive(Component)]
pub struct Eats {
    pub amount: i32,
}

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create a query that matches edible components
    let query = world
        .query::<&Eats>()
        .term_at(0)
        // Change first argument to (Eats, *)
        // alternative you can do  `.select_second_id(flecs::Wildcard::ID)``
        .select_second::<flecs::Wildcard>()
        .build();

    // Create a few entities that match the query
    world
        .entity_named(c"Bob")
        .set_first::<Eats, Apples>(Eats { amount: 10 })
        .set_first::<Eats, Pears>(Eats { amount: 5 });

    world
        .entity_named(c"Alice")
        .set_first::<Eats, Apples>(Eats { amount: 4 });

    // Iterate the query with a flecs::iter. This makes it possible to inspect
    // the pair that we are currently matched with.
    query.each_iter(|it, index, eats| {
        let entity = it.entity(index);
        let pair = it.pair(0).unwrap();
        let food = pair.second();

        fprintln!(snap, "{} eats {} {}", entity, eats.amount, food);
    });

    snap.test();

    // Output:
    //  Alice eats 4 Apples
    //  Bob eats 10 Apples
    //  Bob eats 5 Pears
}
