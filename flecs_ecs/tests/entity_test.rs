use std::ffi::{c_void, CStr};

use flecs_ecs::prelude::*;

mod common;
use common::*;

#[test]
fn entity_new() {
    let world = create_world();
    let entity = world.entity();
    assert!(entity.is_valid());
}

#[test]
fn entity_new_named() {
    let world = create_world();
    let entity = world.entity_named(c"test");

    assert!(entity.is_valid());
    assert_eq!(entity.name(), "test");
}

#[test]
fn entity_new_named_from_scope() {
    let world = create_world();
    let entity = world.entity_named(c"Foo");
    assert!(entity.is_valid());

    let prev = world.set_scope_id(entity);
    let child = world.entity_named(c"Bar");
    assert!(child.is_valid());

    world.set_scope_id(prev);

    assert_eq!(child.name(), "Bar");
    assert_eq!(child.path().unwrap(), "::Foo::Bar");
}

#[test]
fn entity_new_nested_named_from_nested_scope() {
    // Create a world
    let world = create_world();

    // Create an entity with nested name "Foo::Bar"
    let entity = world.entity_named(CStr::from_bytes_with_nul(b"Foo::Bar\0").unwrap());

    // Verify that the entity exists and its name and path are correct
    assert!(entity.is_valid());
    assert_eq!(entity.name(), "Bar");
    assert_eq!(entity.path().unwrap(), "::Foo::Bar");

    // Set the current scope to `entity`
    let prev = world.set_scope_id(entity);

    // Create a child entity with nested name "Hello::World" under the current scope
    let child = world.entity_named(CStr::from_bytes_with_nul(b"Hello::World\0").unwrap());

    // Verify that the child entity exists
    assert!(child.is_valid());

    // Restore the previous scope
    world.set_scope_id(prev);

    // Verify the name and hierarchical path of the child entity
    assert_eq!(child.name(), "World");
    assert_eq!(child.path().unwrap(), "::Foo::Bar::Hello::World");
}

#[test]
fn entity_new_add() {
    let world = create_world();

    let entity = world.entity().add::<Position>();

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());
}

#[test]
fn entity_new_add_2() {
    let world = create_world();

    let entity = world.entity().add::<Position>().add::<Velocity>();

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());
}

#[test]
fn entity_new_set() {
    // Create a world
    let world = create_world();

    // Create an entity and set the Position component data
    let entity = world.entity().set(Position { x: 10, y: 20 });

    // Verify that the entity exists
    assert!(entity.is_valid());

    // Verify that the entity has the Position component
    assert!(entity.has::<Position>());

    // Verify the component data
    let p = entity.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_new_set_2() {
    let world = create_world();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());

    let p = entity.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let v = entity.try_get::<Velocity>().unwrap();
    assert_eq!(v.x, 1);
    assert_eq!(v.y, 2);
}

#[test]
fn entity_add() {
    let world = create_world();

    let entity = world.entity();

    assert!(entity.is_valid());

    entity.add::<Position>();

    assert!(entity.has::<Position>());
}

#[test]
fn entity_remove() {
    let world = create_world();

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add::<Position>();
    assert!(entity.has::<Position>());

    entity.remove::<Position>();
    assert!(!entity.has::<Position>());
}

#[test]
fn entity_set() {
    let world = create_world();

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.set(Position { x: 10, y: 20 });
    assert!(entity.has::<Position>());

    let p = entity.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_add_2() {
    let world = create_world();

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add::<Position>().add::<Velocity>();

    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());
}

#[test]
fn entity_add_entity() {
    let world = create_world();

    let tag = world.entity();
    assert!(tag.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id(tag);
    assert!(entity.has_id(tag));
}

#[test]
fn entity_add_childof() {
    let world = create_world();

    let parent = world.entity();
    assert!(parent.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id((flecs::ChildOf::ID, parent));
    assert!(entity.has_id((flecs::ChildOf::ID, parent)));
}

#[test]
fn entity_add_instanceof() {
    let world = create_world();

    let base = world.entity();
    assert!(base.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id((flecs::IsA::ID, base));
    assert!(entity.has_id((flecs::IsA::ID, base)));
}

#[test]
fn entity_remove_2() {
    let world = create_world();

    let entity = world.entity().add::<Position>().add::<Velocity>();

    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());

    entity.remove::<Position>().remove::<Velocity>();

    assert!(!entity.has::<Position>());
    assert!(!entity.has::<Velocity>());
}

#[test]
fn entity_set_2() {
    let world = create_world();

    let entity = world
        .entity()
        .set::<Position>(Position { x: 10, y: 20 })
        .set::<Velocity>(Velocity { x: 1, y: 2 });

    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());

    let p = entity.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let v = entity.try_get::<Velocity>().unwrap();
    assert_eq!(v.x, 1);
    assert_eq!(v.y, 2);
}

#[test]
fn entity_remove_entity() {
    let world = create_world();

    let tag = world.entity();
    assert!(tag.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id(tag);
    assert!(entity.has_id(tag));

    entity.remove_id(tag);
    assert!(!entity.has_id(tag));
}

#[test]
fn entity_remove_childof() {
    let world = create_world();

    let parent = world.entity();
    assert!(parent.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id((flecs::ChildOf::ID, parent));
    assert!(entity.has_id((flecs::ChildOf::ID, parent)));

    entity.remove_id((flecs::ChildOf::ID, parent));
    assert!(!entity.has_id((flecs::ChildOf::ID, parent)));
}

#[test]
fn entity_remove_instanceof() {
    let world = create_world();

    let base = world.entity();
    assert!(base.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id((flecs::IsA::ID, base));
    assert!(entity.has_id((flecs::IsA::ID, base)));

    entity.remove_id((flecs::IsA::ID, base));
    assert!(!entity.has_id((flecs::IsA::ID, base)));
}

#[test]
fn entity_get_generic() {
    let world = create_world();
    let position = world.add::<Position>();

    let entity = world.entity().set(Position { x: 10, y: 20 });

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());

    let pos_void = entity.get_untyped(position);
    assert!(!pos_void.is_null());

    let pos = unsafe { &*(pos_void as *const Position) };
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);
}

#[test]
fn entity_get_generic_mut() {
    let world = create_world();

    let position = world.component::<Position>();

    let entity = world.entity().set(Position { x: 10, y: 20 });

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());

    world
        .observer::<&Position>()
        .add_event::<flecs::OnSet>()
        .each_entity(|entity, _| {
            entity.world().get_mut::<Flags>().insert("invoked", 1);
        });

    let pos = entity.get_untyped_mut(position.id());
    assert!(!pos.is_null());

    let pos = unsafe { &mut *(pos as *mut Position) };
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);

    entity.modified_id(position);
    assert!(world.get::<Flags>().contains_key("invoked"));
}

#[test]
fn entity_get_mut_generic_w_id() {
    let world = create_world();

    let position = world.component::<Position>();

    let entity = world.entity().set(Position { x: 10, y: 20 });

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());

    let void_p = entity.get_untyped(position);
    assert!(!void_p.is_null());

    let p = unsafe { &*(void_p as *const Position) };
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_set_generic() {
    let world = create_world();
    let position = world.component::<Position>();

    let pos = Position { x: 10, y: 20 };

    let entity = unsafe {
        world.entity().set_ptr_w_size(
            position.id(),
            std::mem::size_of::<Position>(),
            &pos as *const _ as *const c_void,
        )
    };

    assert!(entity.has::<Position>());
    assert!(entity.has_id(position));

    let pos = entity.try_get::<Position>().unwrap();
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);
}

#[test]
fn entity_set_generic_no_size() {
    let world = create_world();
    let position = world.component::<Position>();

    let pos = Position { x: 10, y: 20 };

    let entity = unsafe {
        world
            .entity()
            .set_ptr(position.id(), &pos as *const _ as *const c_void)
    };

    assert!(entity.has::<Position>());
    assert!(entity.has_id(position));

    let pos = entity.try_get::<Position>().unwrap();
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);
}

#[test]
fn entity_add_role() {
    let world = create_world();
    let entity = world.entity();

    let entity = entity.add_flags(flecs::Pair::ID);

    assert_eq!(entity.id() & flecs::Pair::ID, flecs::Pair::ID);
}

#[test]
fn entity_remove_role() {
    let world = create_world();
    let entity = world.entity();
    let id = entity;

    let entity = entity.add_flags(flecs::Pair::ID);
    assert_eq!(entity.id() & flecs::Pair::ID, flecs::Pair::ID);

    let entity = entity.remove_flags();
    assert_eq!(entity, id);
}

#[test]
fn entity_has_role() {
    let world = create_world();
    let entity = world.entity();

    let entity = entity.add_flags(flecs::Pair::ID);
    assert!(entity.has_flags_for(flecs::Pair::ID));

    let entity = entity.remove_flags();
    assert!(!entity.has_flags_for(flecs::Pair::ID));
}

#[test]
fn entity_pair_role() {
    let world = create_world();
    let entity = world.entity();
    let entity2 = world.entity();

    let pair: IdView = IdView::new_from(&world, (entity, entity2));
    let pair = pair.add_flags(flecs::Pair::ID);

    assert!(pair.has_flags_for(flecs::Pair::ID));

    let rel = pair.first_id();
    let obj = pair.second_id();

    assert_eq!(rel, entity);
    assert_eq!(obj, entity2);
}

#[test]
fn entity_equals() {
    let world = create_world();
    let e1 = world.entity();
    let e2 = world.entity();

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
    let world = create_world();
    let e = world.entity();
    let e0 = world.entity_from_id(0);
    let e0_2 = world.entity_from_id(0);

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
    let world = create_world();

    let e1 = world.entity_from_id(500);
    let e2 = world.entity_from_id(600);

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
    let world = create_world();

    let e1 = world.entity();
    let e2 = world.entity();

    assert!(e2 > e1);
    assert!(e2 >= e1);
}

#[test]
fn entity_less_than() {
    let world = create_world();

    let e1 = world.entity();
    let e2 = world.entity();

    assert!(e1 < e2);
    assert!(e1 <= e2);
}

#[test]
fn entity_not_0_or_1() {
    let world = create_world();

    let e = world.entity();

    let id = e;

    assert_ne!(id, 0);
    assert_ne!(id, 1);
}

#[test]
fn entity_has_childof() {
    let world = create_world();

    let parent = world.entity();

    let child = world.entity().add_id((flecs::ChildOf::ID, parent));

    assert!(child.has_id((flecs::ChildOf::ID, parent)));
}

#[test]
fn entity_has_instanceof() {
    let world = create_world();

    let base = world.entity();

    let instance = world.entity().add_id((flecs::IsA::ID, base));

    assert!(instance.has_id((flecs::IsA::ID, base)));
}

