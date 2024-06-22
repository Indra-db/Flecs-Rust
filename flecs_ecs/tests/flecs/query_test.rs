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
    query2.run(|mut it| {
        dbg!(it.iter_mut().flags & flecs_ecs::sys::EcsIterIsValid != 0);
        while it.next() {}
        dbg!(it.iter_mut().flags & flecs_ecs::sys::EcsIterIsValid != 0);
    });
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
    query2.run(|_| {});
    drop(query2);
}

#[test]
fn query_iter_stage() {
    #[derive(Component, Debug)]
    struct Comp(usize);

    let world = World::new();
    world.set_threads(4);

    let query = world.new_query::<&Comp>();

    for i in 0..4 {
        world.entity().set(Comp(i));
    }

    world
        .system::<&Comp>()
        .multi_threaded()
        .each_entity(move |e, _| {
            query.iter_stage(e).each(|_vel| {});
        });

    world.progress();
}
