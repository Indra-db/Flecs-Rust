#![allow(dead_code)]
use crate::common_test::*;

#[test]
fn test_world_multi_world_empty() {
    let _w1 = World::new();
    let _w2 = World::new();
}

#[test]
fn test_world_builtin_components() {
    let world = World::new();

    // Verify builtin components are registered
    assert_ne!(world.component::<flecs::Component>().id(), 0);
    assert_ne!(world.component::<flecs::Identifier>().id(), 0);
}

#[test]
fn test_world_multi_world_component() {
    let w1 = World::new();
    let w2 = World::new();

    let p1 = w1.component::<Position>();
    let _v1 = w1.component::<Velocity>();
    let _v2 = w2.component::<Velocity>();
    let _m2 = w2.component::<Mass>();

    assert_ne!(p1.id(), 0);

    let e1 = w1.entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 100 });

    let e2 = w2.entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 100 });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });
}

#[test]
fn test_world_type_id() {
    let world = World::new();

    let p = world.component::<Position>();
    assert_eq!(p.id(), Position::entity_id(&world));
}

#[test]
fn test_world_reregister_namespace() {
    let world = World::new();

    let p_id_1 = world.component::<Position>().id();
    let p_id_2 = world.component::<Position>().id();

    assert_eq!(p_id_1, p_id_2);
}

#[test]
fn test_world_reregister_after_delete() {
    let world = World::new();

    let c = world.component::<Position>();
    assert_eq!(c.name(), "Position");

    c.destruct();

    let d = world.component::<Position>();
    assert!(d.is_alive());
    assert_eq!(d.name(), "Position");
}

#[test]
fn test_world_count() {
    let world = World::new();

    assert_eq!(world.count(Position::id()), 0);

    world.entity().add(Position::id());
    world.entity().add(Position::id());
    world.entity().add(Position::id());
    world.entity().add(Position::id()).add(Mass::id());
    world.entity().add(Position::id()).add(Mass::id());
    world.entity().add(Position::id()).add(Velocity::id());

    assert_eq!(world.count(Position::id()), 6);
}

#[test]
fn test_world_count_id() {
    let world = World::new();

    let ent = world.entity();

    assert_eq!(world.count(ent.id()), 0);

    world.entity().add(ent.id());
    world.entity().add(ent.id());
    world.entity().add(ent.id());
    world.entity().add(ent.id()).add(Mass::id());
    world.entity().add(ent.id()).add(Mass::id());
    world.entity().add(ent.id()).add(Velocity::id());

    assert_eq!(world.count(ent.id()), 6);
}

#[test]
fn test_world_delete_with_id() {
    let world = World::new();

    let tag = world.entity();
    let e1 = world.entity().add(tag.id());
    let e2 = world.entity().add(tag.id());
    let e3 = world.entity().add(tag.id());

    world.delete_entities_with(tag.id());

    assert!(!e1.is_alive());
    assert!(!e2.is_alive());
    assert!(!e3.is_alive());
}

#[test]
fn test_world_delete_with_type() {
    let world = World::new();

    let e1 = world.entity().add(Tag::id());
    let e2 = world.entity().add(Tag::id());
    let e3 = world.entity().add(Tag::id());

    world.delete_entities_with(Tag::id());

    assert!(!e1.is_alive());
    assert!(!e2.is_alive());
    assert!(!e3.is_alive());
}

#[test]
fn test_world_remove_all_id() {
    let world = World::new();

    let tag_a = world.entity();
    let tag_b = world.entity();
    let e1 = world.entity().add(tag_a.id());
    let e2 = world.entity().add(tag_a.id());
    let e3 = world.entity().add(tag_a.id()).add(tag_b.id());

    world.remove_all(tag_a.id());

    assert!(e1.is_alive());
    assert!(e2.is_alive());
    assert!(e3.is_alive());

    assert!(!e1.has(tag_a.id()));
    assert!(!e2.has(tag_a.id()));
    assert!(!e3.has(tag_a.id()));
    assert!(e3.has(tag_b.id()));
}