#[test]
fn entity_has_instanceof_indirect() {
    let world = create_world();

    let base_of_base = world.entity();
    let base = world.entity().add_id((flecs::IsA::ID, base_of_base));

    let instance = world.entity().add_id((flecs::IsA::ID, base));

    assert!(instance.has_id((flecs::IsA::ID, base_of_base)));
}

#[test]
fn entity_null_string() {
    let world = create_world();

    let entity = world.entity();

    assert_eq!(entity.name(), "");
}

#[test]
fn entity_none_string() {
    let world = create_world();

    let entity = world.entity();

    assert_eq!(entity.get_name(), None);
}

#[test]
fn entity_set_name() {
    let world = create_world();

    let entity = world.entity();

    entity.set_name(c"Foo");

    assert_eq!(entity.name(), "Foo");
}

#[test]
fn entity_set_name_optional() {
    let world = create_world();

    let entity = world.entity();

    entity.set_name(c"Foo");

    assert_eq!(entity.get_name(), Some("Foo"));
}

#[test]
fn entity_change_name() {
    let world = create_world();

    let entity = world.entity_named(c"Bar");
    assert_eq!(entity.name(), "Bar");

    entity.set_name(c"Foo");
    assert_eq!(entity.name(), "Foo");

    entity.set_name(c"Bar");
    assert_eq!(entity.name(), "Bar");
}

#[test]
fn entity_delete() {
    let world = create_world();

    let entity = world.entity().add::<Position>().add::<Velocity>();

    entity.destruct();
    assert!(!entity.is_alive());

    let entity2 = world.entity();

    assert_eq!(*entity2.id() as u32, *entity.id() as u32);
    assert_ne!(entity2, entity);
}

#[test]
fn entity_clear() {
    let world = create_world();

    let entity = world.entity().add::<Position>().add::<Velocity>();

    entity.clear();
    assert!(!entity.has::<Position>());
    assert!(!entity.has::<Velocity>());

    let entity2 = world.entity();
    assert!(entity2 > entity);
}

#[test]
fn entity_force_owned() {
    let world = create_world();

    let prefab = world
        .prefab()
        .add::<Position>()
        .add::<Velocity>()
        .override_type::<Position>();

    let entity = world.entity().add_id((flecs::IsA::ID, prefab));

    assert!(entity.has::<Position>());
    assert!(entity.owns::<Position>());
    assert!(entity.has::<Velocity>());
    assert!(!entity.owns::<Velocity>());
}

#[test]
fn entity_force_owned_2() {
    let world = create_world();

    let prefab = world
        .prefab()
        .add::<Position>()
        .add::<Velocity>()
        .override_type::<Position>()
        .override_type::<Velocity>();

    let entity = world.entity().add_id((flecs::IsA::ID, prefab));

    assert!(entity.has::<Position>());
    assert!(entity.owns::<Position>());
    assert!(entity.has::<Velocity>());
    assert!(entity.owns::<Velocity>());
}

#[test]
fn entity_force_owned_nested() {
    let world = create_world();

    let prefab = world
        .prefab()
        .add::<Position>()
        .add::<Velocity>()
        .override_type::<Position>();

    let prefab_2 = world.entity().add_id((flecs::IsA::ID, prefab));

    let entity = world.entity().add_id((flecs::IsA::ID, prefab_2));

    assert!(entity.has::<Position>());
    assert!(entity.owns::<Position>());
    assert!(entity.has::<Velocity>());
    assert!(!entity.owns::<Velocity>());
}

#[test]
fn entity_tag_has_size_zero() {
    let world = create_world();

    let comp = world.component::<TagA>();
    let ptr = comp.try_get::<EcsComponent>().unwrap();

    assert_eq!(ptr.size, 0);
    assert_eq!(ptr.alignment, 0);
}

#[test]
fn entity_get_null_name() {
    let world = create_world();

    let entity = world.entity();
    let name = entity.get_name();
    assert_eq!(name, None);
}

#[test]
fn entity_get_target() {
    let world = create_world();

    let rel = world.entity();

    let obj1 = world.entity().add::<Position>();
    let obj2 = world.entity().add::<Velocity>();
    let obj3 = world.entity().add::<Mass>();
    let child = world
        .entity()
        .add_id((rel, obj1))
        .add_id((rel, obj2))
        .add_id((rel, obj3));

    let mut target = child.target_id(rel, 0);
    assert!(target.is_valid());
    assert_eq!(target, obj1);

    target = child.target_id(rel, 1);
    assert!(target.is_valid());
    assert_eq!(target, obj2);

    target = child.target_id(rel, 2);
    assert!(target.is_valid());
    assert_eq!(target, obj3);

    target = child.target_id(rel, 3);
    assert!(!target.is_valid());
}

#[test]
fn entity_get_parent() {
    let world = create_world();

    let parent = world.entity();
    let child = world.entity().child_of_id(parent);

    assert_eq!(child.target_id(flecs::ChildOf::ID, 0), parent);
    assert_eq!(child.parent(), parent);
}

/// # See also
/// * C++ tests: `Entity_is_enabled_component_enabled` + `Entity_is_disabled_component_enabled` combined
#[test]
fn entity_is_component_enabled() {
    let world = create_world();

    world.component::<Position>().add_id(flecs::CanToggle::ID);
    world.component::<Velocity>().add_id(flecs::CanToggle::ID);
    world.component::<Mass>().add::<flecs::CanToggle>();

    let entity = world
        .entity()
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

//todo v4 bug flecs core
/// # See also
/// * C++ tests: `Entity_is_enabled_pair_enabled` + `Entity_is_disabled_pair_enabled` combined
//#[test]
// fn entity_is_enabled_pair() {
//     let world = create_world();

//     world.component::<Position>().add_id(flecs::CanToggle::ID);
//     world.component::<TagA>().add_id(flecs::CanToggle::ID);
//     world.component::<TagB>().add_id(flecs::CanToggle::ID);
//     world.component::<TagC>().add_id(flecs::CanToggle::ID);

//     let entity = world
//         .entity()
//         .add::<(Position, TagA)>()
//         .add::<(TagB, TagC)>()
//         .disable::<(Position, TagC)>();

//     assert!(entity.is_enabled::<(Position, TagA)>());
//     assert!(!entity.is_enabled::<(Position, TagB)>());
//     assert!(!entity.is_enabled::<(Position, TagC)>());

//     entity.enable::<(Position, TagB)>();
//     assert!(entity.is_enabled::<(Position, TagB)>());

//     entity.disable::<(Position, TagA)>();
//     assert!(!entity.is_enabled::<(Position, TagA)>());

//     entity.enable::<(Position, TagA)>();
//     entity.enable::<(Position, TagC)>();
//     assert!(entity.is_enabled::<(Position, TagA)>());
//     assert!(entity.is_enabled::<(Position, TagC)>());

//     entity.disable::<(Position, TagB)>();
//     assert!(!entity.is_enabled::<(Position, TagB)>());
// }

//todo v4 bug flecs core
/// # See also
/// * C++ tests: `Entity_is_disabled_pair_enabled_w_tgt_id` + `Entity_is_enabled_pair_enabled_w_tgt_id` +
///  `Entity_is_pair_enabled_w_tgt_id` + `Entity_is_disabled_pair_enabled_w_ids` +
/// `Entity_is_enabled_pair_enabled_w_ids` + `Entity_is_pair_enabled_w_ids` combined
//#[test]
// fn entity_is_enabled_pair_ids() {
//     let world = create_world();

//     let rel = world.entity();
//     let tgt_a = world.entity();
//     let tgt_b = world.entity();

//     let e = world.entity().add_id((rel, tgt_a));

//     assert!(e.is_enabled_id((rel, tgt_a)));
//     assert!(!e.is_enabled_id((rel, tgt_b)));

//     e.disable_id((rel, tgt_a));
//     assert!(!e.is_enabled_id((rel, tgt_a)));

//     e.enable_id((rel, tgt_a));
//     assert!(e.is_enabled_id((rel, tgt_a)));

//     e.add_id((rel, tgt_b)).disable_id((rel, tgt_b));
//     assert!(!e.is_enabled_id((rel, tgt_b)));

//     e.enable_id((rel, tgt_b));
//     assert!(e.is_enabled_id((rel, tgt_b)));
// }
#[test]
fn entity_is_first_enabled() {
    let world = create_world();

    let tgt_a = world.entity();
    let tgt_b = world.entity();

    let e = world.entity().add_first::<Position>(tgt_a);

    assert!(e.is_enabled_first::<Position>(tgt_a));
    assert!(!e.is_enabled_first::<Position>(tgt_b));
}

#[test]
fn entity_get_type() {
    let world = create_world();

    let entity = world.entity();
    assert!(entity.is_valid());

    {
        let type_1 = entity.archetype();
        assert_eq!(type_1.count(), 0);
    }

    entity.add::<Position>();

    {
        let type_2 = entity.archetype();
        assert_eq!(type_2.count(), 1);
        assert_eq!(type_2[0], world.id_from::<Position>());
    }

    entity.add::<Velocity>();
    let type_3 = entity.archetype();
    assert_eq!(type_3.count(), 2);
    assert_eq!(type_3[1], world.id_from::<Velocity>());
}

#[test]
fn entity_get_nonempty_type() {
    let world = create_world();

    let entity = world.entity().add::<Position>();
    assert!(entity.is_valid());

    let type_1 = entity.archetype();
    assert_eq!(type_1.count(), 1);
    assert_eq!(type_1.get(0).unwrap(), world.id_from::<Position>());

    let type_2 = entity.archetype();
    assert_eq!(type_2.count(), 1);
    assert_eq!(type_2.get(0).unwrap(), world.id_from::<Position>());
}

#[test]
fn entity_set_no_copy() {
    let world = create_world();

    let entity = world.entity().set(Pod::new(10));

    let clone_invoked = entity.try_get::<Pod>().unwrap().clone_count;

    assert_eq!(clone_invoked, 0);

    assert!(entity.has::<Pod>());

    let p = entity.try_get::<Pod>();

    assert!(p.is_some());

    let p = p.unwrap();

    assert_eq!(p.value, 10);
}

#[test]
fn entity_set_copy() {
    let world = create_world();

    let entity = world.entity().set(Pod::new(10));

    let entity_dupl = entity.duplicate(true);

    let clone_invoked = entity_dupl.try_get::<Pod>().unwrap().clone_count;

    assert_eq!(clone_invoked, 1);

    assert!(entity.has::<Pod>());
    let p = entity.try_get::<Pod>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.value, 10);

    assert!(entity_dupl.has::<Pod>());
    let p = entity_dupl.try_get::<Pod>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.value, 10);
}

