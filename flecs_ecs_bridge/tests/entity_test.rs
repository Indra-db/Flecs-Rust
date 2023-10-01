use flecs_ecs_bridge::core::{c_types::*, world::World};
mod common;
use common::*;
struct Parent {
    entity_type: EntityType,
}

struct EntityType;

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
fn entity_get_generic_w_id() {
    let world = World::default();
    let position = world.add_component::<Position>();
    let entity = world
        .new_entity()
        .set_component(Position { x: 10.0, y: 20.0 });

    assert!(entity.is_valid());
    assert_eq!(entity.has::<Position>(), true);

    let pos_void = entity.get_component_by_id(position.raw_id);
    assert!(!pos_void.is_null());

    let pos = unsafe { &*(pos_void as *const Position) };
    assert_eq!(pos.x, 10.0);
    assert_eq!(pos.y, 20.0);
}

#[test]
fn entity_get_generic_w_id_t() {
    let world = World::default();
    let position = world.add_component::<Position>();
    let entity = world
        .new_entity()
        .set_component(Position { x: 10.0, y: 20.0 });

    assert!(entity.is_valid());
    assert_eq!(entity.has::<Position>(), true);

    let pos_void = entity.get_component_by_id(position.raw_id);
    assert!(!pos_void.is_null());

    let pos = unsafe { &*(pos_void as *const Position) };
    assert_eq!(pos.x, 10.0);
    assert_eq!(pos.y, 20.0);
}

#[test]
fn entity_set_generic() {
    let world = World::default();
    let position = world.add_component::<Position>();

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
    let position = world.add_component::<Position>();

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
    assert!(entity.has_flags_for_role(ECS_PAIR));

    let entity = entity.remove_flags();
    assert!(!entity.has_flags_for_role(ECS_PAIR));
}