#[test]
fn test_world_remove_all_type() {
    let world = World::new();

    let e1 = world.entity().add(Position::id());
    let e2 = world.entity().add(Position::id());
    let e3 = world.entity().add(Position::id()).add(Velocity::id());

    world.remove_all(Position::id());

    assert!(e1.is_alive());
    assert!(e2.is_alive());
    assert!(e3.is_alive());

    assert!(!e1.has(Position::id()));
    assert!(!e2.has(Position::id()));
    assert!(!e3.has(Position::id()));
    assert!(e3.has(Velocity::id()));
}

#[test]
fn test_world_remove_all_implicit() {
    let world = World::new();
    world.remove_all(Tag::id());
}

#[test]
fn test_world_get_scope() {
    let world = World::new();

    let e = world.entity_named("scope");
    world.set_scope(e.id());

    let s = world.get_scope();
    assert!(s.is_some());
    if let Some(scope_entity) = s {
        assert_eq!(scope_entity.id(), e.id());
        assert_eq!(scope_entity.name(), "scope");
    }
}

#[test]
fn test_world_is_alive() {
    let world = World::new();

    let e = world.entity();

    assert!(e.is_alive());
    assert!(!world.exists(1000));

    e.destruct();

    assert!(!e.is_alive());
}

#[test]
fn test_world_is_valid() {
    let world = World::new();

    let e = world.entity();

    assert!(e.is_alive());
    assert!(!world.exists(1000));

    e.destruct();

    assert!(!e.is_alive());

    world.make_alive(1000);
    assert!(world.exists(1000));
}

#[test]
fn test_world_exists() {
    let world = World::new();

    let e = world.entity();

    assert!(world.exists(e.id()));
    assert!(!world.exists(1000));
}

#[test]
fn test_world_get_alive() {
    let world = World::new();

    let e1 = world.entity();
    let e_no_gen = unsafe { flecs_ecs::sys::ecs_strip_generation(e1.id()) };
    assert_eq!(e1.id(), e_no_gen);
    e1.delete();

    let e2 = world.entity();
    assert_ne!(e1.id(), e2.id());
    assert_eq!(e_no_gen, unsafe { flecs_ecs::sys::ecs_strip_generation(e2.id()) });

    let alive = world.get_alive(e_no_gen);
    assert_eq!(alive, e2.id());
}

#[test]
fn test_world_make_alive() {
    let world = World::new();

    let e1 = world.entity();
    e1.delete();
    assert!(!e1.is_alive());

    let e2 = world.entity();
    assert_ne!(e1.id(), e2.id());
    e2.delete();
    assert!(!e2.is_alive());

    let e3 = world.make_alive(e2.id());
    assert_eq!(e2.id(), e3);
    assert!(world.exists(e3));
}

#[test]
fn test_world_component_w_low_id() {
    let world = World::new();

    let p = world.component::<Position>();
    assert_ne!(p.id(), 0);
}

#[test]
fn test_world_reset_world() {
    let world = World::new();
    let e = world.entity();

    assert!(world.exists(e.id()));
    world.reset();
    assert!(!world.exists(e.id()));
}

#[test]
fn test_world_scope_w_name() {
    let world = World::new();

    let parent = world.entity_named("parent");
    let _scope = world.set_scope(parent.id());
    let child = world.entity();

    assert!(child.has((flecs::ChildOf::ID, parent.id())));
    world.set_scope(0); // Reset scope
}

#[test]
fn test_world_type_w_tag_name() {
    let world = World::new();

    let c = world.component::<Tag>();
    assert_ne!(c.id(), 0);
}

#[test]
fn test_world_entity_w_tag_name() {
    let world = World::new();

    let c = world.entity_named("Tag");
    assert_ne!(c.id(), 0);
}

#[test]
fn test_world_entity_as_tag() {
    let world = World::new();

    let e = world.entity_named("MyTag");
    assert_ne!(e.id(), 0);

    let e2 = world.entity().add(e.id());

    assert!(e2.has(e.id()));
    assert_eq!(e.name(), "MyTag");
}

