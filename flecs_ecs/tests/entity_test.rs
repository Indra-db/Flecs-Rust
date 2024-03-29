use std::ffi::{c_void, CStr};

use flecs_ecs::{
    core::{c_types::*, id::Id, world::World},
    sys::EcsComponent,
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
    let world = World::new();
    let entity = world.new_entity();
    assert!(entity.is_valid());
}

#[test]
fn entity_new_named() {
    let world = World::new();
    let entity = world.new_entity_named(c"test");
    assert!(entity.is_valid());
    assert_eq!(entity.get_name(), "test");
}

#[test]
fn entity_new_named_from_scope() {
    let world = World::new();
    let entity = world.new_entity_named(c"Foo");
    assert!(entity.is_valid());

    let prev = world.set_scope_with_id(entity);
    let child = world.new_entity_named(c"Bar");
    assert!(child.is_valid());

    world.set_scope_with_id(prev);

    assert_eq!(child.get_name(), "Bar");
    assert_eq!(child.get_path().unwrap(), "::Foo::Bar");
}

#[test]
fn entity_new_nested_named_from_nested_scope() {
    // Create a world
    let world = World::new();

    // Create an entity with nested name "Foo::Bar"
    let entity = world.new_entity_named(CStr::from_bytes_with_nul(b"Foo::Bar\0").unwrap());

    // Verify that the entity exists and its name and path are correct
    assert!(entity.is_valid());
    assert_eq!(entity.get_name(), "Bar");
    assert_eq!(entity.get_path().unwrap(), "::Foo::Bar");

    // Set the current scope to `entity`
    let prev = world.set_scope_with_id(entity);

    // Create a child entity with nested name "Hello::World" under the current scope
    let child = world.new_entity_named(CStr::from_bytes_with_nul(b"Hello::World\0").unwrap());

    // Verify that the child entity exists
    assert!(child.is_valid());

    // Restore the previous scope
    world.set_scope_with_id(prev);

    // Verify the name and hierarchical path of the child entity
    assert_eq!(child.get_name(), "World");
    assert_eq!(child.get_path().unwrap(), "::Foo::Bar::Hello::World");
}

#[test]
fn entity_new_add() {
    // Create a world
    let world = World::new();

    // Create an entity and add the Position component to it
    let entity = world.new_entity().add::<Position>();

    // Verify that the entity exists
    assert!(entity.is_valid());

    // Verify that the entity has the Position component
    assert!(entity.has::<Position>());
}

#[test]
fn entity_new_add_2() {
    // Create a world
    let world = World::new();

    // Create an entity and add the Position and Velocity components to it
    let entity = world.new_entity().add::<Position>().add::<Velocity>();

    // Verify that the entity exists
    assert!(entity.is_valid());

    // Verify that the entity has the Position component
    assert!(entity.has::<Position>());

    // Verify that the entity has the Velocity component
    assert!(entity.has::<Velocity>());
}

#[test]
fn entity_new_set() {
    // Create a world
    let world = World::new();

    // Create an entity and set the Position component data
    let entity = world.new_entity().set(Position { x: 10.0, y: 20.0 });

    // Verify that the entity exists
    assert!(entity.is_valid());

    // Verify that the entity has the Position component
    assert!(entity.has::<Position>());

    // Verify the component data
    let p = entity.get::<Position>().unwrap();
    assert_eq!(p.x, 10.0);
    assert_eq!(p.y, 20.0);
}

#[test]
fn entity_new_set_2() {
    let world = World::new();

    let entity = world
        .new_entity()
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());

    let p = entity.get::<Position>().unwrap();
    assert_eq!(p.x, 10.0);
    assert_eq!(p.y, 20.0);

    let v = entity.get::<Velocity>().unwrap();
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
}

#[test]
fn entity_add() {
    let world = World::new();

    let entity = world.new_entity();

    assert!(entity.is_valid());

    entity.add::<Position>();

    assert!(entity.has::<Position>());
}

#[test]
fn entity_remove() {
    let world = World::new();

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add::<Position>();
    assert!(entity.has::<Position>());

    entity.remove::<Position>();
    assert!(!entity.has::<Position>());
}

