use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
pub use flecs_ecs::{core::*, macros::Component};

#[derive(Component)]
pub struct EatsAmount {
    pub amount: i32,
}

#[derive(Component)]
pub struct Apples;

#[derive(Component)]
pub struct Pears;

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create a query that matches edible components
    let query = world
        .query::<&EatsAmount>()
        .term_at(0)
        // Change first argument to (Eats, *)
        // alternative you can do  `.set_second_id(flecs::Wildcard::ID)``
        .set_second::<flecs::Wildcard>()
        .build();

    // Create a few entities that match the query
    world
        .entity_named(c"Bob")
        .set_pair::<EatsAmount, Apples>(EatsAmount { amount: 10 })
        .set_pair::<EatsAmount, Pears>(EatsAmount { amount: 5 });

    world
        .entity_named(c"Alice")
        .set_pair::<EatsAmount, Apples>(EatsAmount { amount: 4 });

    // Iterate the query with a flecs::iter. This makes it possible to inspect
    // the pair that we are currently matched with.
    query.each_iter(|it, index, eats| {
        let entity = it.entity(index);
        let pair = it.pair(0).unwrap();
        let food = pair.second_id();

        fprintln!(it, "{} eats {} {}", entity, eats.amount, food);
    });

    world.get::<&Snap>(|snap| snap.test("query_wildcard".to_string()));

    // Output:
    //  Alice eats 4 Apples
    //  Bob eats 10 Apples
    //  Bob eats 5 Pears
}
