use flecs_ecs_bridge::core::{
    c_binding::bindings::EcsComponent, c_types::*, entity::Entity, world::World,
};
mod common;
use common::*;
//struct Parent {
//    entity_type: EntityType,
//}
//
//struct EntityType;

#[test]
fn entity_new() {
    let world = World::default();
    let entity = world.new_entity();
    assert!(entity.is_valid());
}

#[test]
fn entity_new_named() {
    let world = World::default();
    let entity = world.new_entity_named("test");
    assert!(entity.is_valid());
    assert_eq!(entity.get_name(), "test");
}

#[test]
fn entity_new_named_from_scope() {
    let world = World::default();
    let entity = world.new_entity_named("Foo");
    assert!(entity.is_valid());

    let prev = world.set_scope_id(entity.raw_id);
    let child = world.new_entity_named("Bar");
    assert_eq!(child.is_valid(), true);

    world.set_scope_id(prev.raw_id);

    assert_eq!(child.get_name(), "Bar");
    assert_eq!(child.get_hierachy_path_default().unwrap(), "::Foo::Bar");
}

#[test]
fn entity_new_nested_named_from_nested_scope() {
    // Create a world
    let world = World::default();

    // Create an entity with nested name "Foo::Bar"
    let entity = world.new_entity_named("Foo::Bar");

    // Verify that the entity exists and its name and path are correct
    assert!(entity.is_valid());
    assert_eq!(entity.get_name(), "Bar");
    assert_eq!(entity.get_hierachy_path_default().unwrap(), "::Foo::Bar");

    // Set the current scope to `entity`
    let prev = world.set_scope_id(entity.raw_id);

    // Create a child entity with nested name "Hello::World" under the current scope
    let child = world.new_entity_named("Hello::World");

    // Verify that the child entity exists
    assert_eq!(child.is_valid(), true);

    // Restore the previous scope
    world.set_scope_id(prev.raw_id);

    // Verify the name and hierarchical path of the child entity
    assert_eq!(child.get_name(), "World");
    assert_eq!(
        child.get_hierachy_path_default().unwrap(),
        "::Foo::Bar::Hello::World"
    );
}

#[test]
fn entity_new_add() {
    // Create a world
    let world = World::default();

    // Create an entity and add the Position component to it
    let entity = world.new_entity().add_component::<Position>();

    // Verify that the entity exists
    assert!(entity.is_valid());

    // Verify that the entity has the Position component
    assert_eq!(entity.has::<Position>(), true);
}

#[test]
fn entity_new_add_2() {
    // Create a world
    let world = World::default();

    // Create an entity and add the Position and Velocity components to it
    let entity = world
        .new_entity()
        .add_component::<Position>()
        .add_component::<Velocity>();

    // Verify that the entity exists
    assert!(entity.is_valid());

    // Verify that the entity has the Position component
    assert_eq!(entity.has::<Position>(), true);

    // Verify that the entity has the Velocity component
    assert_eq!(entity.has::<Velocity>(), true);
}

#[test]
fn entity_new_set() {
    // Create a world
    let world = World::default();

    // Create an entity and set the Position component data
    let entity = world
        .new_entity()
        .set_component(Position { x: 10.0, y: 20.0 });

    // Verify that the entity exists
    assert!(entity.is_valid());

    // Verify that the entity has the Position component
    assert_eq!(entity.has::<Position>(), true);

    // Verify the component data
    let p = entity.get_component::<Position>();
    unsafe {
        assert_eq!((*p).x, 10.0);
        assert_eq!((*p).y, 20.0);
    }
}

#[test]
fn entity_new_set_2() {
    let world = World::default();

    let entity = world
        .new_entity()
        .set_component(Position { x: 10.0, y: 20.0 })
        .set_component(Velocity { x: 1.0, y: 2.0 });

    assert!(entity.is_valid());
    assert_eq!(entity.has::<Position>(), true);
    assert_eq!(entity.has::<Velocity>(), true);

    let p = entity.get_component::<Position>();
    unsafe {
        assert_eq!((*p).x, 10.0);
        assert_eq!((*p).y, 20.0);
    }

    let v = entity.get_component::<Velocity>();
    unsafe {
        assert_eq!((*v).x, 1.0);
        assert_eq!((*v).y, 2.0);
    }
}

#[test]
fn entity_add() {
    let world = World::default();

    let entity = world.new_entity();

    assert!(entity.is_valid());

    entity.add_component::<Position>();

    assert_eq!(entity.has::<Position>(), true);
}

#[test]
fn entity_remove() {
    let world = World::default();

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_component::<Position>();
    assert_eq!(entity.has::<Position>(), true);

    entity.remove_component::<Position>();
    assert_eq!(entity.has::<Position>(), false);
}