#[test]
fn entity_set_deduced() {
    let world = create_world();

    let entity = world.entity().set(Position { x: 10, y: 20 });

    assert!(entity.has::<Position>());

    let p = entity.try_get::<Position>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_override() {
    let world = create_world();

    let base = world.entity().override_type::<Position>();

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<Position>());
    assert!(entity.owns::<Position>());
}

#[test]
fn entity_override_id() {
    let world = create_world();

    let tag_a = world.entity();
    let tag_b = world.entity();

    let base = world.entity().override_id(tag_a).add_id(tag_b);

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has_id(tag_a));
    assert!(entity.owns_id(tag_a));

    assert!(entity.has_id(tag_b));
    assert!(!entity.owns_id(tag_b));
}

#[test]
fn entity_override_pair_w_tgt_id() {
    let world = create_world();

    let tgt_a = world.entity();
    let tgt_b = world.entity();

    let base = world
        .entity()
        .override_first::<Position>(tgt_a)
        .add_first::<Position>(tgt_b);

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has_first::<Position>(tgt_a));
    assert!(entity.owns_first::<Position>(tgt_a));

    assert!(entity.has_first::<Position>(tgt_b));
    assert!(!entity.owns_first::<Position>(tgt_b));
}

#[test]
fn entity_override_pair_w_ids() {
    let world = create_world();

    let rel = world.entity();
    let tgt_a = world.entity();
    let tgt_b = world.entity();

    let base = world
        .entity()
        .override_id((rel, tgt_a))
        .add_id((rel, tgt_b));

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has_id((rel, tgt_a)));
    assert!(entity.owns_id((rel, tgt_a)));

    assert!(entity.has_id((rel, tgt_b)));
    assert!(!entity.owns_id((rel, tgt_b)));
}

#[test]
fn entity_override_pair() {
    let world = create_world();

    let base = world
        .entity()
        .override_type::<(Position, TagA)>()
        .add::<(Position, TagB)>();

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<(Position, TagA)>());
    assert!(entity.owns::<(Position, TagA)>());

    assert!(entity.has::<(Position, TagB)>());
    assert!(!entity.owns::<(Position, TagB)>());
}

#[test]
fn entity_set_override() {
    let world = create_world();

    let base = world.entity().set_override(Position { x: 10, y: 20 });

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<Position>());
    assert!(entity.owns::<Position>());

    let p = entity.try_get::<Position>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let p_base = base.try_get::<Position>();
    assert!(p_base.is_some());
    let p_base = p_base.unwrap();
    assert_eq!(p_base.x, 10);
    assert_eq!(p_base.y, 20);
}

#[test]
fn entity_set_override_lvalue() {
    let world = create_world();

    let plvalue = Position { x: 10, y: 20 };

    let base = world.entity().set_override(plvalue);

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<Position>());
    assert!(entity.owns::<Position>());

    let p = entity.try_get::<Position>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let p_base = base.try_get::<Position>();
    assert!(p_base.is_some());
    let p_base = p_base.unwrap();
    assert_eq!(p_base.x, 10);
    assert_eq!(p_base.y, 20);
}

#[test]
fn entity_set_override_pair() {
    let world = create_world();

    let base = world
        .entity()
        .set_override_first::<PositionPair, TagA>(PositionPair { x: 10, y: 20 });

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<(PositionPair, TagA)>());
    assert!(entity.owns::<(PositionPair, TagA)>());

    let p = entity.try_get_first::<PositionPair, TagA>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let p_base = base.try_get_first::<PositionPair, TagA>();
    assert!(p_base.is_some());
    let p_base = p_base.unwrap();
    assert_eq!(p_base.x, 10);
    assert_eq!(p_base.y, 20);
}

#[test]
fn entity_set_override_pair_w_tgt_id() {
    let world = create_world();

    let tgt = world.entity();

    let base = world
        .entity()
        .set_override_first_id::<Position>(Position { x: 10, y: 20 }, tgt);

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has_first::<Position>(tgt));
    assert!(entity.owns_first::<Position>(tgt));

    let p = entity.try_get_first_id::<Position>(tgt);
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let p_base = base.try_get_first_id::<Position>(tgt);
    assert!(p_base.is_some());
    let p_base = p_base.unwrap();
    assert_eq!(p_base.x, 10);
    assert_eq!(p_base.y, 20);
}

#[test]
fn entity_set_override_pair_w_rel_tag() {
    let world = create_world();

    let base = world
        .entity()
        .set_override_second::<TagA, PositionPair>(PositionPair { x: 10, y: 20 });

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<(TagA, PositionPair)>());
    assert!(entity.owns::<(TagA, PositionPair)>());

    let p = entity.try_get_second::<TagA, PositionPair>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let p_base = base.try_get_second::<TagA, PositionPair>();
    assert!(p_base.is_some());
    let p_base = p_base.unwrap();
    assert_eq!(p_base.x, 10);
    assert_eq!(p_base.y, 20);
}

#[test]
fn entity_name() {
    let world = create_world();

    let entity = world.entity_named(c"Foo");

    assert_eq!(entity.name(), "Foo");
    assert_eq!(entity.get_name(), Some("Foo"));
    assert_eq!(entity.name_cstr(), c"Foo");
    assert_eq!(entity.get_name_cstr(), Some(c"Foo"));
}

#[test]
fn entity_name_empty() {
    let world = create_world();

    let entity = world.entity();

    assert_eq!(entity.name(), "");
    assert_eq!(entity.get_name(), None);
    assert_eq!(entity.name_cstr(), c"");
    assert_eq!(entity.get_name_cstr(), None);
}

#[test]
fn entity_path() {
    let world = create_world();

    let parent = world.entity_named(c"parent");
    world.set_scope_id(parent.id());
    let child = world.entity_named(c"child");

    assert_eq!(&child.path().unwrap(), "::parent::child");
}

#[test]
fn entity_path_from() {
    let world = create_world();

    let parent = world.entity_named(c"parent");
    world.set_scope_id(parent.id());
    let child = world.entity_named(c"child");
    world.set_scope_id(child.id());
    let grandchild = world.entity_named(c"grandchild");

    assert_eq!(&grandchild.path().unwrap(), "::parent::child::grandchild");
    assert_eq!(
        &grandchild.path_from_id(parent).unwrap(),
        "child::grandchild"
    );
}

#[test]
fn entity_path_from_type() {
    let world = create_world();

    let parent = world.entity_named(c"parent");
    world.set_scope_id(parent.id());
    let child = world.entity_named(c"child");
    world.set_scope_id(child.id());
    let grandchild = world.entity_named(c"grandchild");

    assert_eq!(&grandchild.path().unwrap(), "::parent::child::grandchild");
    assert_eq!(
        &grandchild.path_from_id(parent).unwrap(),
        "child::grandchild"
    );
}

#[test]
fn entity_path_custom_sep() {
    let world = create_world();

    let parent = world.entity_named(c"parent");
    world.set_scope_id(parent.id());
    let child = world.entity_named(c"child");

    assert_eq!(&child.path_w_sep(c"_", c"?").unwrap(), "?parent_child");
}

#[test]
fn entity_path_from_custom_sep() {
    let world = create_world();

    let parent = world.entity_named(c"parent");
    world.set_scope_id(parent.id());
    let child = world.entity_named(c"child");
    world.set_scope_id(child.id());
    let grandchild = world.entity_named(c"grandchild");

    assert_eq!(
        &grandchild.path_w_sep(c"_", c"?").unwrap(),
        "?parent_child_grandchild"
    );
    assert_eq!(
        &grandchild.path_from_id_w_sep(parent, c"_", c"::").unwrap(),
        "child_grandchild"
    );
}

#[test]
fn entity_path_from_type_custom_sep() {
    let world = create_world();

    let parent = world.entity_from::<Parent>();
    world.set_scope_id(parent.id());
    let child = world.entity_named(c"child");
    world.set_scope_id(child.id());
    let grandchild = world.entity_named(c"grandchild");

    assert_eq!(
        &grandchild.path_w_sep(c"_", c"?").unwrap(),
        "?entity_test_common_Parent_child_grandchild"
    );
    assert_eq!(
        &grandchild.path_from_id_w_sep(parent, c"_", c"::").unwrap(),
        "child_grandchild"
    );
}

#[test]
fn entity_implicit_path_to_char() {
    let world = create_world();

    let entity = world.entity_named(c"Foo::Bar");
    assert!(entity.is_valid());
    assert_eq!(entity.name(), "Bar");
    assert_eq!(entity.path().unwrap(), "::Foo::Bar");
}

#[test]
fn entity_implicit_type_str_to_char() {
    let world = create_world();

    let entity = world.entity_named(c"Foo");
    assert!(entity.is_valid());

    assert_eq!(entity.archetype().to_string().unwrap(), "(Identifier,Name)");
}

#[test]
fn entityview_to_entity_to_entity_view() {
    let world = create_world();

    let entity = world.entity().set(Position { x: 10, y: 20 });
    assert!(entity.is_valid());

    let entity_id = entity.id();

    let entity_view = entity_id.entity_view(&world);
    assert!(entity_view.is_valid());
    assert_eq!(entity, entity_view);

    let p = entity_view.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_entity_view_to_entity_world() {
    let world = create_world();
    let entity = world.entity().set(Position { x: 10, y: 20 });
    assert!(entity.is_valid());
    let entity_id = entity.id();

    let entity_view = entity_id.entity_view(&world);
    assert!(entity_view.is_valid());
    assert_eq!(entity, entity_view);

    let entity_mut = entity_view.mut_current_stage(&world);
    entity_mut.set(Position { x: 10, y: 20 });

    assert!(entity_view.has::<Position>());
    let p = entity_view.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_entity_view_to_entity_stage() {
    let world = create_world();

    let entity_view: EntityView = world.entity();
    let stage = world.stage(0);

    world.readonly_begin(false);

    let entity_mut = entity_view.mut_current_stage(stage);
    entity_mut.set(Position { x: 10, y: 20 });
    assert!(!entity_mut.has::<Position>());

    world.readonly_end();

    assert!(entity_mut.has::<Position>());
    assert!(entity_view.has::<Position>());

    let p = entity_view.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_create_entity_view_from_stage() {
    let world = create_world();
    let stage = world.stage(0);

    world.readonly_begin(false);
    let entity_view: EntityView = stage.entity();

    world.readonly_end();

    let entity_mut = entity_view.mut_current_stage(&world);
    entity_mut.set(Position { x: 10, y: 20 });
    assert!(entity_view.has::<Position>());

    let p = entity_view.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_set_template() {
    let world = create_world();
    let entity = world.entity().set(Template::<Position> {
        value: Position { x: 10, y: 20 },
    });

    let pos = entity.try_get::<Template<Position>>().unwrap();
    assert_eq!(pos.value.x, 10);
    assert_eq!(pos.value.y, 20);
}

#[test]
fn entity_get_1_component_w_callback() {
    let world = create_world();
    let e_1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });
    let e_2 = world.entity().set(Position { x: 11, y: 22 });
    let e_3 = world.entity().set(Velocity { x: 1, y: 2 });

    assert!(e_1.get_callback::<Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }));

    assert!(e_2.get_callback::<Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    }));

    assert!(!e_3.get_callback::<Position>(|_| {}));
}

