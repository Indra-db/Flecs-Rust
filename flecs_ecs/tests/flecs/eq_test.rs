#![allow(dead_code)]
use crate::common_test::*;

/// test for compilation errors, no forgotten implementation
#[test]
fn entity_eq_test() {
    let world = World::new();

    let a_u64: u64 = 1;

    let e1_entity_view = world.entity();
    let e1_id_view = e1_entity_view.id_view();
    let e1_entity = e1_entity_view.id();
    let e1_id: Id = e1_entity.into();

    let comp1 = world.component::<Position>();
    let comp_untyped1 = world.component_untyped_from(Position::id());

    assert_eq!(e1_id, e1_id);
    assert_eq!(e1_id, e1_entity);
    assert_eq!(e1_id, e1_id_view);
    assert_eq!(e1_id, e1_entity_view);

    assert_eq!(e1_entity, e1_id);
    assert_eq!(e1_entity, e1_entity);
    assert_eq!(e1_entity, e1_id_view);
    assert_eq!(e1_entity, e1_entity_view);

    assert_eq!(e1_id_view, e1_id);
    assert_eq!(e1_id_view, e1_entity);
    assert_eq!(e1_id_view, e1_id_view);
    assert_eq!(e1_id_view, e1_entity_view);

    assert_eq!(e1_entity_view, e1_id);
    assert_eq!(e1_entity_view, e1_entity);
    assert_eq!(e1_entity_view, e1_id_view);
    assert_eq!(e1_entity_view, e1_entity_view);

    let test_equality1 = e1_id == comp1;
    let test_equality2 = e1_entity == comp1;
    let test_equality3 = e1_id_view == comp1;
    let test_equality4 = e1_entity_view == comp1;

    assert!(!test_equality1);
    assert!(!test_equality2);
    assert!(!test_equality3);
    assert!(!test_equality4);

    let test_equality5 = comp1 == e1_id;
    let test_equality6 = comp1 == e1_entity;
    let test_equality7 = comp1 == e1_id_view;
    let test_equality8 = comp1 == e1_entity_view;

    assert!(!test_equality5);
    assert!(!test_equality6);
    assert!(!test_equality7);
    assert!(!test_equality8);

    let test_equality9 = e1_id == comp_untyped1;
    let test_equality10 = e1_entity == comp_untyped1;
    let test_equality11 = e1_id_view == comp_untyped1;
    let test_equality12 = e1_entity_view == comp_untyped1;

    assert!(!test_equality9);
    assert!(!test_equality10);
    assert!(!test_equality11);
    assert!(!test_equality12);

    let test_equality13 = comp_untyped1 == e1_id;
    let test_equality14 = comp_untyped1 == e1_entity;
    let test_equality15 = comp_untyped1 == e1_id_view;
    let test_equality16 = comp_untyped1 == e1_entity_view;

    assert!(!test_equality13);
    assert!(!test_equality14);
    assert!(!test_equality15);
    assert!(!test_equality16);

    let test_equality18 = comp1 == comp1;
    let test_equality17 = comp1 == comp_untyped1;
    let test_equality19 = comp_untyped1 == comp1;
    let test_equality20 = comp_untyped1 == comp_untyped1;

    assert!(test_equality17);
    assert!(test_equality18);
    assert!(test_equality19);
    assert!(test_equality20);

    let test_equality21 = e1_id == a_u64;
    let test_equality22 = e1_entity == a_u64;
    let test_equality23 = e1_id_view == a_u64;
    let test_equality24 = e1_entity_view == a_u64;
    let test_equality25 = comp1 == a_u64;
    let test_equality26 = comp_untyped1 == a_u64;

    assert!(!test_equality21);
    assert!(!test_equality22);
    assert!(!test_equality23);
    assert!(!test_equality24);
    assert!(!test_equality25);
    assert!(!test_equality26);

    let test_equality27 = a_u64 == e1_id;
    let test_equality28 = a_u64 == e1_entity;
    let test_equality29 = a_u64 == e1_id_view;
    let test_equality30 = a_u64 == e1_entity_view;
    let test_equality31 = a_u64 == comp1;
    let test_equality32 = a_u64 == comp_untyped1;

    assert!(!test_equality27);
    assert!(!test_equality28);
    assert!(!test_equality29);
    assert!(!test_equality30);
    assert!(!test_equality31);
    assert!(!test_equality32);
}