#[test]
fn entity_set() {
    let world = World::default();

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.set_component(Position { x: 10.0, y: 20.0 });
    assert_eq!(entity.has::<Position>(), true);

    let p = entity.get_component::<Position>();
    unsafe {
        assert_eq!((*p).x, 10.0);
        assert_eq!((*p).y, 20.0);
    }
}

#[test]
fn entity_add_2() {
    let world = World::default();

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity
        .add_component::<Position>()
        .add_component::<Velocity>();

    assert_eq!(entity.has::<Position>(), true);
    assert_eq!(entity.has::<Velocity>(), true);
}

#[test]
fn entity_add_entity() {
    let world = World::default();

    let tag = world.new_entity();
    assert_eq!(tag.is_valid(), true);

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_id(tag.raw_id);
    assert_eq!(entity.has_id(tag.raw_id), true);
}

#[test]
fn entity_add_childof() {
    let world = World::default();

    let parent = world.new_entity();
    assert_eq!(parent.is_valid(), true);

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_pair_ids(ECS_CHILD_OF, parent.raw_id);
    assert_eq!(entity.has_pair_by_ids(ECS_CHILD_OF, parent.raw_id), true);
}

#[test]
fn entity_add_instanceof() {
    let world = World::default();

    let base = world.new_entity();
    assert_eq!(base.is_valid(), true);

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_pair_ids(ECS_IS_A, base.raw_id);
    assert_eq!(entity.has_pair_by_ids(ECS_IS_A, base.raw_id), true);
}

#[test]
fn entity_remove_2() {
    let world = World::default();

    let entity = world
        .new_entity()
        .add_component::<Position>()
        .add_component::<Velocity>();

    assert_eq!(entity.has::<Position>(), true);
    assert_eq!(entity.has::<Velocity>(), true);

    entity
        .remove_component::<Position>()
        .remove_component::<Velocity>();

    assert_eq!(entity.has::<Position>(), false);
    assert_eq!(entity.has::<Velocity>(), false);
}

#[test]
fn entity_set_2() {
    let world = World::default();

    let entity = world
        .new_entity()
        .set_component::<Position>(Position { x: 10.0, y: 20.0 })
        .set_component::<Velocity>(Velocity { x: 1.0, y: 2.0 });

    assert_eq!(entity.has::<Position>(), true);
    assert_eq!(entity.has::<Velocity>(), true);

    let p = entity.get_component::<Position>();
    unsafe {
        assert_eq!((*p).x, 10.0);
        assert_eq!((*p).y, 20.0);
    }

    let v = entity.get_component::<Velocity>();
    unsafe {
        assert_eq!((*v).x, 1.0);
        assert_eq!((*v).y, 2.0);
    }
}

#[test]
fn entity_remove_entity() {
    let world = World::default();

    let tag = world.new_entity();
    assert_eq!(tag.is_valid(), true);

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_id(tag.raw_id);
    assert_eq!(entity.has_id(tag.raw_id), true);

    entity.remove_id(tag.raw_id);
    assert_eq!(entity.has_id(tag.raw_id), false);
}

#[test]
fn entity_remove_childof() {
    let world = World::default();

    let parent = world.new_entity();
    assert_eq!(parent.is_valid(), true);

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_pair_ids(ECS_CHILD_OF, parent.raw_id);
    assert_eq!(entity.has_pair_by_ids(ECS_CHILD_OF, parent.raw_id), true);

    entity.remove_pair_ids(ECS_CHILD_OF, parent.raw_id);
    assert_eq!(entity.has_pair_by_ids(ECS_CHILD_OF, parent.raw_id), false);
}

#[test]
fn entity_remove_instanceof() {
    let world = World::default();

    let base = world.new_entity();
    assert_eq!(base.is_valid(), true);

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_pair_ids(ECS_IS_A, base.raw_id);
    assert_eq!(entity.has_pair_by_ids(ECS_IS_A, base.raw_id), true);

    entity.remove_pair_ids(ECS_IS_A, base.raw_id);
    assert_eq!(entity.has_pair_by_ids(ECS_IS_A, base.raw_id), false);
}

#[test]
fn entity_get_generic() {
    let world = World::default();
    let position = world.add_component::<Position>();

    let entity = world
        .new_entity()
        .set_component(Position { x: 10.0, y: 20.0 });

    assert!(entity.is_valid());
    assert_eq!(entity.has::<Position>(), true);

    let pos_void = entity.get_component_by_id(position.raw_id);
    assert!(pos_void != std::ptr::null());

    let pos = unsafe { &*(pos_void as *const Position) };
    assert_eq!(pos.x, 10.0);
    assert_eq!(pos.y, 20.0);
}