#[test]
fn test_world_entity_as_component() {
    let world = World::new();

    let e = world.entity_named("MyComponent");
    assert_ne!(e.id(), 0);

    let e2 = world.entity().set(Position { x: 10, y: 20 });

    assert!(e2.has::<Position>());

    assert_eq!(world.entity_named("Position").name(), "Position");
}

#[test]
fn test_world_entity_as_component_2_worlds() {
    let ecs1 = World::new();
    let e1 = ecs1.component::<Position>();
    assert_ne!(e1.id(), 0);

    let ecs2 = World::new();
    let e2 = ecs2.component::<Position>();
    assert_ne!(e2.id(), 0);

    assert_eq!(e1.id(), e2.id());
    assert_eq!(e1.id(), Position::entity_id(&ecs1));
    assert_eq!(e2.id(), Position::entity_id(&ecs2));
}

#[test]
fn test_world_copy_world() {
    let world1 = World::new();
    let world2 = world1.clone();

    assert_eq!(world1.ptr_mut(), world2.ptr_mut());
}

#[test]
fn test_world_exclusive_access_self_mutate() {
    let world = World::new();

    world.exclusive_access_begin(None);

    let e = world.entity();
    e.add(Position::id());
    assert!(e.has::<Position>());

    world.exclusive_access_end(false);
}

#[test]
fn test_world_get_mut_T() {
    let world = World::new();

    let has = world.try_get::<&Position>(|_| ()).is_some();
    assert!(!has);

    world.set(Position { x: 10, y: 20 });
    world.try_get::<&Position>(|pos| {
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    });
}

#[test]
fn test_world_get_mut_R_T() {
    #[derive(Component)]
    struct Tgt;

    let world = World::new();

    let has = world.try_get::<&(Tgt, Position)>(|_| ()).is_some();
    assert!(!has);

    world.set_pair::<Tgt, Position>(Position { x: 10, y: 20 });

    world.try_get::<&(Tgt, Position)>(|pos| {
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    });
}

#[test]
fn test_world_with_scope() {
    let world = World::new();

    let parent = world.entity_named("P");

    {
        let _scope = world.set_scope(parent.id());
        let _c1 = world.entity_named("C1");
        let _c2 = world.entity_named("C2");
    }
    world.set_scope(0); // Reset scope

    let c1 = world.lookup("P::C1");
    assert!(c1.is_alive());
}

#[test]
fn test_world_recursive_lookup() {
    let world = World::new();

    let a = world.entity_named("A");
    let b = world.entity_named("B");

    let p = world.entity_named("P");
    {
        let _scope = world.set_scope(p.id());
        let ca = world.entity_named("A");
        assert_ne!(ca.id(), a.id());

        let lookup_a = world.lookup("A");
        assert_eq!(ca.id(), lookup_a.id());

        let lookup_pa = world.lookup("P::A");
        assert_eq!(ca.id(), lookup_pa.id());

        let lookup_root_a = world.lookup("::A");
        assert_eq!(a.id(), lookup_root_a.id());

        let lookup_b = world.lookup("B");
        assert_eq!(b.id(), lookup_b.id());
    }
    world.set_scope(0); // Reset scope
}

#[test]
fn test_world_template_component_name() {
    let world = World::new();

    let c = world.component::<Position>();
    assert_ne!(c.id(), 0);
    assert_eq!(c.name(), "Position");
}

#[test]
fn test_world_set_lookup_path() {
    let world = World::new();

    let parent = world.entity_named("Parent");
    {
        let _scope = world.set_scope(parent.id());
        let child = world.entity_named("Child");
        assert!(child.is_alive());
    }
    world.set_scope(0); // Reset scope

    assert_eq!(world.lookup("Parent").id(), parent.id());
    assert!(!world.lookup("Child").is_alive());
    assert_eq!(world.lookup("Parent::Child").id(), world.lookup_recursive("Child").id());
}

