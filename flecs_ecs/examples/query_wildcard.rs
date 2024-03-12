mod common;
use std::ffi::CStr;

use common::{Apples, Pears};
pub use flecs_ecs::{core::*, macros::Component};

#[derive(Default, Clone, Component)]
pub struct Eats {
    pub value: i32,
}

fn main() {
    let world = World::new();

    // Create a query that matches edible components
    let _query = world
        .query_builder::<(&Eats,)>()
        .term_at(1)
        .select_second_id(ECS_WILDCARD) // Change first argument to (Eats, *)
        .build();

    // Create a few entities that match the query
    world
        .new_entity_named(CStr::from_bytes_with_nul(b"Bob\0").unwrap())
        .set_pair_first::<Eats, Apples>(Eats { value: 10 })
        .set_pair_first::<Eats, Pears>(Eats { value: 5 });

    world
        .new_entity_named(CStr::from_bytes_with_nul(b"Alice\0").unwrap())
        .set_pair_first::<Eats, Apples>(Eats { value: 4 });

    todo!("`.each` signature with it, index and Eats not yet supported in flecs_ecs");
}
