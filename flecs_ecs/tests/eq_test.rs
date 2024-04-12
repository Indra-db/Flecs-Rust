mod common;
use common::*;
use flecs_ecs::core::*;

/// test for compilation errors, no forgotten implementation
#[test]
fn entity_eq_test() {
    let world = World::new();

    let a_u64: u64 = 1;

    let e1_entity_view = world.new_entity();
    let e1_id_view = e1_entity_view.id_view();
    let e1_entity = e1_entity_view.id();
    let e1_id: Id = e1_entity.into();

    let comp1 = world.component::<Position>();
    let comp_untyped1 = world.component_untyped::<Position>();

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
