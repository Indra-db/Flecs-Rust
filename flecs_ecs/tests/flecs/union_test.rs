#![allow(dead_code)]
#![allow(unused_imports)]
use crate::common_test::*;
use flecs_ecs::prelude::*;

#[test]
fn union_add_case() {
    let world = World::new();

    let standing = world.entity_named("Standing");
    let walking = world.entity_named("Walking");
    let movement = world
        .entity()
        .add(flecs::DontFragment)
        .add(flecs::Exclusive);

    let e = world.entity().add((movement, standing));
    assert!(e.has((movement, standing)));

    let table = e.table();

    e.add((movement, walking));
    assert_eq!(e.table(), table);

    assert!(e.has((movement, walking)));
    assert!(!e.has((movement, standing)));
}

#[test]
fn union_get_case() {
    let world = World::new();

    let standing = world.entity_named("Standing");
    world.entity_named("Walking");
    let movement = world
        .entity()
        .add(flecs::DontFragment)
        .add(flecs::Exclusive);

    let e = world.entity().add((movement, standing));
    assert!(e.has((movement, standing)));

    assert_eq!(e.target(movement, 0).unwrap().id(), standing.id());
}

#[derive(Component)]
struct Movement;
#[derive(Component)]
struct Standing;
#[derive(Component)]
struct Walking;

#[test]
fn union_add_case_w_type() {
    let world = World::new();

    world
        .component::<Movement>()
        .add_trait::<flecs::DontFragment>()
        .add_trait::<flecs::Exclusive>();

    let e = world.entity().add((Movement::id(), Standing::id()));
    assert!(e.has((Movement::id(), Standing::id())));

    e.add((Movement::id(), Walking::id()));

    assert!(e.has((Movement::id(), Walking::id())));
    assert!(!e.has((Movement::id(), Standing::id())));
}

#[test]
fn union_add_switch_w_type() {
    let world = World::new();

    world
        .component::<Movement>()
        .add_trait::<flecs::DontFragment>()
        .add_trait::<flecs::Exclusive>();

    let e = world.entity().add((Movement::id(), Standing::id()));
    assert!(e.has((Movement::id(), Standing::id())));

    e.add((Movement::id(), Walking::id()));

    assert!(e.has((Movement::id(), Walking::id())));
    assert!(!e.has((Movement::id(), Standing::id())));
}

#[test]
fn union_add_remove_switch_w_type() {
    let world = World::new();

    world
        .component::<Movement>()
        .add_trait::<flecs::DontFragment>()
        .add_trait::<flecs::Exclusive>();

    let e = world.entity().add((Movement::id(), Standing::id()));
    assert!(e.has((Movement::id(), *flecs::Wildcard)));
    assert!(e.has((Movement::id(), Standing::id())));

    let table = e.table();

    e.add((Movement::id(), Walking::id()));

    assert!(e.has((Movement::id(), Walking::id())));
    assert!(!e.has((Movement::id(), Standing::id())));
    assert_eq!(e.table(), table);

    let c = e.target(Movement::id(), 0).unwrap();
    assert_ne!(c, 0u64);
    assert_eq!(c.id(), world.component_id::<Walking>());

    e.remove((Movement::id(), *flecs::Wildcard));
    assert!(!e.has((Movement::id(), *flecs::Wildcard)));
    assert!(!e.has((Movement::id(), Walking::id())));
    assert_eq!(e.table(), table);
}

#[derive(Component, PartialEq, Debug)]
#[repr(C)]
enum Color {
    Red,
    Green,
    Blue,
}

#[test]
fn union_switch_enum_type() {
    let world = World::new();

    world
        .component::<Color>()
        .add_trait::<flecs::DontFragment>()
        .add_trait::<flecs::Exclusive>();

    let e = world.entity().add_enum(Color::Red);
    assert!(e.has_enum(Color::Red));
    assert!(!e.has_enum(Color::Green));
    assert!(!e.has_enum(Color::Blue));
    assert!(e.has((world.component_id::<Color>(), *flecs::Wildcard)));

    let table = e.table();

    e.add_enum(Color::Green);
    assert!(!e.has_enum(Color::Red));
    assert!(e.has_enum(Color::Green));
    assert!(!e.has_enum(Color::Blue));
    assert!(e.has((world.component_id::<Color>(), *flecs::Wildcard)));
    assert_eq!(e.table(), table);

    e.add_enum(Color::Blue);
    assert!(!e.has_enum(Color::Red));
    assert!(!e.has_enum(Color::Green));
    assert!(e.has_enum(Color::Blue));
    assert!(e.has((world.component_id::<Color>(), *flecs::Wildcard)));
    assert_eq!(e.table(), table);

    e.remove((world.component_id::<Color>(), *flecs::Wildcard));
    assert!(!e.has_enum(Color::Red));
    assert!(!e.has_enum(Color::Green));
    assert!(!e.has_enum(Color::Blue));
    assert!(!e.has((world.component_id::<Color>(), *flecs::Wildcard)));
    assert_eq!(e.table(), table);
}
