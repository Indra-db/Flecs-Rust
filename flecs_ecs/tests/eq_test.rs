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

    assert_eq!(test_equality1, false);
    assert_eq!(test_equality2, false);
    assert_eq!(test_equality3, false);
    assert_eq!(test_equality4, false);

    let test_equality5 = comp1 == e1_id;
    let test_equality6 = comp1 == e1_entity;
    let test_equality7 = comp1 == e1_id_view;
    let test_equality8 = comp1 == e1_entity_view;

    assert_eq!(test_equality5, false);
    assert_eq!(test_equality6, false);
    assert_eq!(test_equality7, false);
    assert_eq!(test_equality8, false);

    let test_equality9 = e1_id == comp_untyped1;
    let test_equality10 = e1_entity == comp_untyped1;
    let test_equality11 = e1_id_view == comp_untyped1;
    let test_equality12 = e1_entity_view == comp_untyped1;

    assert_eq!(test_equality9, false);
    assert_eq!(test_equality10, false);
    assert_eq!(test_equality11, false);
    assert_eq!(test_equality12, false);

    let test_equality13 = comp_untyped1 == e1_id;
    let test_equality14 = comp_untyped1 == e1_entity;
    let test_equality15 = comp_untyped1 == e1_id_view;
    let test_equality16 = comp_untyped1 == e1_entity_view;

    assert_eq!(test_equality13, false);
    assert_eq!(test_equality14, false);
    assert_eq!(test_equality15, false);
    assert_eq!(test_equality16, false);

    let test_equality18 = comp1 == comp1;
    let test_equality17 = comp1 == comp_untyped1;
    let test_equality19 = comp_untyped1 == comp1;
    let test_equality20 = comp_untyped1 == comp_untyped1;

    assert_eq!(test_equality17, true);
    assert_eq!(test_equality18, true);
    assert_eq!(test_equality19, true);
    assert_eq!(test_equality20, true);

    let test_equality21 = e1_id == a_u64;
    let test_equality22 = e1_entity == a_u64;
    let test_equality23 = e1_id_view == a_u64;
    let test_equality24 = e1_entity_view == a_u64;
    let test_equality25 = comp1 == a_u64;
    let test_equality26 = comp_untyped1 == a_u64;

    assert_eq!(test_equality21, false);
    assert_eq!(test_equality22, false);
    assert_eq!(test_equality23, false);
    assert_eq!(test_equality24, false);
    assert_eq!(test_equality25, false);
    assert_eq!(test_equality26, false);

    let test_equality27 = a_u64 == e1_id;
    let test_equality28 = a_u64 == e1_entity;
    let test_equality29 = a_u64 == e1_id_view;
    let test_equality30 = a_u64 == e1_entity_view;
    let test_equality31 = a_u64 == comp1;
    let test_equality32 = a_u64 == comp_untyped1;

    assert_eq!(test_equality27, false);
    assert_eq!(test_equality28, false);
    assert_eq!(test_equality29, false);
    assert_eq!(test_equality30, false);
    assert_eq!(test_equality31, false);
    assert_eq!(test_equality32, false);
}