#[test]
fn entity_set() {
    let world = World::new();

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.set(Position { x: 10.0, y: 20.0 });
    assert!(entity.has::<Position>());

    let p = entity.get::<Position>().unwrap();
    assert_eq!(p.x, 10.0);
    assert_eq!(p.y, 20.0);
}

#[test]
fn entity_add_2() {
    let world = World::new();

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add::<Position>().add::<Velocity>();

    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());
}

#[test]
fn entity_add_entity() {
    let world = World::new();

    let tag = world.new_entity();
    assert!(tag.is_valid());

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_id(tag);
    assert!(entity.has_id(tag));
}

#[test]
fn entity_add_childof() {
    let world = World::new();

    let parent = world.new_entity();
    assert!(parent.is_valid());

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_id((ECS_CHILD_OF, parent));
    assert!(entity.has_id((ECS_CHILD_OF, parent)));
}

#[test]
fn entity_add_instanceof() {
    let world = World::new();

    let base = world.new_entity();
    assert!(base.is_valid());

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_id((ECS_IS_A, base));
    assert!(entity.has_id((ECS_IS_A, base)));
}

#[test]
fn entity_remove_2() {
    let world = World::new();

    let entity = world.new_entity().add::<Position>().add::<Velocity>();

    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());

    entity.remove::<Position>().remove::<Velocity>();

    assert!(!entity.has::<Position>());
    assert!(!entity.has::<Velocity>());
}

#[test]
fn entity_set_2() {
    let world = World::new();

    let entity = world
        .new_entity()
        .set::<Position>(Position { x: 10.0, y: 20.0 })
        .set::<Velocity>(Velocity { x: 1.0, y: 2.0 });

    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());

    let p = entity.get::<Position>().unwrap();
    assert_eq!(p.x, 10.0);
    assert_eq!(p.y, 20.0);

    let v = entity.get::<Velocity>().unwrap();
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
}

#[test]
fn entity_remove_entity() {
    let world = World::new();

    let tag = world.new_entity();
    assert!(tag.is_valid());

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_id(tag);
    assert!(entity.has_id(tag));

    entity.remove_id(tag);
    assert!(!entity.has_id(tag));
}

#[test]
fn entity_remove_childof() {
    let world = World::new();

    let parent = world.new_entity();
    assert!(parent.is_valid());

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_id((ECS_CHILD_OF, parent));
    assert!(entity.has_id((ECS_CHILD_OF, parent)));

    entity.remove_id((ECS_CHILD_OF, parent));
    assert!(!entity.has_id((ECS_CHILD_OF, parent)));
}

#[test]
fn entity_remove_instanceof() {
    let world = World::new();

    let base = world.new_entity();
    assert!(base.is_valid());

    let entity = world.new_entity();
    assert!(entity.is_valid());

    entity.add_id((ECS_IS_A, base));
    assert!(entity.has_id((ECS_IS_A, base)));

    entity.remove_id((ECS_IS_A, base));
    assert!(!entity.has_id((ECS_IS_A, base)));
}

#[test]
fn entity_get_generic() {
    let world = World::new();
    let position = world.add::<Position>();

    let entity = world.new_entity().set(Position { x: 10.0, y: 20.0 });

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());

    let pos_void = entity.get_untyped(position);
    assert!(!pos_void.is_null());

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
    let world = World::new();
    let position = world.component::<Position>();

    let pos = Position { x: 10.0, y: 20.0 };

    let entity = world.new_entity().set_ptr_w_size(
        &position,
        std::mem::size_of::<Position>(),
        &pos as *const _ as *const c_void,
    );

    assert!(entity.has::<Position>());
    assert!(entity.has_id(position));

    let pos = entity.get::<Position>().unwrap();
    assert_eq!(pos.x, 10.0);
    assert_eq!(pos.y, 20.0);
}

#[test]
fn entity_set_generic_no_size() {
    let world = World::new();
    let position = world.component::<Position>();

    let pos = Position { x: 10.0, y: 20.0 };

    let entity = world
        .new_entity()
        .set_ptr(&position, &pos as *const _ as *const c_void);

    assert!(entity.has::<Position>());
    assert!(entity.has_id(position));

    let pos = entity.get::<Position>().unwrap();
    assert_eq!(pos.x, 10.0);
    assert_eq!(pos.y, 20.0);
}