#[test]
fn entity_view_and_id_view_hash_test() {
    use std::collections::{HashMap, HashSet};

    let world = World::new();

    let e1 = world.entity();
    let e2 = world.entity();

    let mut set: HashSet<EntityView<'_>> = HashSet::new();
    set.insert(e1);
    set.insert(e1);
    set.insert(e2);
    assert_eq!(set.len(), 2);
    assert!(set.contains(&e1));
    assert!(set.contains(&e2));

    let mut map: HashMap<EntityView<'_>, i32> = HashMap::new();
    map.insert(e1, 10);
    map.insert(e1, 20);
    assert_eq!(map.len(), 1);
    assert_eq!(map[&e1], 20);

    let id1 = e1.id_view();
    let id2 = e2.id_view();

    let mut id_set: HashSet<IdView<'_>> = HashSet::new();
    id_set.insert(id1);
    id_set.insert(id1);
    id_set.insert(id2);
    assert_eq!(id_set.len(), 2);
    assert!(id_set.contains(&id1));
    assert!(id_set.contains(&id2));
}

#[test]
fn newtype_of_entity_view_derives_eq_ord_hash_test() {
    use std::collections::HashSet;

    flecs_ecs::newtype_of_entity_view!(pub struct MyEntityViewHashTest(EntityView));

    let world = World::new();
    let e1 = world.entity();
    let e2 = world.entity();

    let w1 = MyEntityViewHashTest::new(e1);
    let w1_dup = MyEntityViewHashTest::new(e1);
    let w2 = MyEntityViewHashTest::new(e2);

    assert_eq!(w1, w1_dup);
    assert_ne!(w1, w2);
    assert!(w1 < w2 || w2 < w1);

    let mut set = HashSet::new();
    set.insert(w1);
    set.insert(w1_dup);
    set.insert(w2);
    assert_eq!(set.len(), 2);
}

#[test]
fn newtype_of_entity_derives_eq_ord_hash_test() {
    use std::collections::HashSet;

    flecs_ecs::newtype_of_entity!(pub struct MyEntityHashTest(pub Entity));

    let e1 = Entity::new(100);
    let e1_dup = Entity::new(100);
    let e2 = Entity::new(200);

    let w1 = MyEntityHashTest(e1);
    let w1_dup = MyEntityHashTest(e1_dup);
    let w2 = MyEntityHashTest(e2);

    assert_eq!(w1, w1_dup);
    assert_ne!(w1, w2);
    assert!(w1 < w2);

    let mut set = HashSet::new();
    set.insert(w1);
    set.insert(w1_dup);
    set.insert(w2);
    assert_eq!(set.len(), 2);
}

#[test]
fn id_view_try_first_second_id_matches_get_variant_test() {
    let world = World::new();

    let rel = world.entity();
    let target = world.entity();
    let pair_id = world.id_view_from((rel, target));

    assert_eq!(pair_id.try_first_id(), pair_id.get_first_id());
    assert_eq!(pair_id.try_second_id(), pair_id.get_second_id());
    assert_eq!(pair_id.try_first_id().unwrap(), pair_id.first_id());
    assert_eq!(pair_id.try_second_id().unwrap(), pair_id.second_id());

    let non_pair_id = world.id_view_from(rel);
    assert_eq!(non_pair_id.try_first_id(), None);
    assert_eq!(non_pair_id.try_second_id(), None);
    assert_eq!(non_pair_id.get_first_id(), None);
    assert_eq!(non_pair_id.get_second_id(), None);
}

#[test]
fn table_eq_test() {
    let world = World::new();

    let ent1 = world.entity().add(Position::id());
    let ent2 = world.entity().add(Position::id());
    let ent3 = world.entity().add(Position::id()).add(Velocity::id());

    let table1 = ent1.table().unwrap();
    let table2 = ent2.table().unwrap();
    let table3 = ent3.table().unwrap();

    assert_eq!(table1, table2);
    assert_ne!(table1, table3);

    ent1.add(Velocity::id());
    let table1 = ent1.table().unwrap();

    assert_ne!(table1, table2);
    assert_eq!(table1, table3);
}
