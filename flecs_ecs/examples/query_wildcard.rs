mod common;

use common::{Apples, Pears};
pub use flecs_ecs::{core::*, macros::Component};

#[derive(Component)]
pub struct Eats {
    pub amount: i32,
}

fn main() {
    let world = World::new();

    // Create a query that matches edible components
    let query = world
        .query_builder::<(&Eats,)>()
        .term_at(1)
        .select_second_id(ECS_WILDCARD) // Change first argument to (Eats, *)
        .build();

    // Create a few entities that match the query
    world
        .new_entity_named(c"Bob")
        .set_pair_first::<Eats, Apples>(Eats { amount: 10 })
        .set_pair_first::<Eats, Pears>(Eats { amount: 5 });

    world
        .new_entity_named(c"Alice")
        .set_pair_first::<Eats, Apples>(Eats { amount: 4 });

    // Iterate the query with a flecs::iter. This makes it possible to inspect
    // the pair that we are currently matched with.
    query.each_iter(|it, index, (eats,)| {
        let entity = it.get_entity(index);
        let food = it.get_pair_id(1).unwrap().second();

        println!("{} eats {} {}", entity, eats.amount, food);
    });

    // Output:
    //  Alice eats 4 Apples
    //  Bob eats 10 Apples
    //  Bob eats 5 Pears
}