#[test]
fn entity_add_role() {
    let world = World::new();
    let entity = world.new_entity();

    let entity = entity.add_flags(ECS_PAIR);

    assert_eq!(entity.raw_id & ECS_PAIR, ECS_PAIR);
}

#[test]
fn entity_remove_role() {
    let world = World::new();
    let entity = world.new_entity();
    let id = entity;

    let entity = entity.add_flags(ECS_PAIR);
    assert_eq!(entity.raw_id & ECS_PAIR, ECS_PAIR);

    let entity = entity.remove_flags();
    assert_eq!(entity, id);
}

#[test]
fn entity_has_role() {
    let world = World::new();
    let entity = world.new_entity();

    let entity = entity.add_flags(ECS_PAIR);
    assert!(entity.has_flags_for(ECS_PAIR));

    let entity = entity.remove_flags();
    assert!(!entity.has_flags_for(ECS_PAIR));
}

#[test]
fn entity_pair_role() {
    let world = World::new();
    let entity = world.new_entity();
    let entity2 = world.new_entity();

    let pair: Id = Id::new(None::<World>, (entity, entity2));
    let pair = pair.add_flags(ECS_PAIR);

    assert!(pair.has_flags_for(ECS_PAIR));

    let rel = pair.first();
    let obj = pair.second();

    assert_eq!(rel, entity);
    assert_eq!(obj, entity2);
}

#[test]
fn entity_equals() {
    let world = World::new();
    let e1 = world.new_entity();
    let e2 = world.new_entity();

    let e1_2 = e1;
    let e2_2 = e2;

    assert!(e1 == e1_2);
    assert!(e2 == e2_2);
    assert!(e1 >= e1_2);
    assert!(e1 <= e1_2);
    assert!(e2 >= e2_2);
    assert!(e2 <= e2_2);
    assert!(e1 != e2);

    assert!(e2 != e1_2);
    assert!(e1 != e2_2);
    assert!(e2 > e1_2);
    assert!(e1 < e2_2);
    assert!(e2 == e2);
}

#[test]
fn entity_compare_0() {
    let world = World::new();
    let e = world.new_entity();
    let e0 = world.new_entity_from_id(0);
    let e0_2 = world.new_entity_from_id(0);

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
    let world = World::new();

    let e1 = world.new_entity_from_id(500);
    let e2 = world.new_entity_from_id(600);

    assert_eq!(e1, 500);
    assert_eq!(e2, 600);

    assert_ne!(e1, 600);
    assert_ne!(e2, 500);

    assert!(e1 >= 500);
    assert!(e2 >= 600);

    assert!(e1 <= 500);
    assert!(e2 <= 600);

    assert!(e1 <= 600);
    assert!(e2 >= 500);

    assert!(e1 < 600);
    assert!(e2 > 500);

    assert!(e2 != 500);
    assert!(e1 != 600);

    assert!(e2 == 600);
    assert!(e1 == 500);

    assert!(e1 < 600);
    assert!(e2 > 500);
}

#[test]
fn entity_greater_than() {
    let world = World::new();

    let e1 = world.new_entity();
    let e2 = world.new_entity();

    assert!(e2 > e1);
    assert!(e2 >= e1);
}

#[test]
fn entity_less_than() {
    let world = World::new();

    let e1 = world.new_entity();
    let e2 = world.new_entity();

    assert!(e1 < e2);
    assert!(e1 <= e2);
}

#[test]
fn entity_not_0_or_1() {
    let world = World::new();

    let e = world.new_entity();

    let id = e;

    assert_ne!(id, 0);
    assert_ne!(id, 1);
}

#[test]
fn entity_has_childof() {
    let world = World::new();

    let parent = world.new_entity();

    let child = world.new_entity().add_id((ECS_CHILD_OF, parent));

    assert!(child.has_id((ECS_CHILD_OF, parent)));
}

#[test]
fn entity_has_instanceof() {
    let world = World::new();

    let base = world.new_entity();

    let instance = world.new_entity().add_id((ECS_IS_A, base));

    assert!(instance.has_id((ECS_IS_A, base)));
}

