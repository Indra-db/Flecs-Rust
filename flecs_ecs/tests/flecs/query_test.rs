#![allow(dead_code)]
use flecs_ecs::core::*;
use flecs_ecs::macros::*;

use crate::common_test::*;

#[test]
fn query_uncached_destruction_no_panic() {
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

#[test]
#[should_panic]
fn query_panic_inside() {
    let world = World::new();
    let query = world.query::<&Tag>().build();
    query.run(|_| {
        panic!();
    });
}

#[test]
fn query_run_sparse() {
    let world = World::new();

    world.component::<Position>().add_trait::<flecs::Sparse>();
    world.component::<Velocity>();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.query::<(&mut Position, &Velocity)>().build();

    q.run(|mut it| {
        while it.next() {
            let v = it.field::<Velocity>(1).unwrap();

            for i in it.iter() {
                let p = it.field_at_mut::<Position>(0, i).unwrap();
                p.x += v[i].x;
                p.y += v[i].y;
            }
        }
    });

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn query_each_sparse() {
    let world = World::new();

    world.component::<Position>().add_trait::<flecs::Sparse>();
    world.component::<Velocity>();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.query::<(&mut Position, &Velocity)>().build();

    q.each(|(p, v)| {
        p.x += v.x;
        p.y += v.y;
    });

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}
