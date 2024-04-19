use std::ffi::{c_void, CStr};

use flecs_ecs::core::*;
use flecs_ecs::sys;

mod common;
use common::*;

#[test]
fn entity_new() {
    let world = World::new();
    let entity = world.entity();
    assert!(entity.is_valid());
}

#[test]
fn entity_new_named() {
    let world = World::new();
    let entity = world.entity_named(c"test");
    assert!(entity.is_valid());
    assert_eq!(entity.name(), "test");
}

#[test]
fn entity_new_named_from_scope() {
    let world = World::new();
    let entity = world.entity_named(c"Foo");
    assert!(entity.is_valid());

    let prev = world.set_scope_with_id(entity);
    let child = world.entity_named(c"Bar");
    assert!(child.is_valid());

    world.set_scope_with_id(prev);

    assert_eq!(child.name(), "Bar");
    assert_eq!(child.path().unwrap(), "::Foo::Bar");
}

#[test]
fn entity_new_nested_named_from_nested_scope() {
    // Create a world
    let world = World::new();

    // Create an entity with nested name "Foo::Bar"
    let entity = world.entity_named(CStr::from_bytes_with_nul(b"Foo::Bar\0").unwrap());

    // Verify that the entity exists and its name and path are correct
    assert!(entity.is_valid());
    assert_eq!(entity.name(), "Bar");
    assert_eq!(entity.path().unwrap(), "::Foo::Bar");

    // Set the current scope to `entity`
    let prev = world.set_scope_with_id(entity);

    // Create a child entity with nested name "Hello::World" under the current scope
    let child = world.entity_named(CStr::from_bytes_with_nul(b"Hello::World\0").unwrap());

    // Verify that the child entity exists
    assert!(child.is_valid());

    // Restore the previous scope
    world.set_scope_with_id(prev);

    // Verify the name and hierarchical path of the child entity
    assert_eq!(child.name(), "World");
    assert_eq!(child.path().unwrap(), "::Foo::Bar::Hello::World");
}

#[test]
fn entity_new_add() {
    let world = World::new();

    let entity = world.entity().add::<Position>();

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());
}

#[test]
fn entity_new_add_2() {
    let world = World::new();

    let entity = world.entity().add::<Position>().add::<Velocity>();

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());
}

#[test]
fn entity_new_set() {
    // Create a world
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

    let entity = world.entity();

    assert!(entity.is_valid());

    entity.add::<Position>();

    assert!(entity.has::<Position>());
}

#[test]
fn entity_remove() {
    let world = World::new();

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add::<Position>();
    assert!(entity.has::<Position>());

    entity.remove::<Position>();
    assert!(!entity.has::<Position>());
}

#[test]
fn entity_set() {
    let world = World::new();

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
    let world = World::new();

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add::<Position>().add::<Velocity>();

    assert!(entity.has::<Position>());
    assert!(entity.has::<Velocity>());
}

