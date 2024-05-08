#![allow(dead_code)]
use flecs_ecs::core::*;
use flecs_ecs::macros::*;

#[test]
fn query_uncached_destruction_no_panic() {
    #[derive(Component)]
    struct Tag;

    let world = World::new();
    let query = world.new_query::<&Tag>();
    let query2 = query.clone();
    drop(query);
    query2.iter_only(|_| {});
    drop(query2);
}

#[test]
#[should_panic]
fn query_cached_destruction_lingering_references_panic() {
    #[derive(Component)]
    struct Tag;

    let world = World::new();
    let query = world.query::<&Tag>().set_cached().build();
    let query2 = query.clone();
    query.destruct();
    query2.iter_only(|_| {});
    drop(query2);
}