#[test]
fn entity_has_instanceof_indirect() {
    let world = World::new();

    let base_of_base = world.new_entity();
    let base = world.new_entity().add_id((ECS_IS_A, base_of_base));

    let instance = world.new_entity().add_id((ECS_IS_A, base));

    assert!(instance.has_id((ECS_IS_A, base_of_base)));
}

#[test]
fn entity_null_string() {
    let world = World::new();

    let entity = world.new_entity();

    assert_eq!(entity.get_name(), "");
}

#[test]
fn entity_none_string() {
    let world = World::new();

    let entity = world.new_entity();

    assert_eq!(entity.get_name_optional(), None);
}

#[test]
fn entity_set_name() {
    let world = World::new();

    let entity = world.new_entity();

    entity.set_name(c"Foo");

    assert_eq!(entity.get_name(), "Foo");
}

#[test]
fn entity_set_name_optional() {
    let world = World::new();

    let entity = world.new_entity();

    entity.set_name(c"Foo");

    assert_eq!(entity.get_name_optional(), Some("Foo"));
}

#[test]
fn entity_change_name() {
    let world = World::new();

    let entity = world.new_entity_named(c"Bar");
    assert_eq!(entity.get_name(), "Bar");

    entity.set_name(c"Foo");
    assert_eq!(entity.get_name(), "Foo");

    entity.set_name(c"Bar");
    assert_eq!(entity.get_name(), "Bar");
}

#[test]
fn entity_delete() {
    let world = World::new();

    let entity = world.new_entity().add::<Position>().add::<Velocity>();

    entity.destruct();
    assert!(!entity.is_alive());

    let entity2 = world.new_entity();
    assert_eq!(entity2.raw_id as u32, entity.raw_id as u32);
    assert_ne!(entity2, entity);
}

#[test]
fn entity_clear() {
    let world = World::new();

    let entity = world.new_entity().add::<Position>().add::<Velocity>();

    entity.clear();
    assert!(!entity.has::<Position>());
    assert!(!entity.has::<Velocity>());

    let entity2 = world.new_entity();
    assert!(entity2 > entity);
}

#[test]
#[ignore]
fn entity_force_owned() {
    todo!("prefab not yet implemented");
}

#[test]
#[ignore]
fn entity_force_owned_2() {
    todo!("prefab not yet implemented");
}

#[test]
#[ignore]
fn entity_force_owned_nested() {
    todo!("prefab not get implemented");
}

#[test]
fn entity_tag_has_size_zero() {
    let world = World::new();

    let comp = world.component::<TagA>();
    let ptr = comp.get::<EcsComponent>().unwrap();

    assert_eq!(ptr.size, 0);
    assert_eq!(ptr.alignment, 0);
}

#[test]
fn entity_get_target() {
    let world = World::new();

    let rel = world.new_entity();

    let obj1 = world.new_entity().add::<Position>();
    let obj2 = world.new_entity().add::<Velocity>();
    let obj3 = world.new_entity().add::<Mass>();
    let child = world
        .new_entity()
        .add_id((rel, obj1))
        .add_id((rel, obj2))
        .add_id((rel, obj3));

    let mut target = child.get_target_id(rel, 0);
    assert!(target.is_valid());
    assert_eq!(target, obj1);

    target = child.get_target_id(rel, 1);
    assert!(target.is_valid());
    assert_eq!(target, obj2);

    target = child.get_target_id(rel, 2);
    assert!(target.is_valid());
    assert_eq!(target, obj3);

    target = child.get_target_id(rel, 3);
    assert!(!target.is_valid());
}

#[test]
fn entity_get_parent() {
    let world = World::new();

    let parent = world.new_entity();
    let child = world.new_entity().child_of_id(parent);

    assert_eq!(child.get_target_id(ECS_CHILD_OF, 0), parent);
    assert_eq!(child.get_parent(), parent);
}