#[test]
#[ignore = "get callback not implemented for multiple components"]
fn entity_get_2_components_w_callback() {
    // let world = create_world();
    // let e_1 = world
    //     .entity()
    //     .set(Position { x: 10, y: 20 })
    //     .set(Velocity { x: 1, y: 2 });
    // let e_2 = world.entity().set(Position { x: 11, y: 22 });
    // let e_3 = world.entity().set(Velocity { x: 1, y: 2 });

    // TODO get_callback does not support multiple components
}

#[test]
fn entity_get_mut_1_component_w_callback() {
    let world = create_world();
    let e_1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });
    let e_2 = world.entity().set(Position { x: 11, y: 22 });
    let e_3 = world.entity().set(Velocity { x: 1, y: 2 });

    assert!(e_1.get_callback_mut::<Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        p.x += 1;
        p.y += 2;
    }));

    assert!(e_2.get_callback_mut::<Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
        p.x += 1;
        p.y += 2;
    }));

    assert!(!e_3.get_callback_mut::<Position>(|_| {}));

    let p = e_1.try_get::<Position>().unwrap();
    assert_eq!(p.x, 11);
    assert_eq!(p.y, 22);

    let p = e_2.try_get::<Position>().unwrap();
    assert_eq!(p.x, 12);
    assert_eq!(p.y, 24);
}

#[test]
#[ignore = "get callback not implemented for multiple components"]
fn entity_get_mut_2_components_w_callback() {
    // multiple components not supported in get_callback (for now)
}

#[test]
fn entity_get_component_w_callback_nested() {
    let world = create_world();

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    assert!(e.get_callback::<Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);

        assert!(e.get_callback::<Velocity>(|v| {
            assert_eq!(v.x, 1);
            assert_eq!(v.y, 2);
        }));
    }));
}

#[test]
fn entity_get_mut_component_w_callback_nested() {
    let world = create_world();

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    assert!(e.get_callback_mut::<Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);

        assert!(e.get_callback_mut::<Velocity>(|v| {
            assert_eq!(v.x, 1);
            assert_eq!(v.y, 2);
        }));
    }));
}

// TODO set callbacks

#[test]
fn entity_defer_set_1_component() {
    let world = create_world();

    world.defer_begin();

    let e = world.entity().set(Position { x: 10, y: 20 });

    assert!(!e.has::<Position>());

    world.defer_end();

    assert!(e.has::<Position>());

    let p = e.try_get::<Position>().unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_defer_set_2_components() {
    let world = create_world();

    world.defer_begin();

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    assert!(!e.has::<Position>());
    assert!(!e.has::<Velocity>());

    world.defer_end();

    assert!(e.has::<Position>());
    assert!(e.has::<Velocity>());

    let p = e.get::<Position>();
    let v = e.get::<Velocity>();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
    assert_eq!(v.x, 1);
    assert_eq!(v.y, 2);
}

#[test]
fn entity_defer_set_3_components() {
    let world = create_world();

    world.defer_begin();

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 50 });

    assert!(!e.has::<Position>());
    assert!(!e.has::<Velocity>());
    assert!(!e.has::<Mass>());

    world.defer_end();

    assert!(e.has::<Position>());
    assert!(e.has::<Velocity>());
    assert!(e.has::<Mass>());

    let p = e.get::<Position>();
    let v = e.get::<Velocity>();
    let m = e.get::<Mass>();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
    assert_eq!(v.x, 1);
    assert_eq!(v.y, 2);
    assert_eq!(m.value, 50);
}

#[test]
fn entity_set_2_w_on_set() {
    let world = create_world();

    world
        .observer::<&Position>()
        .add_event::<flecs::OnSet>()
        .each_entity(|entity, p| {
            entity.world().get_mut::<Flags>().insert("position_set", 1);
            assert_eq!(p.x, 10);
            assert_eq!(p.y, 20);
        });

    world
        .observer::<&Velocity>()
        .add_event::<flecs::OnSet>()
        .each_entity(|entity, v| {
            entity.world().get_mut::<Flags>().insert("velocity_set", 1);
            assert_eq!(v.x, 1);
            assert_eq!(v.y, 2);
        });

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    assert!(world.get::<Flags>().contains_key("position_set"));
    assert!(world.get::<Flags>().contains_key("velocity_set"));

    let p = e.get::<Position>();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let v = e.get::<Velocity>();
    assert_eq!(v.x, 1);
    assert_eq!(v.y, 2);
}

#[test]
fn entity_defer_set_2_w_on_set() {
    let world = create_world();

    world
        .observer::<&Position>()
        .add_event::<flecs::OnSet>()
        .each_entity(|e, p| {
            e.world().get_mut::<Flags>().insert("position_set", 1);
            assert_eq!(p.x, 10);
            assert_eq!(p.y, 20);
        });

    world
        .observer::<&Velocity>()
        .add_event::<flecs::OnSet>()
        .each_entity(|e, v| {
            e.world().get_mut::<Flags>().insert("velocity_set", 1);
            assert_eq!(v.x, 1);
            assert_eq!(v.y, 2);
        });

    world.defer_begin();

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    assert!(!world.get::<Flags>().contains_key("position_set"));
    assert!(!world.get::<Flags>().contains_key("velocity_set"));

    world.defer_end();

    assert!(world.get::<Flags>().contains_key("position_set"));
    assert!(world.get::<Flags>().contains_key("velocity_set"));

    let p = e.get::<Position>();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let v = e.get::<Velocity>();
    assert_eq!(v.x, 1);
    assert_eq!(v.y, 2);
}

#[test]
fn entity_set_2_after_set_1() {
    let world = create_world();

    let e = world.entity().set(Position { x: 5, y: 10 });

    assert!(e.has::<Position>());

    if let Some(p) = e.try_get::<Position>() {
        assert_eq!(p.x, 5);
        assert_eq!(p.y, 10);
    }

    e.set(Position { x: 10, y: 20 });
    e.set(Velocity { x: 1, y: 2 });

    if let Some(p) = e.try_get::<Position>() {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }

    if let Some(v) = e.try_get::<Velocity>() {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    }
}

#[test]
fn entity_set_2_after_set_2() {
    let world = create_world();

    let e = world
        .entity()
        .set(Position { x: 5, y: 10 })
        .set(Velocity { x: 1, y: 2 });

    assert!(e.has::<Position>());
    assert!(e.has::<Velocity>());

    if let Some(p) = e.try_get::<Position>() {
        assert_eq!(p.x, 5);
        assert_eq!(p.y, 10);
    }

    if let Some(v) = e.try_get::<Velocity>() {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    }

    e.set(Position { x: 10, y: 20 });
    e.set(Velocity { x: 3, y: 4 });

    if let Some(p) = e.try_get::<Position>() {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }

    if let Some(v) = e.try_get::<Velocity>() {
        assert_eq!(v.x, 3);
        assert_eq!(v.y, 4);
    }
}

#[test]
fn entity_with_self() {
    let world = create_world();

    let tag = world.entity();
    tag.with(|| {
        let e1 = world.entity();
        e1.set(SelfRef { value: e1.into() });

        let e2 = world.entity();
        e2.set(SelfRef { value: e2.into() });

        let e3 = world.entity();
        e3.set(SelfRef { value: e3.into() });
    });

    // Ensures that while Self is (implicitly) registered within the with, it
    // does not get the tag.
    assert!(!world.component::<SelfRef>().has_id(tag));

    let mut count = 0;
    let q = world.query::<()>().with_id(tag).build();

    q.each_entity(|e, _| {
        assert!(e.has_id(tag));

        if let Some(s) = e.try_get::<SelfRef>() {
            assert_eq!(s.value, e);
        }

        count += 1;
    });

    assert_eq!(count, 3);
}

#[test]
fn entity_with_relation_type_self() {
    let world = create_world();

    let bob = world.entity().with_first::<Likes>(|| {
        let e1 = world.entity();
        e1.set(SelfRef { value: e1.into() });

        let e2 = world.entity();
        e2.set(SelfRef { value: e2.into() });

        let e3 = world.entity();
        e3.set(SelfRef { value: e3.into() });
    });

    assert!(!world.component::<SelfRef>().has_first::<Likes>(bob));

    let mut count = 0;
    let q = world.query::<()>().with_first::<&Likes>(bob).build();

    q.each_entity(|e, _| {
        assert!(e.has_first::<Likes>(bob));

        if let Some(s) = e.try_get::<SelfRef>() {
            assert_eq!(s.value, e);
        }

        count += 1;
    });

    assert_eq!(count, 3);
}

#[test]
fn entity_with_relation_self() {
    let world = create_world();

    let bob = world.entity().with_first::<Likes>(|| {
        let e1 = world.entity();
        e1.set(SelfRef { value: e1.into() });

        let e2 = world.entity();
        e2.set(SelfRef { value: e2.into() });

        let e3 = world.entity();
        e3.set(SelfRef { value: e3.into() });
    });

    assert!(!world.component::<SelfRef>().has_first::<Likes>(bob));

    let mut count = 0;
    let q = world.query::<()>().with_first::<&Likes>(bob).build();

    q.each_entity(|e, _| {
        assert!(e.has_first::<Likes>(bob));

        if let Some(s) = e.try_get::<SelfRef>() {
            assert_eq!(s.value, e);
        }

        count += 1;
    });

    assert_eq!(count, 3);
}

#[test]
fn entity_with_self_w_name() {
    let world = create_world();

    let tier1 = world.entity_named(c"Tier1").with(|| {
        let tier2 = world.entity_named(c"Tier2");
        tier2.set(SelfRef {
            value: tier2.into(),
        });
    });
    let tier2 = world.try_lookup(c"Tier2");
    assert!(tier2.is_some());
    let tier2 = tier2.unwrap();
    assert!(tier2.has_id(tier1));
}

#[test]
fn entity_with_self_nested() {
    let world = create_world();

    let tier1 = world.entity_named(c"Tier1").with(|| {
        world.entity_named(c"Tier2").with(|| {
            world.entity_named(c"Tier3");
        });
    });

    let tier2 = world.try_lookup(c"Tier2").unwrap();
    let tier3 = world.try_lookup(c"Tier3").unwrap();

    assert!(tier2.has_id(tier1));
    assert!(tier3.has_id(tier2));
}

