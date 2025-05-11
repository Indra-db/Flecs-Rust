use crate::z_ignore_test_common::*;

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

fn main() {
    let world = World::new();

    // Create a query that matches edible components
    let query = world.new_query::<&(EatsAmount, flecs::Wildcard)>();

    // Create a few entities that match the query
    world
        .entity_named("Bob")
        .set_pair::<EatsAmount, Apples>(EatsAmount { amount: 10 })
        .set_pair::<EatsAmount, Pears>(EatsAmount { amount: 5 });

    world
        .entity_named("Alice")
        .set_pair::<EatsAmount, Apples>(EatsAmount { amount: 4 });

    // Iterate the query with a flecs::iter. This makes it possible to inspect
    // the pair that we are currently matched with.
    query.each_iter(|it, index, eats| {
        let entity = it.entity(index).unwrap();
        let pair = it.pair(0).unwrap();
        let food = pair.second_id();

        println!("{} eats {} {}", entity, eats.amount, food);
    });

    // Output:
    //  Alice eats 4 Apples
    //  Bob eats 10 Apples
    //  Bob eats 5 Pears
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_wildcard".to_string());
}