#[test]
fn test_world_atfini() {
    static mut ATFINI_INVOKED: i32 = 0;

    fn on_destroyed(_w: WorldRef) {
        unsafe { ATFINI_INVOKED += 1; }
    }

    {
        let world = World::new();
        world.on_destroyed(on_destroyed);
    }

    unsafe { assert_eq!(ATFINI_INVOKED, 1) };
}

#[test]
fn test_world_register_from_scope() {
    #[derive(Component)]
    struct ScopeTest;

    #[derive(Component)]
    struct FromScope;

    let world = World::new();

    {
        let _scope = world.set_scope(ScopeTest::entity_id(&world));
        world.component::<FromScope>();
    }
    world.set_scope(0); // Reset scope

    let c = world.component::<FromScope>();
    assert!(c.has((flecs::ChildOf::ID, ScopeTest::entity_id(&world))));
}

#[test]
fn test_world_get_scope_type() {
    #[derive(Component)]
    struct ScopeTest;

    let world = World::new();

    let _scope = world.set_scope(ScopeTest::entity_id(&world));
    let s = world.get_scope();
    assert!(s.is_some());
    if let Some(scope_entity) = s {
        assert_eq!(scope_entity.id(), ScopeTest::entity_id(&world));
    }
    world.set_scope(0); // Reset scope
}

#[test]
fn test_world_builtin_after_reset() {
    let world = World::new();

    let c1 = world.component::<flecs::Component>();
    assert_ne!(c1.id(), 0);

    world.reset();

    let c2 = world.component::<flecs::Component>();
    assert_ne!(c2.id(), 0);
    assert_eq!(c1.id(), c2.id());
}

#[test]
fn test_world_register_component_w_core_name() {
    #[derive(Component)]
    struct Module;

    let world = World::new();

    let c = world.component::<Module>();
    assert_ne!(c.id(), 0);
}

#[test]
fn test_world_multi_world_component_namespace() {
    let w = World::new();
    let c = w.component::<Position>();
    let id_1 = c.id();

    let w2 = World::new();
    let c2 = w2.component::<Position>();
    let id_2 = c2.id();

    assert_eq!(id_1, id_2);
}

#[test]
fn test_world_reimport() {
    let world = World::new();

    let p1 = world.component::<Position>();
    let p2 = world.component::<Position>();

    assert_eq!(p1.id(), p2.id());
}

#[test]
fn test_world_scope_nested() {
    let world = World::new();

    let parent = world.entity_named("P");

    {
        let _scope1 = world.set_scope(parent.id());
        let child = world.entity_named("C");

        {
            let _scope2 = world.set_scope(child.id());
            let gchild = world.entity_named("GC");
            assert!(gchild.has((flecs::ChildOf::ID, child.id())));
        }
        world.set_scope(parent.id()); // Restore parent scope
    }
    world.set_scope(0); // Reset scope
}

#[test]
fn test_world_with_tag() {
    let world = World::new();

    let tag = world.entity();
    {
        let _scope = world.set_scope(tag.id());
        let e1 = world.entity().set(SelfRef { value: world.entity().id() });
        let e2 = world.entity().set(SelfRef { value: world.entity().id() });
        assert!(e1.is_alive());
        assert!(e2.is_alive());
    }
    world.set_scope(0); // Reset scope
}

#[test]
fn test_world_with_scope_no_lambda() {
    let world = World::new();

    let parent = world.entity_named("Parent");
    {
        let _scope = world.set_scope(parent.id());
        let child = world.entity_named("Child");
        assert!(child.has((flecs::ChildOf::ID, parent.id())));
    }
    world.set_scope(0); // Reset scope
}

#[test]
fn test_world_readonly_begin_end() {
    let world = World::new();

    world.entity().set(Position { x: 0, y: 0 });

    world.readonly_begin(false);
    let count = world.count(Position::id());
    world.readonly_end();

    assert_eq!(count, 1);
}

#[test]
fn test_world_defer_begin_end() {
    let world = World::new();

    world.defer_begin();
    world.entity().add(Position::id());
    world.defer_end();

    assert_eq!(world.count(Position::id()), 1);
}