#[test]
#[ignore]
fn entity_get_generic_mut() {
    todo!("observer and event needs to be implemented for this to work");
}

#[test]
#[ignore]
fn entity_get_mut_generic_w_id() {
    todo!("observer and event needs to be implemented for this to work");
}

#[test]
fn entity_set_generic() {
    let world = World::default();
    let position = world.component::<Position>();

    let pos = Position { x: 10.0, y: 20.0 };

    let entity = world.new_entity().set_ptr_w_size(
        position.raw_id,
        std::mem::size_of::<Position>(),
        &pos as *const _ as *const std::ffi::c_void,
    );

    assert!(entity.has::<Position>());
    assert!(entity.has_id(position.raw_id));

    let pos = unsafe { &*entity.get_component::<Position>() };
    assert_eq!(pos.x, 10.0);
    assert_eq!(pos.y, 20.0);
}

#[test]
fn entity_set_generic_no_size() {
    let world = World::default();
    let position = world.component::<Position>();

    let pos = Position { x: 10.0, y: 20.0 };

    let entity = world
        .new_entity()
        .set_ptr(position.raw_id, &pos as *const _ as *const std::ffi::c_void);

    assert!(entity.has::<Position>());
    assert!(entity.has_id(position.raw_id));

    let pos = unsafe { &*entity.get_component::<Position>() };
    assert_eq!(pos.x, 10.0);
    assert_eq!(pos.y, 20.0);
}

#[test]
fn entity_add_role() {
    let world = World::default();
    let entity = world.new_entity();

    let entity = entity.add_flags(ECS_PAIR);

    assert_eq!(entity.raw_id & ECS_PAIR, ECS_PAIR);
}

#[test]
fn entity_remove_role() {
    let world = World::default();
    let entity = world.new_entity();
    let id = entity.raw_id;

    let entity = entity.add_flags(ECS_PAIR);
    assert_eq!(entity.raw_id & ECS_PAIR, ECS_PAIR);

    let entity = entity.remove_flags();
    assert_eq!(entity.raw_id, id);
}

#[test]
fn entity_has_role() {
    let world = World::default();
    let entity = world.new_entity();

    let entity = entity.add_flags(ECS_PAIR);
    assert!(entity.has_flags_for(ECS_PAIR));

    let entity = entity.remove_flags();
    assert!(!entity.has_flags_for(ECS_PAIR));
}

#[test]
fn entity_pair_role() {
    let world = World::default();
    let entity = world.new_entity();
    let entity2 = world.new_entity();

    let mut pair = Entity::new_pair_only(entity.raw_id, entity2.raw_id);
    pair = pair.add_flags(ECS_PAIR);

    assert!(pair.has_flags_for(ECS_PAIR));

    let rel = pair.first();
    let obj = pair.second();

    assert_eq!(rel, entity);
    assert_eq!(obj, entity2);
}

#[test]
fn entity_equals() {
    let world = World::default();
    let e1 = world.new_entity();
    let e2 = world.new_entity();

    let e1_2 = e1.clone();
    let e2_2 = e2.clone();

    assert!(e1 == e1_2);
    assert!(e2 == e2_2);
    assert!(e1 >= e1_2);
    assert!(e1 <= e1_2);
    assert!(e2 >= e2_2);
    assert!(e2 <= e2_2);
    assert!(e1 != e2);

    assert!(!(e2 == e1_2));
    assert!(!(e1 == e2_2));
    assert!(!(e2 <= e1_2));
    assert!(!(e1 >= e2_2));
    assert!(!(e2 != e2));
}

#[test]
fn entity_compare_0() {
    let world = World::default();
    let e = world.new_entity();
    let e0 = world.new_entity_w_id(0);
    let e0_2 = world.new_entity_w_id(0);

    assert!(e != e0);
    assert!(e > e0);
    assert!(e >= e0);
    assert!(e0 < e);
    assert!(e0 <= e);

    assert!(e0 == e0_2);
    assert!(e0 >= e0_2);
    assert!(e0 <= e0_2);
}

#[test]
fn entity_compare_literal() {
    let world = World::default();

    let e1 = world.new_entity_w_id(500);
    let e2 = world.new_entity_w_id(600);

    assert_eq!(e1.raw_id, 500);
    assert_eq!(e2.raw_id, 600);

    assert_ne!(e1.raw_id, 600);
    assert_ne!(e2.raw_id, 500);

    assert!(e1.raw_id >= 500);
    assert!(e2.raw_id >= 600);

    assert!(e1.raw_id <= 500);
    assert!(e2.raw_id <= 600);

    assert!(e1.raw_id <= 600);
    assert!(e2.raw_id >= 500);

    assert!(e1.raw_id < 600);
    assert!(e2.raw_id > 500);

    assert!(e2.raw_id != 500);
    assert!(e1.raw_id != 600);

    assert!(e2.raw_id == 600);
    assert!(e1.raw_id == 500);

    assert!(e1.raw_id < 600);
    assert!(e2.raw_id > 500);
}