#[test]
fn entity_is_enabled_disabled() {
    let world = World::new();

    let entity = world
        .new_entity()
        .add::<Position>()
        .add::<Velocity>()
        .add::<Mass>();

    assert!(entity.is_enabled::<Position>());
    assert!(entity.is_enabled::<Velocity>());
    assert!(entity.is_enabled::<Mass>());

    entity.disable::<Position>();

    assert!(!entity.is_enabled::<Position>());
    assert!(entity.is_enabled::<Velocity>());
    assert!(entity.is_enabled::<Mass>());

    entity.disable::<Velocity>();

    assert!(!entity.is_enabled::<Position>());
    assert!(!entity.is_enabled::<Velocity>());
    assert!(entity.is_enabled::<Mass>());

    entity.disable::<Mass>();

    assert!(!entity.is_enabled::<Position>());
    assert!(!entity.is_enabled::<Velocity>());
    assert!(!entity.is_enabled::<Mass>());

    entity.enable::<Position>();

    assert!(entity.is_enabled::<Position>());
    assert!(!entity.is_enabled::<Velocity>());
    assert!(!entity.is_enabled::<Mass>());

    entity.enable::<Velocity>();

    assert!(entity.is_enabled::<Position>());
    assert!(entity.is_enabled::<Velocity>());
    assert!(!entity.is_enabled::<Mass>());

    entity.enable::<Mass>();

    assert!(entity.is_enabled::<Position>());
    assert!(entity.is_enabled::<Velocity>());
    assert!(entity.is_enabled::<Mass>());
}

#[test]
fn entity_is_enabled_pair() {
    let world = World::new();

    let entity = world
        .new_entity()
        .add::<(Position, TagA)>()
        .add::<(TagB, TagC)>()
        .disable::<(Position, TagC)>();

    assert!(entity.is_enabled::<(Position, TagA)>());
    assert!(!entity.is_enabled::<(Position, TagB)>());
    assert!(!entity.is_enabled::<(Position, TagC)>());

    entity.enable::<(Position, TagB)>();
    assert!(entity.is_enabled::<(Position, TagB)>());

    entity.disable::<(Position, TagA)>();
    assert!(!entity.is_enabled::<(Position, TagA)>());

    entity.enable::<(Position, TagA)>();
    entity.enable::<(Position, TagC)>();
    assert!(entity.is_enabled::<(Position, TagA)>());
    assert!(entity.is_enabled::<(Position, TagC)>());

    entity.disable::<(Position, TagB)>();
    assert!(!entity.is_enabled::<(Position, TagB)>());
}

#[test]
fn entity_is_enabled_pair_ids() {
    let world = World::new();

    let rel = world.new_entity();
    let tgt_a = world.new_entity();
    let tgt_b = world.new_entity();

    let e = world.new_entity().add_id((rel, tgt_a));

    assert!(e.is_enabled_id((rel, tgt_a)));
    assert!(!e.is_enabled_id((rel, tgt_b)));

    e.disable_id((rel, tgt_a));
    assert!(!e.is_enabled_id((rel, tgt_a)));

    e.enable_id((rel, tgt_a));
    assert!(e.is_enabled_id((rel, tgt_a)));

    e.add_id((rel, tgt_b)).disable_id((rel, tgt_b));
    assert!(!e.is_enabled_id((rel, tgt_b)));

    e.enable_id((rel, tgt_b));
    assert!(e.is_enabled_id((rel, tgt_b)));
}

#[test]
fn entity_is_pair_first_enabled() {
    let world = World::new();

    let tgt_a = world.new_entity();
    let tgt_b = world.new_entity();

    let e = world.new_entity().add_pair_first::<Position>(tgt_a);

    assert!(e.is_enabled_pair_first::<Position>(tgt_a));
    assert!(!e.is_enabled_pair_first::<Position>(tgt_b));
}

#[test]
fn entity_get_type() {
    let world = World::new();

    let entity = world.new_entity();
    assert!(entity.is_valid());

    let type_1 = entity.get_archetype();
    assert_eq!(type_1.count(), 0);

    entity.add::<Position>();

    let type_2 = entity.get_archetype();
    assert_eq!(type_2.count(), 1);
    assert_eq!(type_2.id_at_index(0).unwrap(), world.get_id::<Position>());

    entity.add::<Velocity>();
    let type_3 = entity.get_archetype();
    assert_eq!(type_3.count(), 2);
    assert_eq!(type_3.id_at_index(1).unwrap(), world.get_id::<Velocity>());
}