#[test]
fn entity_add_entity() {
    let world = World::new();

    let tag = world.entity();
    assert!(tag.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id(tag);
    assert!(entity.has_id(tag));
}

#[test]
fn entity_add_childof() {
    let world = World::new();

    let parent = world.entity();
    assert!(parent.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id((flecs::ChildOf::ID, parent));
    assert!(entity.has_id((flecs::ChildOf::ID, parent)));
}

#[test]
fn entity_add_instanceof() {
    let world = World::new();

    let base = world.entity();
    assert!(base.is_valid());

    let entity = world.entity();
    assert!(entity.is_valid());

    entity.add_id((flecs::IsA::ID, base));
    assert!(entity.has_id((flecs::IsA::ID, base)));
}

#[test]
fn entity_remove_2() {
    let world = World::new();

    let entity = world.entity().add::<Position>().add::<Velocity>();

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
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();
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
    let world = World::new();

    let position = world.component::<Position>();

    let entity = world.entity().set(Position { x: 10, y: 20 });

    assert!(entity.is_valid());
    assert!(entity.has::<Position>());

    let mut invoked = false;
    world
        .observer::<&Position>()
        .add_event::<flecs::OnSet>()
        .each(|_| {
            invoked = true;
        });

    let pos = entity.get_untyped_mut(position.id());
    assert!(!pos.is_null());

    let pos = unsafe { &mut *(pos as *mut Position) };
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);

    entity.modified_id(position);
    assert!(invoked);
}

#[test]
fn entity_get_mut_generic_w_id() {
    let world = World::new();

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
    let world = World::new();
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
    let world = World::new();
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
    let world = World::new();
    let entity = world.entity();

    let entity = entity.add_flags(flecs::Pair::ID);

    assert_eq!(entity.id() & flecs::Pair::ID, flecs::Pair::ID);
}

#[test]
fn entity_remove_role() {
    let world = World::new();
    let entity = world.entity();
    let id = entity;

    let entity = entity.add_flags(flecs::Pair::ID);
    assert_eq!(entity.id() & flecs::Pair::ID, flecs::Pair::ID);

    let entity = entity.remove_flags();
    assert_eq!(entity, id);
}

#[test]
fn entity_has_role() {
    let world = World::new();
    let entity = world.entity();

    let entity = entity.add_flags(flecs::Pair::ID);
    assert!(entity.has_flags_for(flecs::Pair::ID));

    let entity = entity.remove_flags();
    assert!(!entity.has_flags_for(flecs::Pair::ID));
}

#[test]
fn entity_pair_role() {
    let world = World::new();
    let entity = world.entity();
    let entity2 = world.entity();

    let pair: IdView = IdView::new_from(&world, (entity, entity2));
    let pair = pair.add_flags(flecs::Pair::ID);

    assert!(pair.has_flags_for(flecs::Pair::ID));

    let rel = pair.first();
    let obj = pair.second();

    assert_eq!(rel, entity);
    assert_eq!(obj, entity2);
}

#[test]
fn entity_equals() {
    let world = World::new();
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
    let world = World::new();
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
    let world = World::new();

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
    let world = World::new();

    let e1 = world.entity();
    let e2 = world.entity();

    assert!(e2 > e1);
    assert!(e2 >= e1);
}

#[test]
fn entity_less_than() {
    let world = World::new();

    let e1 = world.entity();
    let e2 = world.entity();

    assert!(e1 < e2);
    assert!(e1 <= e2);
}

#[test]
fn entity_not_0_or_1() {
    let world = World::new();

    let e = world.entity();

    let id = e;

    assert_ne!(id, 0);
    assert_ne!(id, 1);
}

#[test]
fn entity_has_childof() {
    let world = World::new();

    let parent = world.entity();

    let child = world.entity().add_id((flecs::ChildOf::ID, parent));

    assert!(child.has_id((flecs::ChildOf::ID, parent)));
}

#[test]
fn entity_has_instanceof() {
    let world = World::new();

    let base = world.entity();

    let instance = world.entity().add_id((flecs::IsA::ID, base));

    assert!(instance.has_id((flecs::IsA::ID, base)));
}

#[test]
fn entity_has_instanceof_indirect() {
    let world = World::new();

    let base_of_base = world.entity();
    let base = world.entity().add_id((flecs::IsA::ID, base_of_base));

    let instance = world.entity().add_id((flecs::IsA::ID, base));

    assert!(instance.has_id((flecs::IsA::ID, base_of_base)));
}

#[test]
fn entity_null_string() {
    let world = World::new();

    let entity = world.entity();

    assert_eq!(entity.name(), "");
}

#[test]
fn entity_none_string() {
    let world = World::new();

    let entity = world.entity();

    assert_eq!(entity.get_name(), None);
}

#[test]
fn entity_set_name() {
    let world = World::new();

    let entity = world.entity();

    entity.set_name(c"Foo");

    assert_eq!(entity.name(), "Foo");
}

#[test]
fn entity_set_name_optional() {
    let world = World::new();

    let entity = world.entity();

    entity.set_name(c"Foo");

    assert_eq!(entity.get_name(), Some("Foo"));
}

#[test]
fn entity_change_name() {
    let world = World::new();

    let entity = world.entity_named(c"Bar");
    assert_eq!(entity.name(), "Bar");

    entity.set_name(c"Foo");
    assert_eq!(entity.name(), "Foo");

    entity.set_name(c"Bar");
    assert_eq!(entity.name(), "Bar");
}

#[test]
fn entity_delete() {
    let world = World::new();

    let entity = world.entity().add::<Position>().add::<Velocity>();

    entity.destruct();
    assert!(!entity.is_alive());

    let entity2 = world.entity();

    assert_eq!(*entity2.id() as u32, *entity.id() as u32);
    assert_ne!(entity2, entity);
}

#[test]
fn entity_clear() {
    let world = World::new();

    let entity = world.entity().add::<Position>().add::<Velocity>();

    entity.clear();
    assert!(!entity.has::<Position>());
    assert!(!entity.has::<Velocity>());

    let entity2 = world.entity();
    assert!(entity2 > entity);
}

#[test]
fn entity_force_owned() {
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

    let comp = world.component::<TagA>();
    let ptr = comp.try_get::<sys::EcsComponent>().unwrap();

    assert_eq!(ptr.size, 0);
    assert_eq!(ptr.alignment, 0);
}

#[test]
fn entity_get_null_name() {
    let world = World::new();

    let entity = world.entity();
    let name = entity.get_name();
    assert_eq!(name, None);
}

#[test]
fn entity_get_target() {
    let world = World::new();

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
    let world = World::new();

    let parent = world.entity();
    let child = world.entity().child_of_id(parent);

    assert_eq!(child.target_id(flecs::ChildOf::ID, 0), parent);
    assert_eq!(child.parent(), parent);
}

/// # See also
/// * C++ tests: `Entity_is_enabled_component_enabled` + `Entity_is_disabled_component_enabled` combined
#[test]
fn entity_is_component_enabled() {
    let world = World::new();

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
//     let world = World::new();

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
//     let world = World::new();

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
fn entity_is_pair_first_enabled() {
    let world = World::new();

    let tgt_a = world.entity();
    let tgt_b = world.entity();

    let e = world.entity().add_pair_first::<Position>(tgt_a);

    assert!(e.is_enabled_pair_first::<Position>(tgt_a));
    assert!(!e.is_enabled_pair_first::<Position>(tgt_b));
}

#[test]
fn entity_get_type() {
    let world = World::new();

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
        assert_eq!(type_2[0], world.id::<Position>());
    }

    entity.add::<Velocity>();
    let type_3 = entity.archetype();
    assert_eq!(type_3.count(), 2);
    assert_eq!(type_3[1], world.id::<Velocity>());
}

#[test]
fn entity_get_nonempty_type() {
    let world = World::new();

    let entity = world.entity().add::<Position>();
    assert!(entity.is_valid());

    let type_1 = entity.archetype();
    assert_eq!(type_1.count(), 1);
    assert_eq!(type_1.get(0).unwrap(), world.id::<Position>());

    let type_2 = entity.archetype();
    assert_eq!(type_2.count(), 1);
    assert_eq!(type_2.get(0).unwrap(), world.id::<Position>());
}

#[test]
fn entity_set_no_copy() {
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

    let base = world.entity().override_type::<Position>();

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<Position>());
    assert!(entity.owns::<Position>());
}

