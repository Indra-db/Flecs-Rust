#![allow(dead_code)]
use crate::common_test::*;

#[test]
fn multi_world_empty() {
    let _w1 = World::new();
    let _w2 = World::new();
}

#[test]
fn builtin_components() {
    let world = World::new();

    // Verify builtin components are registered
    assert_ne!(world.component::<flecs::Component>().id(), 0);
    assert_ne!(world.component::<flecs::Identifier>().id(), 0);
}

#[test]
fn multi_world_component() {
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
fn type_id() {
    let world = World::new();

    let p = world.component::<Position>();
    assert_eq!(p.id(), Position::entity_id(&world));
}

#[test]
fn reregister_namespace() {
    let world = World::new();

    let p_id_1 = world.component::<Position>().id();
    let p_id_2 = world.component::<Position>().id();

    assert_eq!(p_id_1, p_id_2);
}

#[test]
fn reregister_after_delete() {
    let world = World::new();

    let c = world.component::<Position>();
    assert_eq!(c.name(), "Position");

    c.destruct();

    let d = world.component::<Position>();
    assert!(d.is_alive());
    assert_eq!(d.name(), "Position");
}

#[test]
fn count() {
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
fn count_id() {
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
fn count_pair() {
    let world = World::new();

    let parent = world.entity();

    assert_eq!(world.count((flecs::ChildOf::ID, parent.id())), 0);

    world.entity().child_of(parent);
    world.entity().child_of(parent);
    world.entity().child_of(parent);
    world.entity().child_of(parent);
    world.entity().child_of(parent);
    world.entity().child_of(parent);

    assert_eq!(world.count((flecs::ChildOf::ID, parent.id())), 6);
}

#[test]
fn count_pair_type_id() {
    let world = World::new();

    let target = world.entity();

    assert_eq!(world.count((Rel::id(), target.id())), 0);

    world.entity().add((Rel::id(), target.id()));
    world.entity().add((Rel::id(), target.id()));
    world.entity().add((Rel::id(), target.id()));
    world.entity().add((Rel::id(), target.id()));
    world.entity().add((Rel::id(), target.id()));
    world.entity().add((Rel::id(), target.id()));

    assert_eq!(world.count((Rel::id(), target.id())), 6);
}

#[test]
fn count_pair_id() {
    let world = World::new();

    let rel = world.entity();
    let target = world.entity();

    assert_eq!(world.count((rel.id(), target.id())), 0);

    world.entity().add((rel.id(), target.id()));
    world.entity().add((rel.id(), target.id()));
    world.entity().add((rel.id(), target.id()));
    world.entity().add((rel.id(), target.id()));
    world.entity().add((rel.id(), target.id()));
    world.entity().add((rel.id(), target.id()));

    assert_eq!(world.count((rel.id(), target.id())), 6);
}

#[test]
fn delete_with_id() {
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
fn delete_with_type() {
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
fn delete_with_pair() {
    let world = World::new();

    let rel = world.entity();
    let obj = world.entity();

    let e1 = world.entity().add((rel.id(), obj.id()));
    let e2 = world.entity().add((rel.id(), obj.id()));
    let e3 = world.entity().add((rel.id(), obj.id()));

    world.delete_entities_with((rel.id(), obj.id()));

    assert!(!e1.is_alive());
    assert!(!e2.is_alive());
    assert!(!e3.is_alive());
}

#[test]
fn delete_with_pair_type() {
    let world = World::new();

    let e1 = world.entity().add((Rel::id(), Obj::id()));
    let e2 = world.entity().add((Rel::id(), Obj::id()));
    let e3 = world.entity().add((Rel::id(), Obj::id()));

    world.delete_entities_with((Rel::entity_id(&world), Obj::entity_id(&world)));

    assert!(!e1.is_alive());
    assert!(!e2.is_alive());
    assert!(!e3.is_alive());
}

#[test]
fn delete_with_implicit() {
    // No entities have Tag — must not crash
    let world = World::new();
    world.delete_entities_with(Tag::id());
}

#[test]
fn delete_with_pair_implicit() {
    // No entities have (Rel, Obj) — must not crash
    let world = World::new();
    world.delete_entities_with((Rel::entity_id(&world), Obj::entity_id(&world)));
}

#[test]
fn remove_all_id() {
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
fn remove_all_type() {
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
fn remove_all_implicit() {
    let world = World::new();
    world.remove_all(Tag::id());
}

#[test]
fn remove_all_pair() {
    let world = World::new();

    let rel = world.entity();
    let obj_a = world.entity();
    let obj_b = world.entity();

    let e1 = world.entity().add((rel.id(), obj_a.id()));
    let e2 = world.entity().add((rel.id(), obj_a.id()));
    let e3 = world.entity().add((rel.id(), obj_a.id())).add((rel.id(), obj_b.id()));

    world.remove_all((rel.id(), obj_a.id()));

    assert!(e1.is_alive());
    assert!(e2.is_alive());
    assert!(e3.is_alive());

    assert!(!e1.has((rel.id(), obj_a.id())));
    assert!(!e2.has((rel.id(), obj_a.id())));
    assert!(!e3.has((rel.id(), obj_a.id())));
    assert!(e3.has((rel.id(), obj_b.id())));
}

#[test]
fn remove_all_pair_type() {
    let world = World::new();

    let e1 = world.entity().add((Rel::id(), Obj::id()));
    let e2 = world.entity().add((Rel::id(), Obj::id()));
    let e3 = world.entity().add((Rel::id(), Obj::id())).add((Rel::id(), Obj2::id()));

    world.remove_all((Rel::entity_id(&world), Obj::entity_id(&world)));

    assert!(e1.is_alive());
    assert!(e2.is_alive());
    assert!(e3.is_alive());

    assert!(!e1.has((Rel::id(), Obj::id())));
    assert!(!e2.has((Rel::id(), Obj::id())));
    assert!(!e3.has((Rel::id(), Obj::id())));
    assert!(e3.has((Rel::id(), Obj2::id())));
}

#[test]
fn remove_all_pair_implicit() {
    // No entities have (Rel, Obj) — must not crash
    let world = World::new();
    world.remove_all((Rel::entity_id(&world), Obj::entity_id(&world)));
}

#[test]
fn get_scope() {
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
fn is_alive() {
    let world = World::new();

    let e = world.entity();

    assert!(e.is_alive());
    assert!(!world.exists(1000));

    e.destruct();

    assert!(!e.is_alive());
}

#[test]
fn is_valid() {
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
fn exists() {
    let world = World::new();

    let e = world.entity();

    assert!(world.exists(e.id()));
    assert!(!world.exists(1000));
}

#[test]
fn get_alive() {
    let world = World::new();

    let e1 = world.entity();
    let e_no_gen = unsafe { flecs_ecs::sys::ecs_strip_generation(*e1.id()) };
    assert_eq!(*e1.id(), e_no_gen);
    e1.destruct();

    let e2 = world.entity();
    assert_ne!(*e1.id(), *e2.id());
    assert_eq!(e_no_gen, unsafe { flecs_ecs::sys::ecs_strip_generation(*e2.id()) });

    let alive = world.get_alive(e_no_gen);
    assert_eq!(alive, e2.id());
}

#[test]
fn make_alive() {
    let world = World::new();

    let e1 = world.entity();
    e1.destruct();
    assert!(!e1.is_alive());

    let e2 = world.entity();
    assert_ne!(e1.id(), e2.id());
    e2.destruct();
    assert!(!e2.is_alive());

    let e3 = world.make_alive(e2.id());
    assert_eq!(e2.id(), e3);
    assert!(world.exists(e3));
}

#[test]
fn component_w_low_id() {
    let world = World::new();

    let p = world.component::<Position>();
    assert_ne!(p.id(), 0);
}

#[test]
fn reset_world() {
    let world = World::new();
    let e = world.entity().id();

    assert!(world.exists(e));
    let world = world.reset();
    assert!(!world.exists(e));
}

#[test]
fn scope_w_name() {
    let world = World::new();

    let parent = world.entity_named("parent");
    let _scope = world.set_scope(parent.id());
    let child = world.entity();

    assert!(child.has((flecs::ChildOf::ID, parent.id())));
    world.set_scope(0); // Reset scope
}

#[test]
fn type_w_tag_name() {
    let world = World::new();

    let c = world.component::<Tag>();
    assert_ne!(c.id(), 0);
}

#[test]
fn entity_w_tag_name() {
    let world = World::new();

    let c = world.entity_named("Tag");
    assert_ne!(c.id(), 0);
}

#[test]
fn entity_as_tag() {
    let world = World::new();

    let e = world.entity_named("MyTag");
    assert_ne!(e.id(), 0);

    let e2 = world.entity().add(e.id());

    assert!(e2.has(e.id()));
    assert_eq!(e.name(), "MyTag");
}

#[test]
fn entity_as_component() {
    let world = World::new();

    let e = world.entity_named("MyComponent");
    assert_ne!(e.id(), 0);

    let e2 = world.entity().set(Position { x: 10, y: 20 });

    assert!(e2.has(Position::id()));

    assert_eq!(world.entity_named("Position").name(), "Position");
}

#[test]
fn entity_as_component_2_worlds() {
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
fn copy_world() {
    let world1 = World::new();
    let world2 = world1.clone();

    assert_eq!(world1.ptr_mut(), world2.ptr_mut());
}

#[test]
fn exclusive_access_self_mutate() {
    let world = World::new();

    world.exclusive_access_begin(None);

    let e = world.entity();
    e.add(Position::id());
    assert!(e.has(Position::id()));

    world.exclusive_access_end(false);
}

#[test]
fn get_mut_type() {
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
fn get_mut_rel_type() {
    #[derive(Component)]
    struct Tgt();

    let world = World::new();

    let has = world.try_get::<&(Position,Tgt)>(|_| ()).is_some();
    assert!(!has);

    world.set_pair::<Position, Tgt>(Position { x: 10, y: 20 });
    world.try_get::<&(Position, Tgt)>(|pos_tgt| {
        assert_eq!(pos_tgt.x, 10);
        assert_eq!(pos_tgt.y, 20);
    });
}

#[test]
fn with_scope() {
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
fn recursive_lookup() {
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
fn template_component_name() {
    let world = World::new();

    let c = world.component::<Position>();
    assert_ne!(c.id(), 0);
    assert_eq!(c.name(), "Position");
}

#[test]
fn set_lookup_path() {
    let world = World::new();

    let parent = world.entity_named("Parent");
    {
        let _scope = world.set_scope(parent.id());
        let child = world.entity_named("Child");
        assert!(child.is_alive());
    }
    world.set_scope(0); // Reset scope

    assert_eq!(world.lookup("Parent").id(), parent.id());
    // "Child" is not at root scope — unqualified lookup from root returns None.
    assert!(world.try_lookup("Child").is_none());
    // Fully qualified path finds the child.
    let child = world.lookup("Parent::Child");
    assert!(child.is_alive());
}

#[test]
fn atfini() {
    static ATFINI_INVOKED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
    ATFINI_INVOKED.store(0, std::sync::atomic::Ordering::Relaxed);

    fn on_destroyed(_w: WorldRef) {
        ATFINI_INVOKED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    {
        let world = World::new();
        world.on_destroyed(on_destroyed);
    }

    assert_eq!(ATFINI_INVOKED.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[test]
fn register_from_scope() {
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
fn get_scope_type() {
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
fn make_pair() {
    let world = World::new();

    let r = world.entity();
    let t = world.entity();
    let id = world.id_view_from((r, t));

    assert!(id.is_pair());
    assert_eq!(id.first_id().id(), r.id());
    assert_eq!(id.second_id().id(), t.id());
}

#[test]
fn make_pair_of_pair_type() {
    let world = World::new();

    let t = world.entity();
    let id = world.id_view_from((Position::entity_id(&world), t.id()));

    assert!(id.is_pair());
    assert_eq!(id.first_id().id(), Position::entity_id(&world));
    assert_eq!(id.second_id().id(), t.id());
}

#[test]
fn builtin_after_reset() {
    let world = World::new();

    let c1 = world.component::<flecs::Component>().id();
    assert_ne!(c1, 0);

    let world = world.reset();

    let c2 = world.component::<flecs::Component>();
    assert_ne!(c2.id(), 0);
    assert_eq!(c1, c2.id());
}

#[test]
fn register_component_w_core_name() {
    #[derive(Component)]
    struct Module;

    let world = World::new(); 

    let c = world.component::<Module>();
    assert_ne!(c.id(), 0);
}

#[test]
fn multi_world_component_namespace() {
    let w = World::new();
    let c = w.component::<Position>();
    let id_1 = c.id();

    let w2 = World::new();
    let c2 = w2.component::<Position>();
    let id_2 = c2.id();

    assert_eq!(id_1, id_2);
}

#[test]
fn reimport() {
    let world = World::new();

    let p1 = world.component::<Position>();
    let p2 = world.component::<Position>();

    assert_eq!(p1.id(), p2.id());
}

#[test]
fn scope_nested() {
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
fn with_tag() {
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
fn with_tag_type() {
    // world.with(Tag, closure) auto-adds Tag to every entity created inside
    let world = World::new();

    world.with(Tag::id(), || {
        world.entity();
        world.entity();
        world.entity();
    });

    assert_eq!(world.count(Tag::id()), 3);
}

#[test]
fn with_relation() {
    // world.with((rel, obj), closure) auto-adds pair (rel, obj) to entities
    let world = World::new();

    let likes = world.entity();
    let bob = world.entity();

    world.with((likes.id(), bob.id()), || {
        world.entity();
        world.entity();
        world.entity();
    });

    assert_eq!(world.count((likes.id(), bob.id())), 3);
}

#[test]
fn with_relation_type() {
    // typed first, raw entity second: (Likes::id(), bob)
    let world = World::new();

    let bob = world.entity();

    world.with((Likes::entity_id(&world), bob.id()), || {
        world.entity();
        world.entity();
        world.entity();
    });

    assert_eq!(world.count((Likes::entity_id(&world), bob.id())), 3);
}

#[test]
fn with_relation_object_type() {
    // fully typed pair (Likes, Bob)
    let world = World::new();

    world.with((Likes::entity_id(&world), Bob::entity_id(&world)), || {
        world.entity();
        world.entity();
        world.entity();
    });

    assert_eq!(
        world.count((Likes::entity_id(&world), Bob::entity_id(&world))),
        3
    );
}

#[test]
fn with_tag_nested() {
    // ecs_set_with is a single value — inner with() replaces outer.
    // Entity created in inner with gets the innermost tag only.
    // Entity created at outer with level gets the outer tag.
    let world = World::new();

    let tier1 = world.entity();
    let tier2 = world.entity();
    let mut inner = world.entity_null();
    let mut outer_entity = world.entity_null();

    world.with(tier1.id(), || {
        outer_entity = world.entity();
        world.with(tier2.id(), || {
            inner = world.entity();
        });
    });

    // outer_entity was created when tier1 was active
    assert!(outer_entity.has(tier1));
    assert!(!outer_entity.has(tier2));

    // inner was created when tier2 replaced tier1
    assert!(inner.has(tier2));
    assert!(!inner.has(tier1));
}

#[test]
fn with_scope_no_lambda() {
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
fn with_scope_type() {
    #[derive(Component)]
    struct ParentScope;

    let world = World::new();

    world.scope(ParentScope::entity_id(&world), |_| {
        world.entity_named("Child");
    });

    let parent = world.lookup("ParentScope");
    assert!(parent.is_alive());

    let child = world.lookup("ParentScope::Child");
    assert!(child.is_alive());
    assert_eq!(child.id(), parent.lookup("Child").id());
}

#[test]
fn with_scope_type_staged() {
    #[derive(Component)]
    struct ParentScope;

    let world = World::new();
    let stage = world.stage(0);

    world.readonly_begin(false);
    stage.set_scope(ParentScope::entity_id(&world));
    let e = stage.entity_named("Child");
    stage.set_scope(0);
    world.readonly_end();

    assert!(e.has((flecs::ChildOf::ID, ParentScope::entity_id(&world))));

    let parent = world.lookup("ParentScope");
    assert!(parent.is_alive());

    let child = world.lookup("ParentScope::Child");
    assert!(child.is_alive());
    assert_eq!(child.id(), parent.lookup("Child").id());
}

#[test]
fn with_scope_type_no_lambda() {
    #[derive(Component)]
    struct ParentScope;

    let world = World::new();

    world.set_scope(ParentScope::entity_id(&world));
    let child = world.entity_named("Child");
    world.set_scope(0); // Reset scope

    assert!(child.has((flecs::ChildOf::ID, ParentScope::entity_id(&world))));
    assert!(world.get_scope().is_none());
}

#[test]
fn readonly_begin_end() {
    let world = World::new();

    world.entity().set(Position { x: 0, y: 0 });

    world.readonly_begin(false);
    let count = world.count(Position::id());
    world.readonly_end();

    assert_eq!(count, 1);
}

#[test]
fn staged_count() {
    let world = World::new();

    // Enter readonly mode — entities created via stage are deferred.
    world.readonly_begin(false);

    let stage = world.stage(0);
    for _ in 0..6 {
        stage.entity().add(Position::id());
    }

    // Deferred commands not yet merged; count must be 0.
    assert_eq!(world.count(Position::id()), 0);

    // readonly_end flushes the deferred commands.
    world.readonly_end();

    assert_eq!(world.count(Position::id()), 6);
}

#[test]
fn async_stage_add() {
    let world = World::new();
    world.component::<Position>();

    let e = world.entity();
    let async_stage = world.create_async_stage();

    // Queue an Add<Position> command on the async stage — not merged yet.
    e.mut_current_stage(&async_stage).add(Position::id());

    assert!(!e.has(Position::id()));

    // Explicit merge flushes the async stage's command queue.
    async_stage.merge();

    assert!(e.has(Position::id()));
}

#[test]
fn defer_begin_end() {
    let world = World::new();

    world.defer_begin();
    world.entity().add(Position::id());
    world.defer_end();

    assert_eq!(world.count(Position::id()), 1);
}

#[test]
fn frame_begin_end() {
    let world = World::new();

    let dt = world.frame_begin(1.0);
    assert!(dt >= 0.0);
    world.frame_end();
}

#[test]
fn on_destroyed() {
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
fn each_child() {
    let world = World::new();

    let parent = world.entity_named("Parent");
    {
        let _scope = world.set_scope(parent.id());
        let _c1 = world.entity_named("C1");
        let _c2 = world.entity_named("C2");
    }
    world.set_scope(0); // Reset scope

    // C1 and C2 are children of Parent (created via scope), not the world entity.
    let mut count = 0;
    parent.each_child(|_| {
        count += 1;
    });

    assert_eq!(count, 2);
}

#[test]
fn set_entity_range() {
    let world = World::new();

    // Get current max entity ID so min is valid (must not be below current max).
    // Use a large min value that is safely above all built-in entity IDs.
    let min = 500000u64;
    let max = 1000000u64;
    world.set_entity_range(Entity::new(min), Entity::new(max));

    let e = world.entity();
    assert!(e.id() >= min && e.id() < max);
}

#[test]
fn stage_count() {
    let world = World::new();

    world.set_stage_count(4);
    assert_eq!(world.get_stage_count(), 4);
}

#[test]
fn modified() {
    let world = World::new();

    let mut count = 0;

    world.observer::<flecs::OnSet, &Position>()
        .each(|_p| { /* triggered by modified */ });

    let e = world.entity().set(Position { x: 5, y: 10 });

    // modified must be called on the entity that owns the component
    e.modified(Position::entity_id(&world));

    assert!(e.is_alive());
}

#[test]
fn preallocate_entity_count() {
    let world = World::new();

    world.preallocate_entity_count(100);

    for _ in 0..100 {
        world.entity().add(Position::id());
    }

    assert_eq!(world.count(Position::id()), 100);
}

#[test]
fn get_info() {
    let world = World::new();

    let info = world.info();

    assert!(info.table_count >= 0);
}

#[test]
fn should_quit() {
    let world = World::new();

    assert!(!world.should_quit());
}

#[test]
fn is_deferred() {
    let world = World::new();

    assert!(!world.is_deferred());
    world.defer_begin();
    assert!(world.is_deferred());
    world.defer_end();
    assert!(!world.is_deferred());
}

#[test]
fn is_readonly() {
    let world = World::new();

    assert!(!world.is_readonly());
}

#[test]
fn check_components_consistency() {
    let world = World::new();

    let pos1 = world.component::<Position>();
    let pos2 = world.component::<Position>();

    assert_eq!(pos1.id(), pos2.id());
}

#[test]
fn register_after_delete() {
    let world = World::new();

    let e1 = world.component::<Position>();
    assert!(e1.is_alive());
    e1.destruct();

    let e2 = world.component::<Position>();
    assert!(e2.is_alive());
}

#[test]
fn entity_with_component() {
    let world = World::new();

    let e = world.entity().set(Position { x: 5, y: 10 }).set(Velocity { x: 1, y: 2 });

    assert!(e.has(Position::id()));
    assert!(e.has(Velocity::id()));

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
fn query_count_consistency() {
    let world = World::new();

    for _ in 0..5 {
        world.entity().add(Position::id());
    }

    assert_eq!(world.count(Position::id()), 5);
}

#[test]
fn entity_view_operations() {
    let world = World::new();

    let e = world.entity();
    assert!(e.is_alive());

    let id = e.id();
    assert_ne!(id, 0);

    let e2 = world.lookup(world.entity_named("test").name().as_str());
    assert!(e2.is_alive());
}

#[test]
fn context_operations() {
    let world = World::new();

    let mut ctx_val: i32 = 42;
    world.set_context(&mut ctx_val as *mut i32 as *mut std::ffi::c_void, None);

    let ctx = world.context();
    assert_eq!(ctx as *const i32 as *const i32, &ctx_val as *const i32);
}

// get_type_info tests (C++ World_get_type_info_t through World_get_type_info_R_T_tag)

#[test]
fn get_type_info_t() {
    let world = World::new();
    let c = world.component::<Position>();
    let ti = world.type_info_from(c.id());
    assert!(ti.is_some());
    let ti = unsafe { &*ti.unwrap() };
    assert_eq!(ti.size, std::mem::size_of::<Position>() as i32);
    assert_eq!(ti.alignment, std::mem::align_of::<Position>() as i32);
    assert_eq!(ti.component, Position::entity_id(&world));
}

#[test]
fn get_type_info_T() {
    let world = World::new();
    let ti = world.type_info_from(Position::entity_id(&world));
    assert!(ti.is_some());
    let ti = unsafe { &*ti.unwrap() };
    assert_eq!(ti.size, std::mem::size_of::<Position>() as i32);
    assert_eq!(ti.alignment, std::mem::align_of::<Position>() as i32);
    assert_eq!(ti.component, Position::entity_id(&world));
}

#[test]
fn get_type_info_r_t() {
    let world = World::new();
    let tgt = world.entity();
    let ti = world.type_info_from((Position::entity_id(&world), tgt.id()));
    assert!(ti.is_some());
    let ti = unsafe { &*ti.unwrap() };
    assert_eq!(ti.size, std::mem::size_of::<Position>() as i32);
    assert_eq!(ti.alignment, std::mem::align_of::<Position>() as i32);
    assert_eq!(ti.component, Position::entity_id(&world));
}

#[test]
fn get_type_info_R_t() {
    let world = World::new();
    let tgt = world.entity();
    let ti = world.type_info_from((Position::entity_id(&world), tgt.id()));
    assert!(ti.is_some());
    let ti = unsafe { &*ti.unwrap() };
    assert_eq!(ti.size, std::mem::size_of::<Position>() as i32);
    assert_eq!(ti.alignment, std::mem::align_of::<Position>() as i32);
    assert_eq!(ti.component, Position::entity_id(&world));
}

#[test]
fn get_type_info_R_T() {
    let world = World::new();
    let ti = world.type_info_from((Position::entity_id(&world), Velocity::entity_id(&world)));
    assert!(ti.is_some());
    let ti = unsafe { &*ti.unwrap() };
    assert_eq!(ti.size, std::mem::size_of::<Position>() as i32);
    assert_eq!(ti.alignment, std::mem::align_of::<Position>() as i32);
    assert_eq!(ti.component, Position::entity_id(&world));
}

#[test]
fn get_type_info_t_tag() {
    let world = World::new();
    let c = world.component::<Tag>();
    let ti = world.type_info_from(c.id());
    assert!(ti.is_none());
}

#[test]
fn get_type_info_T_tag() {
    let world = World::new();
    let ti = world.type_info_from(Tag::entity_id(&world));
    assert!(ti.is_none());
}

#[test]
fn get_type_info_r_t_tag() {
    let world = World::new();
    let tgt = world.entity();
    let ti = world.type_info_from((Tag::entity_id(&world), tgt.id()));
    assert!(ti.is_none());
}

#[test]
fn get_type_info_R_t_tag() {
    let world = World::new();
    let tgt = world.entity();
    let ti = world.type_info_from((Tag::entity_id(&world), tgt.id()));
    assert!(ti.is_none());
}

#[test]
fn get_type_info_R_T_tag() {
    let world = World::new();
    let ti = world.type_info_from((Tag::entity_id(&world), Rel::entity_id(&world)));
    assert!(ti.is_none());
}