#[test]
fn entity_greater_than() {
    let world = World::default();

    let e1 = world.new_entity();
    let e2 = world.new_entity();

    assert!(e2 > e1);
    assert!(e2 >= e1);
}

#[test]
fn entity_less_than() {
    let world = World::default();

    let e1 = world.new_entity();
    let e2 = world.new_entity();

    assert!(e1 < e2);
    assert!(e1 <= e2);
}

#[test]
fn entity_not_0_or_1() {
    let world = World::default();

    let e = world.new_entity();

    let id = e.raw_id;

    assert_ne!(id, 0);
    assert_ne!(id, 1);
}

#[test]
fn entity_has_childof() {
    let world = World::default();

    let parent = world.new_entity();

    let child = world.new_entity().add_pair_ids(ECS_CHILD_OF, parent.raw_id);

    assert!(child.has_pair_by_ids(ECS_CHILD_OF, parent.raw_id));
}

#[test]
fn entity_has_instanceof() {
    let world = World::default();

    let base = world.new_entity();

    let instance = world.new_entity().add_pair_ids(ECS_IS_A, base.raw_id);

    assert!(instance.has_pair_by_ids(ECS_IS_A, base.raw_id));
}

#[test]
fn entity_has_instanceof_indirect() {
    let world = World::default();

    let base_of_base = world.new_entity();
    let base = world
        .new_entity()
        .add_pair_ids(ECS_IS_A, base_of_base.raw_id);

    let instance = world.new_entity().add_pair_ids(ECS_IS_A, base.raw_id);

    assert!(instance.has_pair_by_ids(ECS_IS_A, base_of_base.raw_id));
}

#[test]
fn entity_null_string() {
    let world = World::default();

    let entity = world.new_entity();

    assert_eq!(entity.get_name(), "");
}

#[test]
fn entity_none_string() {
    let world = World::default();

    let entity = world.new_entity();

    assert_eq!(entity.get_name_optional(), None);
}

#[test]
fn entity_set_name() {
    let world = World::default();

    let entity = world.new_entity();

    entity.set_name("Foo");

    assert_eq!(entity.get_name(), "Foo");
}

#[test]
fn entity_set_name_optional() {
    let world = World::default();

    let entity = world.new_entity();

    entity.set_name("Foo");

    assert_eq!(entity.get_name_optional(), Some("Foo"));
}

#[test]
fn entity_change_name() {
    let world = World::default();

    let entity = world.new_entity_named("Bar");
    assert_eq!(entity.get_name(), "Bar");

    entity.set_name("Foo");
    assert_eq!(entity.get_name(), "Foo");

    entity.set_name("Bar");
    assert_eq!(entity.get_name(), "Bar");
}

#[test]
fn entity_delete() {
    let world = World::default();

    let entity = world
        .new_entity()
        .add_component::<Position>()
        .add_component::<Velocity>();

    entity.destruct();
    assert!(!entity.is_alive());

    let entity2 = world.new_entity();
    assert_eq!(entity2.raw_id as u32, entity.raw_id as u32);
    assert_ne!(entity2, entity);
}

#[test]
fn entity_clear() {
    let world = World::default();

    let entity = world
        .new_entity()
        .add_component::<Position>()
        .add_component::<Velocity>();

    entity.clear();
    assert!(!entity.has::<Position>());
    assert!(!entity.has::<Velocity>());

    let entity2 = world.new_entity();
    assert!(entity2 > entity);
}

#[test]
#[ignore]
fn entity_force_owned() {
    todo!("prefab not get implemented");
}

#[test]
#[ignore]
fn entity_force_owned_2() {
    todo!("prefab not get implemented");
}

#[test]
#[ignore]
fn entity_force_owned_nested() {
    todo!("prefab not get implemented");
}

#[test]
fn entity_tag_has_size_zero() {
    let world = World::default();

    world.component::<EcsComponent>();
    let comp = world.component::<MyTag>();
    //let ptr: *const EcsComponent = comp.get_component_by_id(unsafe {
    //    flecs_ecs_bridge::core::c_binding::bindings::FLECS__EEcsComponent
    //}) as *const _ as *const EcsComponent;
    let ptr = comp.get_component::<Component>();
    assert_eq!(unsafe { (*ptr).size }, 0);
    assert_eq!(unsafe { (*ptr).alignment }, 0);
}