#[test]
fn entity_override_id() {
    let world = World::new();

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
    let world = World::new();

    let tgt_a = world.entity();
    let tgt_b = world.entity();

    let base = world
        .entity()
        .override_pair_first::<Position>(tgt_a)
        .add_pair_first::<Position>(tgt_b);

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has_pair_first::<Position>(tgt_a));
    assert!(entity.owns_pair_first::<Position>(tgt_a));

    assert!(entity.has_pair_first::<Position>(tgt_b));
    assert!(!entity.owns_pair_first::<Position>(tgt_b));
}

#[test]
fn entity_override_pair_w_ids() {
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

    let base = world
        .entity()
        .set_override_pair_first::<Position, TagA>(Position { x: 10, y: 20 });

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<(Position, TagA)>());
    assert!(entity.owns::<(Position, TagA)>());

    let p = entity.try_get_pair_first::<Position, TagA>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let p_base = base.try_get_pair_first::<Position, TagA>();
    assert!(p_base.is_some());
    let p_base = p_base.unwrap();
    assert_eq!(p_base.x, 10);
    assert_eq!(p_base.y, 20);
}

#[test]
fn entity_set_override_pair_w_tgt_id() {
    let world = World::new();

    let tgt = world.entity();

    let base = world
        .entity()
        .set_override_pair_first_id::<Position>(Position { x: 10, y: 20 }, tgt);

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has_pair_first::<Position>(tgt));
    assert!(entity.owns_pair_first::<Position>(tgt));

    let p = entity.try_get_pair_first_id::<Position>(tgt);
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let p_base = base.try_get_pair_first_id::<Position>(tgt);
    assert!(p_base.is_some());
    let p_base = p_base.unwrap();
    assert_eq!(p_base.x, 10);
    assert_eq!(p_base.y, 20);
}