#[test]
fn entity_with_scope() {
    let world = create_world();

    let parent = world.entity_named(c"P").scope(|_| {
        let e1 = world.entity_named(c"C1");
        e1.set(SelfRef { value: e1.into() });
        let e2 = world.entity_named(c"C2");
        e2.set(SelfRef { value: e2.into() });
        let e3 = world.entity_named(c"C3");
        e3.set(SelfRef { value: e3.into() });

        assert_eq!(world.lookup(c"C1"), e1);
        assert_eq!(world.lookup(c"C2"), e2);
        assert_eq!(world.lookup(c"C3"), e3);
        assert_eq!(world.lookup(c"::P::C1"), e1);
        assert_eq!(world.lookup(c"::P::C2"), e2);
        assert_eq!(world.lookup(c"::P::C3"), e3);
    });

    // Ensure entities are created in correct scope
    assert!(world.try_lookup(c"C1").is_none());
    assert!(world.try_lookup(c"C2").is_none());
    assert!(world.try_lookup(c"C3").is_none());

    assert!(parent.try_lookup(c"C1").is_some());
    assert!(parent.try_lookup(c"C2").is_some());
    assert!(parent.try_lookup(c"C3").is_some());

    assert_eq!(world.lookup(c"P::C1"), parent.lookup(c"C1"));
    assert_eq!(world.lookup(c"P::C2"), parent.lookup(c"C2"));
    assert_eq!(world.lookup(c"P::C3"), parent.lookup(c"C3"));

    // Ensures that while self is (implicitly) registered within the with, it
    // does not become a child of the parent.
    assert!(!world
        .component::<SelfRef>()
        .has_id((flecs::ChildOf::ID, parent)));

    let mut count = 0;
    let q = world
        .query::<()>()
        .with_id((*flecs::ChildOf, parent))
        .build();

    q.each_entity(|e, _| {
        assert!(e.has_id((*flecs::ChildOf, parent)));

        if let Some(s) = e.try_get::<SelfRef>() {
            assert_eq!(s.value, e);
        }

        count += 1;
    });

    assert_eq!(count, 3);
}

#[test]
fn entity_with_scope_nested() {
    let world = create_world();

    let parent = world.entity_named(c"P").scope(|world| {
        let child = world.entity_named(c"C").scope(|world| {
            let grandchild = world.entity_named(c"GC");
            assert_eq!(grandchild, world.lookup(c"GC"));
            assert_eq!(grandchild, world.lookup(c"::P::C::GC"));
        });

        assert_eq!(world.lookup(c"C"), child);
        assert_eq!(world.lookup(c"::P::C"), child);
    });

    assert!(world.try_lookup(c"C").is_none());
    assert!(world.try_lookup(c"GC").is_none());
    assert!(world.try_lookup(c"C::GC").is_none());

    let child = world.lookup(c"P::C");
    assert!(child.has_id((flecs::ChildOf::ID, parent)));

    let grandchild = world.lookup(c"P::C::GC");
    assert!(grandchild.has_id((flecs::ChildOf::ID, child)));
}

#[test]
fn entity_with_scope_nested_same_name_as_parent() {
    let world = create_world();

    let parent = world.entity_named(c"P").scope(|world| {
        let child = world.entity_named(c"C").scope(|world| {
            let gchild = world.entity_named(c"C");
            assert_eq!(gchild, world.lookup(c"C"));
            assert_eq!(gchild, world.lookup(c"::P::C::C"));
        });

        assert_eq!(world.lookup(c"C"), child);
        assert_eq!(world.lookup(c"::P::C"), child);
    });

    assert!(world.try_lookup(c"C").is_none());
    assert!(world.try_lookup(c"C::C").is_none());

    let child = world.lookup(c"P::C");
    assert!(child.has_id((flecs::ChildOf::ID, parent)));

    let gchild = world.lookup(c"P::C::C");
    assert!(gchild.has_id((flecs::ChildOf::ID, child)));
}

#[test]
fn entity_no_recursive_lookup() {
    let world = create_world();

    let p = world.entity_named(c"P");
    let c = world.entity_named(c"C").child_of_id(p);
    let gc = world.entity_named(c"GC").child_of_id(c);

    assert_eq!(c.lookup_current_scope(c"GC"), gc);
    assert!(c.try_lookup_current_scope(c"C").is_none());
    assert!(c.try_lookup_current_scope(c"P").is_none());
}

#[test]
fn entity_defer_new_w_name() {
    let world = create_world();
    let mut e = EntityView::new_null(&world);

    world.defer(|| {
        e = world.entity_named(c"Foo");
        assert!(e.is_valid());
    });

    assert!(e.has_first::<flecs::EcsIdentifier>(flecs::Name::ID));
    assert_eq!(e.name(), "Foo");
}

#[test]
fn entity_defer_new_w_nested_name() {
    let world = create_world();
    let mut e = EntityView::new_null(&world);

    world.defer(|| {
        e = world.entity_named(c"Foo::Bar");
        assert!(e.is_valid());
    });

    assert!(e.has_first::<flecs::EcsIdentifier>(flecs::Name::ID));
    assert_eq!(e.name(), "Bar");
    assert_eq!(e.path().unwrap(), "::Foo::Bar");
}

#[test]
fn entity_defer_new_w_scope_name() {
    let world = create_world();
    let parent = world.entity_named(c"Parent");
    let mut e = EntityView::new_null(&world);

    world.defer(|| {
        parent.scope(|_w| {
            e = world.entity_named(c"Foo");
            assert!(e.is_valid());
        });
    });

    assert!(e.has_first::<flecs::EcsIdentifier>(flecs::Name::ID));
    assert_eq!(e.name(), "Foo");
    assert_eq!(e.path().unwrap(), "::Parent::Foo");
}

#[test]
fn entity_defer_new_w_scope_nested_name() {
    let world = create_world();
    let parent = world.entity_named(c"Parent");
    let mut e = EntityView::new_null(&world);

    world.defer(|| {
        parent.scope(|_w| {
            e = world.entity_named(c"Foo::Bar");
            assert!(e.is_valid());
        });
    });

    assert!(e.has_first::<flecs::EcsIdentifier>(flecs::Name::ID));
    assert_eq!(e.name(), "Bar");
    assert_eq!(e.path().unwrap(), "::Parent::Foo::Bar");
}

#[test]
fn entity_defer_new_w_scope() {
    let world = create_world();

    let parent = world.entity();
    let mut e = EntityView::new_null(&world);

    world.defer(|| {
        parent.scope(|_w| {
            e = world.entity();
            assert!(e.is_valid());
        });
    });

    assert!(e.has_first::<flecs::ChildOf>(parent));
}

#[test]
fn entity_defer_new_w_with() {
    let world = create_world();
    let mut e = EntityView::new_null(&world);
    let tag = world.entity();

    world.defer(|| {
        tag.with(|| {
            e = world.entity();
            assert!(e.is_valid());
            assert!(!e.has_id(tag));
        });
    });

    assert!(e.has_id(tag));
}

#[test]
fn entity_defer_new_w_name_scope_with() {
    let world = create_world();
    let tag = world.entity();
    let mut e = EntityView::new_null(&world);
    let parent = world.entity_named(c"Parent");

    world.defer(|| {
        tag.with(|| {
            parent.scope(|_w| {
                e = world.entity_named(c"Foo");
                assert!(e.is_valid());
                assert!(!e.has_id(tag));
            });

            assert!(!e.has_id(tag));
        });

        assert!(!e.has_id(tag));
    });

    assert!(e.has_id(tag));
    assert!(e.has_first::<flecs::EcsIdentifier>(flecs::Name::ID));
    assert_eq!(e.name(), "Foo");
    assert_eq!(e.path().unwrap(), "::Parent::Foo");
}

#[test]
fn entity_defer_new_w_nested_name_scope_with() {
    let world = create_world();
    let tag = world.entity();
    let parent = world.entity_named(c"Parent");
    let mut e = EntityView::new_null(&world);

    world.defer(|| {
        tag.with(|| {
            parent.scope(|_w| {
                e = world.entity_named(c"Foo::Bar");
                assert!(e.is_valid());
                assert!(!e.has_id(tag));
            });

            assert!(!e.has_id(tag));
        });

        assert!(!e.has_id(tag));
    });

    assert!(e.has_id(tag));
    assert!(e.has_first::<flecs::EcsIdentifier>(flecs::Name::ID));
    assert_eq!(e.name(), "Bar");
    assert_eq!(e.path().unwrap(), "::Parent::Foo::Bar");
}

#[test]
fn entity_defer_w_with_implicit_component() {
    let world = create_world();
    let mut e = EntityView::new_null(&world);

    world.defer(|| {
        world.with::<Tag>(|| {
            e = world.entity();
            assert!(!e.has::<Tag>());
        });

        assert!(!e.has::<Tag>());
    });

    assert!(e.has::<Tag>());
}

#[test]
fn entity_defer_suspend_resume() {
    let world = create_world();
    let e = world.entity();

    world.defer(|| {
        e.set(Position { x: 10, y: 20 });
        assert!(!e.has::<Position>());

        world.defer_suspend();
        e.set(Velocity { x: 1, y: 2 });
        assert!(!e.has::<Position>());
        assert!(e.has::<Velocity>());
        world.defer_resume();

        assert!(!e.has::<Position>());
        assert!(e.has::<Velocity>());
    });

    assert!(e.has::<Position>());
    assert!(e.has::<Velocity>());
}

#[test]
fn entity_with_after_builder_method() {
    let world = create_world();

    let a = world.entity().set(Position { x: 10, y: 20 }).with(|| {
        world.entity_named(c"X");
    });

    let b = world
        .entity()
        .set(Position { x: 30, y: 40 })
        .with_first::<Likes>(|| {
            world.entity_named(c"Y");
        });

    let c = world
        .entity()
        .set(Position { x: 50, y: 60 })
        .with_first_id(*flecs::IsA, || {
            world.entity_named(c"Z");
        });

    let pos = a.get::<Position>();
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);

    let pos = b.get::<Position>();
    assert_eq!(pos.x, 30);
    assert_eq!(pos.y, 40);

    let pos = c.get::<Position>();
    assert_eq!(pos.x, 50);
    assert_eq!(pos.y, 60);

    let x = world.lookup(c"X");
    assert!(x.has_id(a));

    let y = world.lookup(c"Y");
    assert!(y.has_first::<Likes>(b));

    let z = world.lookup(c"Z");
    assert!(z.has_id((*flecs::IsA, c)));
}

