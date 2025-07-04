#![allow(dead_code)]

use flecs_ecs::core::*;

use crate::common_test::*;

#[test]
#[should_panic]
fn observer_panic_on_add_1() {
    let world = World::new();

    world
        .observer::<flecs::OnAdd, ()>()
        .with(&Position::id())
        .each_entity(|_, _| {});

    world.entity().set(Position { x: 10, y: 20 });
}

#[test]
#[should_panic]
fn observer_panic_on_add_2() {
    let world = World::new();

    world
        .observer::<flecs::OnAdd, ()>()
        .with(&mut Position::id())
        .each_entity(|_, _| {});

    world.entity().set(Position { x: 10, y: 20 });
}

#[test]
#[should_panic]
fn observer_panic_on_add_3() {
    let world = World::new();

    world
        .observer::<flecs::OnAdd, &Position>()
        .each_entity(|_, _| {});

    world.entity().set(Position { x: 10, y: 20 });
}

#[test]
#[should_panic]
fn observer_panic_on_add_4() {
    let world = World::new();

    world
        .observer::<flecs::OnAdd, &mut Position>()
        .each_entity(|_, _| {});

    world.entity().set(Position { x: 10, y: 20 });
}