#[test]
fn entity_set_override_pair_w_rel_tag() {
    let world = World::new();

    let base = world
        .entity()
        .set_override_pair_second::<TagA, Position>(Position { x: 10, y: 20 });

    let entity = world.entity().add_id((flecs::IsA::ID, base));

    assert!(entity.has::<(TagA, Position)>());
    assert!(entity.owns::<(TagA, Position)>());

    let p = entity.try_get_pair_second::<TagA, Position>();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let p_base = base.try_get_pair_second::<TagA, Position>();
    assert!(p_base.is_some());
    let p_base = p_base.unwrap();
    assert_eq!(p_base.x, 10);
    assert_eq!(p_base.y, 20);
}

#[test]
fn entity_name() {
    let world = World::new();

    let entity = world.entity_named(c"Foo");

    assert_eq!(entity.name(), "Foo");
    assert_eq!(entity.get_name(), Some("Foo"));
    assert_eq!(entity.name_cstr(), c"Foo");
    assert_eq!(entity.get_name_cstr(), Some(c"Foo"));
}

#[test]
fn entity_name_empty() {
    let world = World::new();

    let entity = world.entity();

    assert_eq!(entity.name(), "");
    assert_eq!(entity.get_name(), None);
    assert_eq!(entity.name_cstr(), c"");
    assert_eq!(entity.get_name_cstr(), None);
}

#[test]
fn entity_path() {
    let world = World::new();

    let parent = world.entity_named(c"parent");
    world.set_scope_with_id(parent.id());
    let child = world.entity_named(c"child");

    assert_eq!(&child.path().unwrap(), "::parent::child");
}

#[test]
fn entity_path_from() {
    let world = World::new();

    let parent = world.entity_named(c"parent");
    world.set_scope_with_id(parent.id());
    let child = world.entity_named(c"child");
    world.set_scope_with_id(child.id());
    let grandchild = world.entity_named(c"grandchild");

    assert_eq!(&grandchild.path().unwrap(), "::parent::child::grandchild");
    assert_eq!(
        &grandchild.path_from_id(parent).unwrap(),
        "child::grandchild"
    );
}

#[test]
fn entity_path_from_type() {
    let world = World::new();

    let parent = world.entity_named(c"parent");
    world.set_scope_with_id(parent.id());
    let child = world.entity_named(c"child");
    world.set_scope_with_id(child.id());
    let grandchild = world.entity_named(c"grandchild");

    assert_eq!(&grandchild.path().unwrap(), "::parent::child::grandchild");
    assert_eq!(
        &grandchild.path_from_id(parent).unwrap(),
        "child::grandchild"
    );
}

#[test]
fn entity_path_custom_sep() {
    let world = World::new();

    let parent = world.entity_named(c"parent");
    world.set_scope_with_id(parent.id());
    let child = world.entity_named(c"child");

    assert_eq!(&child.path_w_sep(c"_", c"?").unwrap(), "?parent_child");
}

#[test]
fn entity_path_from_custom_sep() {
    let world = World::new();

    let parent = world.entity_named(c"parent");
    world.set_scope_with_id(parent.id());
    let child = world.entity_named(c"child");
    world.set_scope_with_id(child.id());
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
    let world = World::new();

    let parent = world.entity_from::<Parent>();
    world.set_scope_with_id(parent.id());
    let child = world.entity_named(c"child");
    world.set_scope_with_id(child.id());
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
    let world = World::new();

    let entity = world.entity_named(c"Foo::Bar");
    assert!(entity.is_valid());
    assert_eq!(entity.name(), "Bar");
    assert_eq!(entity.path().unwrap(), "::Foo::Bar");
}