#[test]
fn entity_with_before_builder_method() {
    let world = create_world();

    let a = world
        .entity()
        .with(|| {
            world.entity_named(c"X");
        })
        .set(Position { x: 10, y: 20 });

    let b = world
        .entity()
        .with_first::<Likes>(|| {
            world.entity_named(c"Y");
        })
        .set(Position { x: 30, y: 40 });

    let c = world
        .entity()
        .with_first_id(*flecs::IsA, || {
            world.entity_named(c"Z");
        })
        .set(Position { x: 50, y: 60 });

    let pos = a.get::<Position>();
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);

    let pos = b.get::<Position>();
    assert_eq!(pos.x, 30);
    assert_eq!(pos.y, 40);

    let pos = c.get::<Position>();
    assert_eq!(pos.x, 50);
    assert_eq!(pos.y, 60);

    let x = world.lookup(c"X");
    assert!(x.has_id(a));

    let y = world.lookup(c"Y");
    assert!(y.has_first::<Likes>(b));

    let z = world.lookup(c"Z");
    assert!(z.has_id((*flecs::IsA, c)));
}

#[test]
fn entity_scope_after_builder_method() {
    let world = create_world();

    world
        .entity_named(c"P")
        .set(Position { x: 10, y: 20 })
        .scope(|_| {
            world.entity_named(c"C");
        });

    let c = world.lookup(c"P::C");
    assert!(c.is_valid());
}

#[test]
fn entity_scope_before_builder_method() {
    let world = create_world();

    world
        .entity_named(c"P")
        .scope(|_| {
            world.entity_named(c"C");
        })
        .set(Position { x: 10, y: 20 });

    let c = world.lookup(c"P::C");
    assert!(c.is_valid());
}

