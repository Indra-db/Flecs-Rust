#![allow(dead_code)]
#![allow(unused_imports)]
use crate::common_test::*;
use flecs_ecs::prelude::*;

#[test]
fn ordered_children_iter_no_children() {
    let world = World::new();

    let parent = world.entity().add(flecs::OrderedChildren);

    let mut count = 0;
    parent.each_child(|_| {
        count += 1;
    });

    assert_eq!(count, 0);
}

#[test]
fn ordered_children_children_1_table() {
    let world = World::new();

    let parent = world.entity().add(flecs::OrderedChildren);
    let child_a = world.entity().child_of(parent).set(Position { x: 0, y: 0 });
    let child_b = world.entity().child_of(parent).set(Position { x: 0, y: 0 });
    let child_c = world.entity().child_of(parent).set(Position { x: 0, y: 0 });

    let mut v: Vec<Entity> = Vec::new();
    parent.each_child(|e| {
        v.push(e.id());
    });

    assert_eq!(v[0], child_a.id());
    assert_eq!(v[1], child_b.id());
    assert_eq!(v[2], child_c.id());
}

#[test]
fn ordered_children_children_2_tables() {
    let world = World::new();

    let parent = world.entity().add(flecs::OrderedChildren);
    let child_a = world.entity().child_of(parent).set(Position { x: 0, y: 0 });
    let child_b = world.entity().child_of(parent).set(Velocity { x: 0, y: 0 });
    let child_c = world.entity().child_of(parent).set(Position { x: 0, y: 0 });

    let mut v: Vec<Entity> = Vec::new();
    parent.each_child(|e| {
        v.push(e.id());
    });

    assert_eq!(v[0], child_a.id());
    assert_eq!(v[1], child_b.id());
    assert_eq!(v[2], child_c.id());
}

#[test]
fn ordered_children_set_child_order() {
    let world = World::new();

    let parent = world.entity().add(flecs::OrderedChildren);
    let child_a = world.entity().child_of(parent).set(Position { x: 0, y: 0 });
    let child_b = world.entity().child_of(parent).set(Position { x: 0, y: 0 });
    let child_c = world.entity().child_of(parent).set(Position { x: 0, y: 0 });

    {
        let mut v: Vec<Entity> = Vec::new();
        parent.each_child(|e| {
            v.push(e.id());
        });
        assert_eq!(v[0], child_a.id());
        assert_eq!(v[1], child_b.id());
        assert_eq!(v[2], child_c.id());
    }

    let entities = vec![child_c.id(), child_a.id(), child_b.id()];
    parent.set_child_order(&entities);

    {
        let mut v: Vec<Entity> = Vec::new();
        parent.each_child(|e| {
            v.push(e.id());
        });
        assert_eq!(v[0], child_c.id());
        assert_eq!(v[1], child_a.id());
        assert_eq!(v[2], child_b.id());
    }
}