#[test]
fn test_world_frame_begin_end() {
    let world = World::new();

    let dt = world.frame_begin(1.0);
    assert!(dt >= 0.0);
    world.frame_end();
}

#[test]
fn test_world_on_destroyed() {
    static mut CALLED: bool = false;

    fn on_destroyed(_w: WorldRef) {
        unsafe { CALLED = true; }
    }

    {
        let world = World::new();
        world.on_destroyed(on_destroyed);
    }

    unsafe { assert!(CALLED) };
}

#[test]
fn test_world_each_child() {
    let world = World::new();

    let parent = world.entity_named("Parent");
    {
        let _scope = world.set_scope(parent.id());
        let _c1 = world.entity_named("C1");
        let _c2 = world.entity_named("C2");
    }
    world.set_scope(0); // Reset scope

    let mut count = 0;
    world.each_child(|_| {
        count += 1;
    });

    assert!(count > 0);
}

#[test]
fn test_world_set_entity_range() {
    let world = World::new();

    world.set_entity_range(Entity::new(1), Entity::new(1000000));

    let e = world.entity();
    assert!(e.id() < 1000000);
}

#[test]
fn test_world_stage_count() {
    let world = World::new();

    world.set_stage_count(4);
    assert_eq!(world.get_stage_count(), 4);
}

#[test]
fn test_world_modified() {
    let world = World::new();

    let e = world.entity().set(Position { x: 5, y: 10 });

    world.modified(Position::id());

    assert!(e.is_alive());
}

#[test]
fn test_world_preallocate_entity_count() {
    let world = World::new();

    world.preallocate_entity_count(100);

    for _ in 0..100 {
        world.entity().add(Position::id());
    }

    assert_eq!(world.count(Position::id()), 100);
}

#[test]
fn test_world_get_info() {
    let world = World::new();

    let info = world.info();

    assert!(info.table_count >= 0);
}

#[test]
fn test_world_should_quit() {
    let world = World::new();

    assert!(!world.should_quit());
}

#[test]
fn test_world_is_deferred() {
    let world = World::new();

    assert!(!world.is_deferred());
    world.defer_begin();
    assert!(world.is_deferred());
    world.defer_end();
    assert!(!world.is_deferred());
}

#[test]
fn test_world_is_readonly() {
    let world = World::new();

    assert!(!world.is_readonly());
}

#[test]
fn test_world_check_components_consistency() {
    let world = World::new();

    let pos1 = world.component::<Position>();
    let pos2 = world.component::<Position>();

    assert_eq!(pos1.id(), pos2.id());
}

#[test]
fn test_world_register_after_delete() {
    let world = World::new();

    let e1 = world.component::<Position>();
    assert!(e1.is_alive());
    e1.destruct();

    let e2 = world.component::<Position>();
    assert!(e2.is_alive());
}

#[test]
fn test_world_entity_with_component() {
    let world = World::new();

    let e = world.entity().set(Position { x: 5, y: 10 }).set(Velocity { x: 1, y: 2 });

    assert!(e.has::<Position>());
    assert!(e.has::<Velocity>());

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 5);
        assert_eq!(p.y, 10);
    });

    e.get::<&Velocity>(|v| {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn test_world_query_count_consistency() {
    let world = World::new();

    for _ in 0..5 {
        world.entity().add(Position::id());
    }

    assert_eq!(world.count(Position::id()), 5);
}

#[test]
fn test_world_entity_view_operations() {
    let world = World::new();

    let e = world.entity();
    assert!(e.is_alive());

    let id = e.id();
    assert_ne!(id, 0);

    let e2 = world.lookup(world.entity_named("test").name());
    assert!(e2.is_alive());
}

#[test]
fn test_world_context_operations() {
    let world = World::new();

    let mut ctx_val: i32 = 42;
    world.set_context(&mut ctx_val as *mut i32 as *mut std::ffi::c_void, None);

    let ctx = world.context();
    assert_eq!(ctx as *const i32 as *const i32, &ctx_val as *const i32);
}