#[test]
fn entity_emplace() {
    let world = create_world();

    let e = world.entity().emplace(Position { x: 10, y: 20 });
    assert!(e.has::<Position>());

    let p = e.get::<Position>();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn entity_entity_id_str() {
    let world = create_world();

    let id = world.entity_named(c"Foo");
    assert_eq!(id.to_str(), "Foo");
}

#[test]
fn entity_pair_id_str() {
    let world = create_world();

    let id = world.id_from_id((world.entity_named(c"Rel"), world.entity_named(c"Obj")));

    assert_eq!(id.to_str(), "(Rel,Obj)");
}

#[test]
fn entity_role_id_str() {
    let world = create_world();

    let id = world.id_from_id(flecs::Override::ID | world.entity_named(c"Foo").id());

    assert_eq!(id.to_str(), "OVERRIDE|Foo");
}

#[test]
fn entity_id_str_from_entity_view() {
    let world = create_world();

    let id = world.entity_named(c"Foo");
    assert_eq!(id.to_str(), "Foo");
}

#[test]
fn entity_id_str_from_entity() {
    let world = create_world();

    let id = world.entity_named(c"Foo");
    assert_eq!(id.to_str(), "Foo");
}

#[test]
fn entity_null_entity_w_world() {
    let world = create_world();
    let e = EntityView::new_null(&world);
    assert_eq!(e.id(), 0);
}

#[test]
fn entity_is_wildcard() {
    let world = create_world();

    let e1 = world.entity();
    let e2 = world.entity();

    let p0 = e1;
    let p1 = world.id_from_id((e1, e2));
    let p2 = world.id_from_id((e1, *flecs::Wildcard));
    let p3 = world.id_from_id((*flecs::Wildcard, e2));
    let p4 = world.id_from_id((*flecs::Wildcard, *flecs::Wildcard));

    assert!(!e1.is_wildcard());
    assert!(!e2.is_wildcard());
    assert!(!p0.is_wildcard());
    assert!(!p1.is_wildcard());
    assert!(p2.is_wildcard());
    assert!(p3.is_wildcard());
    assert!(p4.is_wildcard());
}

#[test]
fn entity_has_id_t() {
    let world = create_world();

    let id_1 = world.entity();
    let id_2 = world.entity();

    let e = world.entity().add_id(id_1);

    assert!(e.has_id(id_1));
    assert!(!e.has_id(id_2));
}

#[test]
fn entity_has_pair_id_t() {
    let world = create_world();

    let id_1 = world.entity();
    let id_2 = world.entity();
    let id_3 = world.entity();

    let e = world.entity().add_id((id_1, id_2));

    assert!(e.has_id((id_1, id_2)));
    assert!(!e.has_id((id_1, id_3)));
}

#[test]
fn entity_has_pair_id_t_w_type() {
    let world = create_world();

    let id_2 = world.entity();
    let id_3 = world.entity();

    let e = world.entity().add_first::<Rel>(id_2);

    assert!(e.has_first::<Rel>(id_2));
    assert!(!e.has_first::<Rel>(id_3));
}

#[test]
fn entity_has_id() {
    let world = create_world();

    let id_1 = world.entity();
    let id_2 = world.entity();

    let e = world.entity().add_id(id_1);

    assert!(e.has_id(id_1));
    assert!(!e.has_id(id_2));
}

#[test]
fn entity_has_pair_id() {
    let world = create_world();

    let id_1 = world.entity();
    let id_2 = world.entity();
    let id_3 = world.entity();

    let e = world.entity().add_id((id_1, id_2));

    assert!(e.has_id((id_1, id_2)));
    assert!(!e.has_id((id_1, id_3)));
}

#[test]
fn entity_has_pair_id_w_type() {
    let world = create_world();

    let id_2 = world.entity();
    let id_3 = world.entity();

    let e = world.entity().add_first::<Rel>(id_2);

    assert!(e.has_first::<Rel>(id_2));
    assert!(!e.has_first::<Rel>(id_3));
}

#[test]
fn entity_has_wildcard_id() {
    let world = create_world();

    let id = world.entity();

    let e1 = world.entity().add_id(id);
    let e2 = world.entity();

    assert!(e1.has_id(*flecs::Wildcard));
    assert!(!e2.has_id(*flecs::Wildcard));
}

#[test]
fn entity_has_wildcard_pair_id() {
    let world = create_world();

    let rel = world.entity();
    let obj = world.entity();
    let obj_2 = world.entity();

    let w1 = world.id_from_id((rel, *flecs::Wildcard));
    let w2 = world.id_from_id((*flecs::Wildcard, obj));

    let e1 = world.entity().add_id((rel, obj));
    let e2 = world.entity().add_id((rel, obj_2));

    assert!(e1.has_id(w1));
    assert!(e1.has_id(w2));
    assert!(e2.has_id(w1));
    assert!(!e2.has_id(w2));
}

#[test]
fn entity_owns_id_t() {
    let world = create_world();

    let id_1 = world.entity();
    let id_2 = world.entity();

    let e = world.entity().add_id(id_1);

    assert!(e.owns_id(id_1));
    assert!(!e.owns_id(id_2));
}

#[test]
fn entity_owns_pair_id_t() {
    let world = create_world();

    let id_1 = world.entity();
    let id_2 = world.entity();
    let id_3 = world.entity();

    let e = world.entity().add_id((id_1, id_2));

    assert!(e.owns_id((id_1, id_2)));
    assert!(!e.owns_id((id_1, id_3)));
}

#[test]
fn entity_owns_pair_id_t_w_type() {
    let world = create_world();

    let id_2 = world.entity();
    let id_3 = world.entity();

    let e = world.entity().add_first::<Rel>(id_2);

    assert!(e.owns_first::<Rel>(id_2));
    assert!(!e.owns_first::<Rel>(id_3));
}

#[test]
fn entity_owns_id() {
    let world = create_world();

    let id_1 = world.entity();
    let id_2 = world.entity();

    let e = world.entity().add_id(id_1);

    assert!(e.owns_id(id_1));
    assert!(!e.owns_id(id_2));
}

#[test]
fn entity_owns_pair_id() {
    let world = create_world();

    let id_1 = world.entity();
    let id_2 = world.entity();
    let id_3 = world.entity();

    let e = world.entity().add_id((id_1, id_2));

    assert!(e.owns_id((id_1, id_2)));
    assert!(!e.owns_id((id_1, id_3)));
}

#[test]
fn entity_owns_wildcard_id() {
    let world = create_world();

    let id = world.entity();

    let e1 = world.entity().add_id(id);
    let e2 = world.entity();

    assert!(e1.owns_id(*flecs::Wildcard));
    assert!(!e2.owns_id(*flecs::Wildcard));
}

#[test]
fn entity_owns_wildcard_pair() {
    let world = create_world();

    let rel = world.entity();
    let obj = world.entity();
    let obj_2 = world.entity();

    let w1 = world.id_from_id((rel, *flecs::Wildcard));
    let w2 = world.id_from_id((*flecs::Wildcard, obj));

    let e1 = world.entity().add_id((rel, obj));
    let e2 = world.entity().add_id((rel, obj_2));

    assert!(e1.owns_id(w1));
    assert!(e1.owns_id(w2));
    assert!(e2.owns_id(w1));
    assert!(!e2.owns_id(w2));
}

#[test]
fn entity_owns_pair_id_w_type() {
    let world = create_world();

    let id_2 = world.entity();
    let id_3 = world.entity();

    let e = world.entity().add_first::<Rel>(id_2);

    assert!(e.owns_first::<Rel>(id_2));
    assert!(!e.owns_first::<Rel>(id_3));
}

#[test]
fn entity_id_from_world() {
    let world = create_world();

    let e = world.entity();
    assert!(e.is_valid());

    let id_1 = world.id_from_id(e);
    assert!(id_1.is_valid());
    assert_eq!(id_1, e.id());
    assert_eq!(id_1.world().ptr_mut(), world.ptr_mut());
    assert!(!id_1.is_pair());
    assert!(!id_1.is_wildcard());

    let id_2 = world.id_from_id(*flecs::Wildcard);
    assert!(id_2.is_valid());
    assert_eq!(id_2, *flecs::Wildcard);
    assert_eq!(id_2.world().ptr_mut(), world.ptr_mut());
    assert!(!id_2.is_pair());
    assert!(id_2.is_wildcard());
}

#[test]
fn entity_id_pair_from_world() {
    let world = create_world();

    let rel = world.entity();
    assert!(rel.is_valid());

    let obj = world.entity();
    assert!(obj.is_valid());

    let id_1 = world.id_from_id((rel, obj));
    assert_eq!(id_1.first_id(), rel);
    assert_eq!(id_1.second_id(), obj);
    assert_eq!(id_1.world().ptr_mut(), world.ptr_mut());
    assert!(id_1.is_pair());
    assert!(!id_1.is_wildcard());

    let id_2 = world.id_from_id((rel, *flecs::Wildcard));
    assert_eq!(id_2.first_id(), rel);
    assert_eq!(id_2.second_id(), *flecs::Wildcard);
    assert_eq!(id_2.world().ptr_mut(), world.ptr_mut());
    assert!(id_2.is_pair());
    assert!(id_2.is_wildcard());
}

#[test]
fn entity_is_a_id() {
    let world = create_world();

    let base = world.entity();

    let e = world.entity().is_a_id(base);

    assert!(e.has_id((*flecs::IsA, base)));
}

#[test]
fn entity_is_a_w_type() {
    let world = create_world();

    let base = world.entity_from::<Prefab>();

    let e = world.entity().is_a::<Prefab>();

    assert!(e.has_id((*flecs::IsA, base)));
    assert!(e.has_second::<Prefab>(*flecs::IsA));
}

#[test]
fn entity_child_of_id() {
    let world = create_world();

    let base = world.entity();

    let e = world.entity().child_of_id(base);

    assert!(e.has_id((*flecs::ChildOf, base)));
}

#[test]
fn entity_child_of_w_type() {
    let world = create_world();

    let base = world.entity_from::<Parent>();

    let e = world.entity().child_of::<Parent>();

    assert!(e.has_id((*flecs::ChildOf, base)));
    assert!(e.has_second::<Parent>(*flecs::ChildOf));
}

#[test]
fn entity_slot_of() {
    let world = create_world();

    let base = world.prefab();
    let base_child = world.prefab().child_of_id(base).slot_of_id(base);

    assert!(base_child.has_id((*flecs::SlotOf, base)));

    let inst = world.entity().is_a_id(base);
    assert!(inst.has_id((base_child, *flecs::Wildcard)));
}

#[test]
fn entity_slot_of_w_type() {
    let world = create_world();

    let base = world.prefab_type::<Parent>();
    let base_child = world.prefab().child_of_id(base).slot_of::<Parent>();

    assert!(base_child.has_id((*flecs::SlotOf, base)));

    let inst = world.entity().is_a_id(base);
    assert!(inst.has_id((base_child, *flecs::Wildcard)));
}

#[test]
fn entity_slot() {
    let world = create_world();

    let base = world.prefab();
    let base_child = world.prefab().child_of_id(base).slot();

    assert!(base_child.has_id((*flecs::SlotOf, base)));

    let inst = world.entity().is_a_id(base);
    assert!(inst.has_id((base_child, *flecs::Wildcard)));
}

#[test]
fn entity_id_get_entity() {
    let world = create_world();

    let e = world.entity();

    let id = world.id_from_id(e);

    assert_eq!(id.entity_view(), e);
}

#[test]
#[should_panic]
fn entity_id_get_invalid_entity() {
    let world = create_world();

    let r = world.entity();
    let o = world.entity();

    let id = world.id_from_id((r, o));

    assert!(!id.entity_view().is_valid());
}

#[test]
fn entity_each_in_stage() {
    let world = create_world();

    let e = world.entity().add::<(Rel, Obj)>();
    assert!(e.has::<(Rel, Obj)>());

    world.readonly_begin(false);

    let s = world.stage(0);
    let em = e.mut_current_stage(s);
    assert!(em.has::<(Rel, Obj)>());
    let mut count = 0;

    em.each_target::<Rel>(|obj| {
        count += 1;
        assert_eq!(obj, world.entity_from::<Obj>());
    });

    assert_eq!(count, 1);

    world.readonly_end();
}

#[test]
fn entity_iter_recycled_parent() {
    let world = create_world();

    let e = world.entity();
    e.destruct();

    let e2 = world.entity();
    assert_ne!(e, e2);
    assert_eq!(*e.id() as u32, *e2.id() as u32);

    let e_child = world.entity().child_of_id(e2);
    let mut count = 0;

    e2.each_child(|child| {
        count += 1;
        assert_eq!(child, e_child);
    });

    assert_eq!(count, 1);
}

#[test]
fn entity_get_obj_by_template() {
    let world = create_world();

    let e1 = world.entity();
    let o1 = world.entity();
    let o2 = world.entity();

    e1.add_first::<Rel>(o1);
    e1.add_first::<Rel>(o2);

    assert_eq!(o1, e1.target::<Rel>(0));
    assert_eq!(o2, e1.target::<Rel>(1));
}

#[test]
fn entity_create_named_twice_deferred() {
    let world = create_world();

    world.defer_begin();

    let e1 = world.entity_named(c"e");
    let e2 = world.entity_named(c"e");

    let f1 = world.entity_named(c"p::f");
    let f2 = world.entity_named(c"p::f");

    world.entity_named(c"q").scope(|_w| {
        world.entity_named(c"g");
    });

    world.defer_end();

    assert_eq!(e1.path().unwrap(), "::e");
    assert_eq!(f1.path().unwrap(), "::p::f");
    assert!(world.try_lookup(c"::q::g").is_some());

    assert_eq!(e1, e2);
    assert_eq!(f1, f2);
}

#[test]
fn entity_clone() {
    let world = create_world();

    let v = PositionClone { x: 10, y: 20 };

    let src = world.entity().add::<TagClone>().set(v);
    let dst = src.duplicate(false);
    assert!(dst.has::<TagClone>());
    assert!(dst.has::<PositionClone>());

    let ptr = dst.try_get::<PositionClone>();
    assert!(ptr.is_some());
    assert_ne!(ptr.unwrap().x, 10);
    assert_ne!(ptr.unwrap().y, 20);
}

#[test]
fn entity_clone_w_value() {
    let world = create_world();

    let v = PositionClone { x: 10, y: 20 };

    let src = world.entity().add::<TagClone>().set(v);
    let dst = src.duplicate(true);
    assert!(dst.has::<TagClone>());
    assert!(dst.has::<PositionClone>());

    let ptr = dst.try_get::<PositionClone>();
    assert!(ptr.is_some());
    assert_eq!(ptr.unwrap().x, 10);
    assert_eq!(ptr.unwrap().y, 20);
}

#[test]
fn entity_clone_to_existing() {
    let world = create_world();

    let v = PositionClone { x: 10, y: 20 };

    let src = world.entity().add::<TagClone>().set(v);
    let dst = world.entity();
    let result = src.duplicate_into(true, dst);
    assert_eq!(result, dst);

    assert!(dst.has::<TagClone>());
    assert!(dst.has::<PositionClone>());

    let ptr = dst.try_get::<PositionClone>();
    assert!(ptr.is_some());
    assert_eq!(ptr.unwrap().x, 10);
    assert_eq!(ptr.unwrap().y, 20);
}

#[test]
#[should_panic]
#[ignore = "Panic test: panics in C, which isn't captured by rust"]
fn entity_clone_to_existing_overlap() {
    let world = create_world();

    let v = PositionClone { x: 10, y: 20 };

    let src = world.entity().add::<TagClone>().set(v);
    let dst = world.entity().add::<PositionClone>();

    src.duplicate_into(true, dst);
}

// TODO set doc name test cases with doc addon

#[test]
fn entity_entity_w_root_name() {
    let world = create_world();

    let e = world.entity_named(c"::foo");
    assert_eq!(e.name(), "foo");
    assert_eq!(e.path().unwrap(), "::foo");
}

#[test]
fn entity_entity_w_root_name_from_scope() {
    let world = create_world();

    let p = world.entity_named(c"parent");
    world.set_scope_id(p);
    let e = world.entity_named(c"::foo");
    world.set_scope_id(0);

    assert_eq!(e.name(), "foo");
    assert_eq!(e.path().unwrap(), "::foo");
}

#[test]
fn entity_entity_w_type() {
    let world = create_world();

    let e = world.entity_from::<EntityType>();

    assert_eq!(e.name(), "EntityType");
    assert_eq!(e.path().unwrap(), "::entity_test::common::EntityType");
    //assert!(!e.has::<flecs::EcsComponent>());
    //TODO this assert should work, but we register it a bit different than cpp, no problem though.
    let e_2 = world.entity_from::<EntityType>();
    assert_eq!(e, e_2);
}

#[test]
fn entity_prefab_hierarchy_w_types() {
    let world = create_world();

    let turret = world.prefab_type::<Turret>();
    let turret_base = world
        .prefab_type::<Base>()
        .child_of::<Turret>()
        .slot_of::<Turret>();

    assert!(turret.is_valid());
    assert!(turret_base.is_valid());
    assert!(turret_base.has_id((*flecs::ChildOf, turret)));

    assert_eq!(turret.path().unwrap(), "::entity_test::common::Turret");
    assert_eq!(
        turret_base.path().unwrap(),
        "::entity_test::common::Turret::Base"
    );

    assert_eq!(turret.symbol(), "entity_test::common::Turret");
    assert_eq!(turret_base.symbol(), "entity_test::common::Base");

    let railgun = world.prefab_type::<Railgun>().is_a::<Turret>();
    let railgun_base = railgun.lookup(c"Base");
    let railgun_head = world
        .prefab_type::<Head>()
        .child_of::<Railgun>()
        .slot_of::<Railgun>();
    let railgun_beam = world
        .prefab_type::<Beam>()
        .child_of::<Railgun>()
        .slot_of::<Railgun>();

    assert!(railgun.is_valid());
    assert!(railgun_base.is_valid());
    assert!(railgun_head.is_valid());
    assert!(railgun_beam.is_valid());
    assert!(railgun_base.has_id((*flecs::ChildOf, railgun)));
    assert!(railgun_head.has_id((*flecs::ChildOf, railgun)));
    assert!(railgun_beam.has_id((*flecs::ChildOf, railgun)));

    assert_eq!(railgun.path().unwrap(), "::entity_test::common::Railgun");
    assert_eq!(
        railgun_base.path().unwrap(),
        "::entity_test::common::Railgun::Base"
    );
    assert_eq!(
        railgun_head.path().unwrap(),
        "::entity_test::common::Railgun::Head"
    );
    assert_eq!(
        railgun_beam.path().unwrap(),
        "::entity_test::common::Railgun::Beam"
    );

    assert_eq!(railgun.symbol(), "entity_test::common::Railgun");
    assert_eq!(railgun_head.symbol(), "entity_test::common::Head");
    assert_eq!(railgun_beam.symbol(), "entity_test::common::Beam");
}

#[test]
fn entity_prefab_hierarchy_w_root_types() {
    let world = create_world();

    let turret = world.prefab_type::<Turret>();
    let turret_base = world
        .prefab_type::<Base>()
        .child_of::<Turret>()
        .slot_of::<Turret>();

    assert!(turret.is_valid());
    assert!(turret_base.is_valid());
    assert!(turret_base.has_id((*flecs::ChildOf, turret)));

    assert_eq!(turret.path().unwrap(), "::entity_test::common::Turret");
    assert_eq!(
        turret_base.path().unwrap(),
        "::entity_test::common::Turret::Base"
    );

    assert_eq!(turret.symbol(), "entity_test::common::Turret");
    assert_eq!(turret_base.symbol(), "entity_test::common::Base");

    let inst = world.entity().is_a::<Turret>();
    assert!(inst.is_valid());

    let inst_base = inst.lookup(c"Base");
    assert!(inst_base.is_valid());
}

#[test]
fn entity_entity_array() {
    let world = create_world();

    let entities = [world.entity(), world.entity(), world.entity()];

    for e in entities.iter() {
        e.add::<TagA>().add::<TagB>();
    }

    assert_eq!(world.count::<TagA>(), 3);
    assert_eq!(world.count::<TagB>(), 3);
}

#[test]
fn entity_entity_w_type_defer() {
    let world = create_world();

    world.defer_begin();

    let e = world.entity_from::<Tag>();

    world.defer_end();

    assert_eq!(e.name(), "Tag");
    assert_eq!(e.symbol(), "entity_test::common::Tag");
    assert_eq!(world.id_from::<Tag>(), e);
}

#[test]
fn entity_add_if_true_t() {
    let world = create_world();

    let e = world.entity();

    e.add_if::<Tag>(true);
    assert!(e.has::<Tag>());
}

#[test]
fn entity_add_if_false_t() {
    let world = create_world();

    let e = world.entity();

    e.add_if::<Tag>(false);
    assert!(!e.has::<Tag>());

    e.add::<Tag>();
    assert!(e.has::<Tag>());
    e.add_if::<Tag>(false);
    assert!(!e.has::<Tag>());
}

#[test]
fn entity_add_if_true_id() {
    let world = create_world();

    let e = world.entity();
    let t = world.entity();

    e.add_id_if(t, true);
    assert!(e.has_id(t));
}

#[test]
fn entity_add_if_false_id() {
    let world = create_world();

    let e = world.entity();
    let t = world.entity();

    e.add_id_if(t, false);
    assert!(!e.has_id(t));

    e.add_id(t);
    assert!(e.has_id(t));
    e.add_id_if(t, false);
    assert!(!e.has_id(t));
}

#[test]
fn entity_add_if_true_r_o() {
    let world = create_world();

    let e = world.entity();

    e.add_if::<(Rel, Obj)>(true);
    assert!(e.has::<(Rel, Obj)>());
}

#[test]
fn entity_add_if_false_r_o() {
    let world = create_world();

    let e = world.entity();
    e.add_if::<(Rel2, Obj2)>(false);
    assert!(!e.has::<(Rel2, Obj2)>());
    e.add::<(Rel2, Obj2)>();
    assert!(e.has::<(Rel2, Obj2)>());
    e.add_if::<(Rel2, Obj2)>(false);
    assert!(!e.has::<(Rel2, Obj2)>());
}

#[test]
fn entity_add_if_true_r_o_2() {
    let world = create_world();

    let e = world.entity();
    let o = world.entity();

    e.add_first_if::<Rel>(o, true);
    assert!(e.has_first::<Rel>(o));
}

#[test]
fn entity_add_if_false_r_o_2() {
    let world = create_world();

    let e = world.entity();
    let o = world.entity();

    e.add_first_if::<Rel2>(o, false);
    assert!(!e.has_first::<Rel2>(o));
    e.add_first::<Rel2>(o);
    assert!(e.has_first::<Rel2>(o));
    e.add_first_if::<Rel2>(o, false);
    assert!(!e.has_first::<Rel2>(o));
}

#[test]
fn entity_add_if_true_r_o_3() {
    let world = create_world();

    let e = world.entity();
    let r = world.entity();
    let o = world.entity();

    e.add_id_if((r, o), true);
    assert!(e.has_id((r, o)));
}

#[test]
fn entity_add_if_false_r_o_3() {
    let world = create_world();

    let e = world.entity();
    let r = world.entity();
    let o = world.entity();

    e.add_id_if((r, o), false);
    assert!(!e.has_id((r, o)));
    e.add_id((r, o));
    assert!(e.has_id((r, o)));
    e.add_id_if((r, o), false);
    assert!(!e.has_id((r, o)));
}

#[test]
fn entity_add_if_exclusive_r_o() {
    let world = create_world();

    let e = world.entity();
    let r = world.entity().add::<flecs::Exclusive>();
    let o_1 = world.entity();
    let o_2 = world.entity();

    e.add_id((r, o_1));
    assert!(e.has_id((r, o_1)));

    e.add_id_if((r, o_2), true);
    assert!(!e.has_id((r, o_1)));
    assert!(e.has_id((r, o_2)));

    e.add_id_if((r, o_1), false);
    assert!(!e.has_id((r, o_1)));
    assert!(!e.has_id((r, o_2)));
}

#[test]
fn entity_add_if_exclusive_r_o_2() {
    let world = create_world();

    world.component::<First>().add::<flecs::Exclusive>();

    let e = world.entity();
    let o_1 = world.entity();
    let o_2 = world.entity();

    e.add_first::<First>(o_1);
    assert!(e.has_first::<First>(o_1));

    e.add_first_if::<First>(o_2, true);
    assert!(!e.has_first::<First>(o_1));
    assert!(e.has_first::<First>(o_2));

    e.add_first_if::<First>(o_1, false);
    assert!(!e.has_first::<First>(o_1));
    assert!(!e.has_first::<First>(o_2));
}

#[test]
fn entity_add_if_exclusive_r_o_3() {
    let world = create_world();

    world.component::<Rel>().add::<flecs::Exclusive>();

    let e = world.entity();

    e.add::<(Rel, Obj)>();
    assert!(e.has::<(Rel, Obj)>());

    e.add_if::<(Rel, Obj2)>(true);
    assert!(!e.has::<(Rel, Obj)>());
    assert!(e.has::<(Rel, Obj2)>());

    e.add_if::<(Rel, Obj)>(false);
    assert!(!e.has::<(Rel, Obj)>());
    assert!(!e.has::<(Rel, Obj2)>());
}

#[test]
fn entity_add_if_pair_w_0_object() {
    let world = create_world();

    let e = world.entity();
    let r = world.entity();
    let o_1 = world.entity();

    e.add_id((r, o_1));
    assert!(e.has_id((r, o_1)));

    e.add_id_if((r, 0), false);
    assert!(!e.has_id((r, o_1)));
    assert!(!e.has_id((r, *flecs::Wildcard)));
}

#[test]
fn entity_children_w_custom_relation() {
    let world = create_world();

    let rel = world.entity();

    let parent = world.entity();
    let child_1 = world.entity().add_id((rel, parent));
    let child_2 = world.entity().add_id((rel, parent));
    world.entity().child_of_id(parent);

    let mut child_1_found = false;
    let mut child_2_found = false;
    let mut count = 0;

    parent.each_child_of_id(rel, |child| {
        if child == child_1 {
            child_1_found = true;
        } else if child == child_2 {
            child_2_found = true;
        }
        count += 1;
    });

    assert_eq!(count, 2);
    assert!(child_1_found);
    assert!(child_2_found);
}

#[test]
fn entity_children_w_custom_relation_type() {
    let world = create_world();

    let parent = world.entity();
    let child_1 = world.entity().add_first::<Rel>(parent);
    let child_2 = world.entity().add_first::<Rel>(parent);
    world.entity().child_of_id(parent);

    let mut child_1_found = false;
    let mut child_2_found = false;
    let mut count = 0;

    parent.each_child_of::<Rel>(|child| {
        if child == child_1 {
            child_1_found = true;
        } else if child == child_2 {
            child_2_found = true;
        }
        count += 1;
    });

    assert_eq!(count, 2);
    assert!(child_1_found);
    assert!(child_2_found);
}

#[test]
fn entity_children_w_this() {
    let world = create_world();

    let mut count = 0;
    world.entity_from_id(*flecs::This_).each_child(|_| {
        count += 1;
    });
    assert_eq!(count, 0);
}

#[test]
fn entity_children_w_wildcard() {
    let world = create_world();

    let mut count = 0;
    world.entity_from_id(*flecs::Wildcard).each_child(|_| {
        count += 1;
    });
    assert_eq!(count, 0);
}

#[test]
fn entity_children_w_any() {
    let world = create_world();

    let mut count = 0;
    world.entity_from_id(*flecs::Any).each_child(|_| {
        count += 1;
    });
    assert_eq!(count, 0);
}

#[test]
fn entity_children_from_root() {
    let world = create_world();

    let mut count = 0;
    world.entity_from_id(0).each_child(|e| {
        assert!((e.name() == "flecs") || (e.name() == "entity_test"));
        count += 1;
    });
    assert_eq!(count, 2);
}

#[test]
fn entity_children_from_root_world() {
    let world = create_world();

    let mut count = 0;
    world.each_child(|e| {
        assert_eq!(e.name(), "flecs");
        count += 1;
    });
}

#[test]
fn entity_get_depth() {
    let world = create_world();

    let e1 = world.entity();
    let e2 = world.entity().child_of_id(e1);
    let e3 = world.entity().child_of_id(e2);
    let e4 = world.entity().child_of_id(e3);

    assert_eq!(e1.depth_id(*flecs::ChildOf), 0);
    assert_eq!(e2.depth_id(*flecs::ChildOf), 1);
    assert_eq!(e3.depth_id(*flecs::ChildOf), 2);
    assert_eq!(e4.depth_id(*flecs::ChildOf), 3);
}

#[test]
fn entity_get_depth_w_type() {
    let world = create_world();

    world.component::<Rel>().add::<flecs::Traversable>();

    let e1 = world.entity();
    let e2 = world.entity().add_first::<Rel>(e1);
    let e3 = world.entity().add_first::<Rel>(e2);
    let e4 = world.entity().add_first::<Rel>(e3);

    assert_eq!(e1.depth::<Rel>(), 0);
    assert_eq!(e2.depth::<Rel>(), 1);
    assert_eq!(e3.depth::<Rel>(), 2);
    assert_eq!(e4.depth::<Rel>(), 3);
}

#[test]
fn entity_set_alias() {
    let world = create_world();

    let e = world.entity_named(c"parent::child");
    e.set_alias(c"parent_child");

    assert_eq!(e, world.lookup(c"parent::child"));
    assert_eq!(e, world.lookup(c"parent_child"));
}

#[test]
fn entity_emplace_w_observer() {
    let world = create_world();

    world
        .observer::<&Position>()
        .add_event_id(*flecs::OnAdd)
        .each_entity(|e, _| {
            e.emplace(Velocity { x: 1, y: 2 });
        });

    let e = world.entity().emplace(Position { x: 10, y: 20 });

    assert!(e.has::<Position>());
    assert!(e.has::<Velocity>());
    assert_eq!(e.get::<Velocity>().x, 1);
    assert_eq!(e.get::<Velocity>().y, 2);
    assert_eq!(e.get::<Position>().x, 10);
    assert_eq!(e.get::<Position>().y, 20);
}

#[test]
#[ignore = "Scoped world was removed, debating if we should add it back"]
fn entity_scoped_world() {
    //TODO add back scoped world
}

#[test]
#[ignore = "Scoped world was removed, debating if we should add it back"]
fn entity_entity_lookup_not_recursive() {
    //TODO add back scoped world
}

#[test]
#[ignore = "Scoped world was removed, debating if we should add it back"]
fn entity_world_lookup_not_recursive() {
    //TODO add back scoped world
}