#[test]
fn entity_implicit_type_str_to_char() {
    let world = World::new();

    let entity = world.entity_named(c"Foo");
    assert!(entity.is_valid());

    assert_eq!(entity.archetype().to_string().unwrap(), "(Identifier,Name)");
}

#[test]
fn entityview_to_entity_to_entity_view() {
    let world = World::new();

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
    let world = World::new();
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
    let world = World::new();

    let entity_view: EntityView = world.entity();
    let stage = world.stage(0);

    world.readonly_begin(false);

    let entity_mut = entity_view.mut_current_stage(&stage);
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
    let world = World::new();
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
    let world = World::new();
    let entity = world.entity().set(Template::<Position> {
        value: Position { x: 10, y: 20 },
    });

    let pos = entity.try_get::<Template<Position>>().unwrap();
    assert_eq!(pos.value.x, 10);
    assert_eq!(pos.value.y, 20);
}

#[test]
fn entity_get_1_component_w_callback() {
    let world = World::new();
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

#[ignore]
#[test]
fn entity_get_2_components_w_callback() {
    // let world = World::new();
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
    let world = World::new();
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

#[ignore]
#[test]
fn entity_get_mut_2_components_w_callback() {
    // multiple components not supported in get_callback (for now)
}

#[test]
fn entity_get_component_w_callback_nested() {
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

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
    let world = World::new();

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

    let pos = e.get::<Position>();
    let vel = e.get::<Velocity>();
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);
    assert_eq!(vel.x, 1);
    assert_eq!(vel.y, 2);
}

#[test]
fn entity_defer_set_3_components() {
    let world = World::new();

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

    let pos = e.get::<Position>();
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);

    let vel = e.get::<Velocity>();
    assert_eq!(vel.x, 1);
    assert_eq!(vel.y, 2);

    let mass = e.get::<Mass>();
    assert_eq!(mass.value, 50);
}

#[test]
fn entity_defer_set_2_component_w_on_set() {
    let world = World::new();

    let mut position_set = 0;
    let mut velocity_set = 0;

    world
        .observer::<&Position>()
        .add_event_id(*flecs::OnSet)
        .each_entity(|_e, p| {
            position_set += 1;
            assert_eq!(p.x, 10);
            assert_eq!(p.y, 20);
        });

    world
        .observer::<&Velocity>()
        .add_event_id(*flecs::OnSet)
        .each_entity(|_e, v| {
            velocity_set += 1;
            assert_eq!(v.x, 1);
            assert_eq!(v.y, 2);
        });

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    assert_eq!(position_set, 1);
    assert_eq!(velocity_set, 1);

    let pos = e.get::<Position>();
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);

    let vel = e.get::<Velocity>();
    assert_eq!(vel.x, 1);
    assert_eq!(vel.y, 2);
}

/*

void Entity_set_2_w_on_set(void) {
    flecs::world ecs;

    int32_t position_set = 0;
    int32_t velocity_set = 0;

    ecs.observer<Position>()
        .event(flecs::OnSet)
        .each([&](flecs::entity e, Position& p) {
            position_set ++;
            test_int(p.x, 10);
            test_int(p.y, 20);
        });

    ecs.observer<Velocity>()
        .event(flecs::OnSet)
        .each([&](flecs::entity e, Velocity& v) {
            velocity_set ++;
            test_int(v.x, 1);
            test_int(v.y, 2);
        });

    auto e = ecs.entity()
        .set([](Position& p, Velocity& v){
            p = {10, 20};
            v = {1, 2};
        });

    test_int(position_set, 1);
    test_int(velocity_set, 1);

    test_bool(e.get([](const Position& p, const Velocity& v) {
        test_int(p.x, 10);
        test_int(p.y, 20);

        test_int(v.x, 1);
        test_int(v.y, 2);
    }), true);
}

*/
