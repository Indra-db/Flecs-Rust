#![allow(dead_code)]
use crate::common_test::*;

// Test 1: Query_default_ctor_no_assign
#[test]
fn test_query_default_ctor_no_assign() {
    let _world = World::new();
    let _q: Query<()> = Query::new();
}

// Test 2: Query_term_get_id
#[test]
fn test_query_term_get_id() {
    let world = World::new();

    let foo = world.entity();
    let bar = world.entity();

    let q = world
        .query::<Position>()
        .with::<Velocity>()
        .with_id(foo, bar)
        .build();

    assert_eq!(q.field_count(), 3);

    let t = q.term(0);
    assert_eq!(t.id(), Position::id());

    let t = q.term(1);
    assert_eq!(t.id(), Velocity::id());

    let t = q.term(2);
    assert_eq!(t.id(), world.pair(foo, bar));
}

// Test 3: Query_term_get_subj
#[test]
fn test_query_term_get_subj() {
    let world = World::new();

    let foo = world.entity();
    let bar = world.entity();
    let src = world.entity();

    let q = world
        .query::<Position>()
        .with_id_src::<Velocity>(src)
        .with_id(foo, bar)
        .build();

    assert_eq!(q.field_count(), 3);

    let t = q.term(0);
    assert_eq!(t.get_src(), Id::THIS);

    let t = q.term(1);
    assert_eq!(t.get_src(), src);

    let t = q.term(2);
    assert_eq!(t.get_src(), Id::THIS);
}

// Test 4: Query_term_get_pred
#[test]
fn test_query_term_get_pred() {
    let world = World::new();

    let foo = world.entity();
    let bar = world.entity();

    let q = world
        .query::<Position>()
        .with::<Velocity>()
        .with_id(foo, bar)
        .build();

    assert_eq!(q.field_count(), 3);

    let t = q.term(0);
    assert_eq!(t.get_first(), Position::id());

    let t = q.term(1);
    assert_eq!(t.get_first(), Velocity::id());

    let t = q.term(2);
    assert_eq!(t.get_first(), foo);
}

// Test 5: Query_term_get_obj
#[test]
fn test_query_term_get_obj() {
    let world = World::new();

    let foo = world.entity();
    let bar = world.entity();

    let q = world
        .query::<Position>()
        .with::<Velocity>()
        .with_id(foo, bar)
        .build();

    assert_eq!(q.field_count(), 3);

    let t = q.term(0);
    assert_eq!(t.get_second(), 0);

    let t = q.term(1);
    assert_eq!(t.get_second(), 0);

    let t = q.term(2);
    assert_eq!(t.get_second(), bar);
}

// Test 6: Query_get_first
#[test]
fn test_query_get_first() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let q = world.query::<Position>().build();
    let first = q.iter().first();

    assert_ne!(first, 0);
    assert_eq!(first, e1.id());
}

// Test 7: Query_get_count_direct
#[test]
fn test_query_get_count_direct() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let q = world.query::<Position>().build();
    assert_eq!(q.count(), 3);
}

// Test 8: Query_get_is_true_direct
#[test]
fn test_query_get_is_true_direct() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let q_1 = world.query::<Position>().build();
    let q_2 = world.query::<Velocity>().build();

    assert!(q_1.is_true());
    assert!(!q_2.is_true());
}

// Test 9: Query_get_first_direct
#[test]
fn test_query_get_first_direct() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let q = world.query::<Position>().build();
    let first = q.first();

    assert_ne!(first, 0);
    assert_eq!(first, e1.id());
}

// Test 10: Query_each_w_no_this
#[test]
fn test_query_each_w_no_this() {
    let world = World::new();

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world
        .query_builder::<(Position, Velocity)>()
        .build();

    let count = std::cell::RefCell::new(0i32);
    q.each(|(p, v): (&Position, &Velocity)| {
        *count.borrow_mut() += 1;
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 11: Query_each_w_iter_no_this
#[test]
fn test_query_each_w_iter_no_this() {
    let world = World::new();

    let _e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world
        .query_builder::<(Position, Velocity)>()
        .build();

    let count = std::cell::RefCell::new(0i32);
    q.each(|(p, v): (&Position, &Velocity)| {
        *count.borrow_mut() += 1;
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 12: Query_named_query
#[test]
fn test_query_named_query() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Position { x: 3, y: 4 });

    let q = world.query::<Position>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|e, _pos: &Position| {
        assert!(e.id() == e1.id() || e.id() == e2.id());
        *count.borrow_mut() += 1;
    });
    assert_eq!(*count.borrow(), 2);
}

// Test 13: Query_named_scoped_query
#[test]
fn test_query_named_scoped_query() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Position { x: 3, y: 4 });

    let q = world.query::<Position>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|e, _pos: &Position| {
        assert!(e.id() == e1.id() || e.id() == e2.id());
        *count.borrow_mut() += 1;
    });
    assert_eq!(*count.borrow(), 2);
}

// Test 14: Query_find
#[test]
fn test_query_find() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 10, y: 20 });
    let e2 = world.entity().set(Position { x: 20, y: 30 });

    let q = world.query::<Position>().build();

    let result = q.find(|(p, _)| p.x == 20);
    assert_eq!(result, Some(e2.id()));
}

// Test 15: Query_find_not_found
#[test]
fn test_query_find_not_found() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 10, y: 20 });
    let _e2 = world.entity().set(Position { x: 20, y: 30 });

    let q = world.query::<Position>().build();

    let result = q.find(|(p, _)| p.x == 30);
    assert!(result.is_none());
}

// Test 16: Query_find_w_entity
#[test]
fn test_query_find_w_entity() {
    let world = World::new();

    let _e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 20, y: 30 });
    let e2 = world
        .entity()
        .set(Position { x: 20, y: 30 })
        .set(Velocity { x: 20, y: 30 });

    let q = world.query::<Position>().build();

    let result = q.find_entity(|e, (p, _)| {
        e.get::<&Velocity>(|v| p.x == v.x && p.y == v.y).unwrap_or(false)
    });

    assert_eq!(result, Some(e2.id()));
}

// Test 17: Query_each
#[test]
fn test_query_each() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.query::<(Position, Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

// Test 18: Query_each_const
#[test]
fn test_query_each_const() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.query::<(Position, &Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

// Test 19: Query_each_sparse
#[test]
fn test_query_each_sparse() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.query::<(Position, Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

// Test 20: Query_each_dont_fragment
#[test]
fn test_query_each_dont_fragment() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.query::<(Position, Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

// Test 21: Query_tag_w_each
#[test]
fn test_query_tag_w_each() {
    let world = World::new();

    let q = world.query::<Tag>().build();

    let e = world.entity().add::<Tag>();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|qe, _tag: Tag| {
        assert_eq!(qe.id(), e.id());
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 22: Query_shared_tag_w_each
#[test]
fn test_query_shared_tag_w_each() {
    let world = World::new();

    let q = world.query::<Tag>().build();

    let base = world.prefab().add::<Tag>();
    let e = world.entity().add(flecs_ecs::sys::EcsIsA, base);

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|qe, _tag: Tag| {
        assert_eq!(qe.id(), e.id());
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 23: Query_changed
#[test]
fn test_query_changed() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 0 });

    let q = world.query::<&Position>().build();
    let q_w = world.query::<Position>().build();

    assert!(q.changed());

    q.each(|_pos: &Position| {});
    assert!(!q.changed());

    e.set(Position { x: 2, y: 0 });
    assert!(q.changed());

    q.each(|_pos: &Position| {});
    assert!(!q.changed());

    q_w.each(|_pos: &Position| {});
    assert!(q.changed());
}

// Test 24: Query_default_ctor
#[test]
fn test_query_default_ctor() {
    let world = World::new();

    let mut q_var: Option<Query<Position>> = None;
    let q = world.query::<Position>().build();

    let e = world.entity().set(Position { x: 10, y: 20 });

    q_var = Some(q.clone());

    if let Some(q) = q_var {
        let count = std::cell::RefCell::new(0i32);
        q.each_entity(|_e, p: &Position| {
            assert_eq!(p.x, 10);
            assert_eq!(p.y, 20);
            *count.borrow_mut() += 1;
        });

        assert_eq!(*count.borrow(), 1);
    }
}

// Test 25: Query_inspect_terms
#[test]
fn test_query_inspect_terms() {
    let world = World::new();

    let p = world.entity();

    let q = world
        .query::<Position>()
        .with::<Velocity>()
        .with_id(flecs_ecs::sys::EcsChildOf, p)
        .build();

    assert_eq!(q.field_count(), 3);

    let t = q.term(0);
    assert_eq!(t.id(), Position::id());

    let t = q.term(1);
    assert_eq!(t.id(), Velocity::id());

    let t = q.term(2);
    assert_eq!(t.id(), world.pair(flecs_ecs::sys::EcsChildOf, p));
}

// Test 26: Query_inspect_terms_w_each
#[test]
fn test_query_inspect_terms_w_each() {
    let world = World::new();

    let p = world.entity();

    let q = world
        .query::<Position>()
        .with::<Velocity>()
        .with_id(flecs_ecs::sys::EcsChildOf, p)
        .build();

    let count = std::cell::RefCell::new(0i32);
    q.each_term(|_term| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 3);
}

// Test 27: Query_comp_to_str
#[test]
fn test_query_comp_to_str() {
    let world = World::new();

    let q = world
        .query::<Position>()
        .with::<Velocity>()
        .build();

    let s = q.str();
    assert!(!s.is_empty());
}

// Test 28: Query_each_pair_type
#[test]
fn test_query_each_pair_type() {
    let world = World::new();

    #[derive(Component)]
    struct Pair {
        amount: i32,
    }

    let e1 = world.entity().set(Pair { amount: 10 });

    let q = world.query::<&Pair>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|e, p: &Pair| {
        assert_eq!(p.amount, 10);
        assert_eq!(e.id(), e1.id());
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 29: Query_each_no_entity_1_comp
#[test]
fn test_query_each_no_entity_1_comp() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 2 });

    let q = world.query::<Position>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each(|p: &Position| {
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);

    e.get::<&Position>(|pos| {
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 2);
    });
}

// Test 30: Query_each_no_entity_2_comps
#[test]
fn test_query_each_no_entity_2_comps() {
    let world = World::new();

    let e = world
        .entity()
        .set(Position { x: 1, y: 2 })
        .set(Velocity { x: 10, y: 20 });

    let q = world.query::<(Position, Velocity)>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each(|(p, v): (&Position, &Velocity)| {
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
        assert_eq!(v.x, 10);
        assert_eq!(v.y, 20);
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);

    e.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
        assert_eq!(v.x, 10);
        assert_eq!(v.y, 20);
    });
}

// Test 31: Query_instanced_query_w_singleton_each
#[test]
fn test_query_instanced_query_w_singleton_each() {
    let world = World::new();

    world.set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(SelfRef { value: world.entity() });

    let e2 = world
        .entity()
        .set(Position { x: 20, y: 30 })
        .set(SelfRef { value: world.entity() });

    e1.set(SelfRef { value: e1 });
    e2.set(SelfRef { value: e2 });

    let q = world.query::<(SelfRef, Position, &Velocity)>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|e, (s, _p, _v)| {
        assert_eq!(e.id(), s.value.id());
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 2);
}

// Test 32: Query_instanced_query_w_base_each
#[test]
fn test_query_instanced_query_w_base_each() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .add(flecs_ecs::sys::EcsIsA, base)
        .set(Position { x: 10, y: 20 })
        .set(SelfRef { value: world.entity() });

    let e2 = world
        .entity()
        .add(flecs_ecs::sys::EcsIsA, base)
        .set(Position { x: 20, y: 30 })
        .set(SelfRef { value: world.entity() });

    e1.set(SelfRef { value: e1 });
    e2.set(SelfRef { value: e2 });

    let q = world.query::<(SelfRef, Position, &Velocity)>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|e, (s, _p, _v)| {
        assert_eq!(e.id(), s.value.id());
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 2);
}

// Test 33: Query_query_each_from_component
#[test]
fn test_query_query_each_from_component() {
    let world = World::new();

    let _e1 = world
        .entity()
        .set(Position { x: 1, y: 2 })
        .set(Velocity { x: 1, y: 2 });
    let _e2 = world
        .entity()
        .set(Position { x: 3, y: 4 })
        .set(Velocity { x: 3, y: 4 });

    let q = world.query::<(Position, Velocity)>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each(|_p: &Position, _v: &Velocity| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 2);
}

// Test 34: Query_query_each_w_func_ptr
#[test]
fn test_query_query_each_w_func_ptr() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let q = world.query::<Position>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|_e, _p: &Position| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);

    e.get::<&Position>(|pos| {
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    });
}

// Test 35: Query_run
#[test]
fn test_query_run() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.query::<(Position, Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        let x = p.x + v.x;
        let y = p.y + v.y;
        assert_eq!(x, 11);
        assert_eq!(y, 22);
    });

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });
}

// Test 36: Query_run_const
#[test]
fn test_query_run_const() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.query::<(Position, &Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        let x = p.x + v.x;
        let y = p.y + v.y;
        assert_eq!(x, 11);
        assert_eq!(y, 22);
    });

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });
}

// Test 37: Query_each_shared
#[test]
fn test_query_each_shared() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add(flecs_ecs::sys::EcsIsA, base);

    let e2 = world
        .entity()
        .set(Position { x: 20, y: 30 })
        .add(flecs_ecs::sys::EcsIsA, base);

    let e3 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 3, y: 4 });

    let q = world.query::<(Position, &Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        let x = p.x + v.x;
        let y = p.y + v.y;
        assert!(x >= 11 && y >= 22);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 20);
        assert_eq!(p.y, 30);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });
}

// Test 38: Query_optional_pair_term
#[test]
fn test_query_optional_pair_term() {
    let world = World::new();

    let _e1 = world.entity().add::<Tag>().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().add::<Tag>();

    let count = std::cell::RefCell::new((0i32, 0i32));

    let q = world.query::<(Option<&Position>,)>().with::<Tag>().build();

    q.each(|(p,): (Option<&Position>,)| {
        let mut c = count.borrow_mut();
        if let Some(p) = p {
            c.0 += 1;
            assert_eq!(p.x, 1);
            assert_eq!(p.y, 2);
        } else {
            c.1 += 1;
        }
    });

    let (with, without) = *count.borrow();
    assert_eq!(with, 1);
    assert_eq!(without, 1);
}

// Test 39: Query_copy_operators
#[test]
fn test_query_copy_operators() {
    let world = World::new();

    let q = world.query::<Position>().build();

    let copy_ctor = q.clone();
    let copy_assign = q.clone();

    assert_eq!(copy_ctor.c_ptr(), q.c_ptr());
    assert_eq!(copy_assign.c_ptr(), q.c_ptr());

    let default_init: Query<Position> = Query::new();
    let copy_ctor_default = default_init.clone();
    let copy_assign_default = default_init.clone();

    assert_eq!(copy_ctor_default.c_ptr(), default_init.c_ptr());
    assert_eq!(copy_assign_default.c_ptr(), default_init.c_ptr());
}

// Test 40: Query_optional_singleton
#[test]
fn test_query_optional_singleton() {
    let world = World::new();

    let invoked = std::cell::RefCell::new(0i32);

    world.query::<Option<&Mass>>().build().each(|_m: Option<&Mass>| {
        *invoked.borrow_mut() += 1;
    });

    assert_eq!(*invoked.borrow(), 1);

    world.set(Mass { value: 5 });

    world.query::<Option<&Mass>>().build().each(|m: Option<&Mass>| {
        if let Some(mass) = m {
            assert_eq!(mass.value, 5);
        }
        *invoked.borrow_mut() += 1;
    });

    assert_eq!(*invoked.borrow(), 2);
}

// Test 41: Query_has_entity
#[test]
fn test_query_has_entity() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Velocity { x: 3, y: 4 });

    let q = world.query::<Position>().build();

    assert!(q.has(e1));
    assert!(!q.has(e2));
}

// Test 42: Query_has_table
#[test]
fn test_query_has_table() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Velocity { x: 3, y: 4 });

    let q = world.query::<Position>().build();

    assert!(q.has(e1));
    assert!(!q.has(e2));
}

// Test 43: Query_empty_tables_each
#[test]
fn test_query_empty_tables_each() {
    let world = World::new();

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let e2 = world
        .entity()
        .set(Position { x: 20, y: 30 })
        .set(Velocity { x: 2, y: 3 });

    e2.add::<Tag>();
    e2.remove::<Tag>();

    let q = world.query::<(Position, Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        let x = p.x + v.x;
        let y = p.y + v.y;
        assert!(x >= 11);
        assert!(y >= 22);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 20);
        assert_eq!(p.y, 30);
    });
}

// Test 44: Query_empty_tables_each_w_entity
#[test]
fn test_query_empty_tables_each_w_entity() {
    let world = World::new();

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let e2 = world
        .entity()
        .set(Position { x: 20, y: 30 })
        .set(Velocity { x: 2, y: 3 });

    e2.add::<Tag>();
    e2.remove::<Tag>();

    let q = world.query::<(Position, Velocity)>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|_e, (p, v): (&Position, &Velocity)| {
        let x = p.x + v.x;
        let y = p.y + v.y;
        assert!(x >= 11);
        assert!(y >= 22);
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 2);
}

// Test 45: Query_iter_entities
#[test]
fn test_query_iter_entities() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 10, y: 20 });
    let e2 = world.entity().set(Position { x: 10, y: 20 });
    let e3 = world.entity().set(Position { x: 10, y: 20 });

    let q = world.query::<Position>().build();

    q.run(|mut it| {
        while it.next() {
            let entities = it.entities();
            assert_eq!(entities.len(), 3);
            assert_eq!(entities[0], e1.id());
            assert_eq!(entities[1], e2.id());
            assert_eq!(entities[2], e3.id());
        }
    });
}

// Test 46: Query_iter_get_pair_w_id
#[test]
fn test_query_iter_get_pair_w_id() {
    let world = World::new();

    let rel = world.entity();
    let tgt = world.entity();
    let e = world.entity().add_id(rel, tgt);

    let q = world
        .query_builder::<()>()
        .with_id_second(rel, flecs_ecs::sys::EcsWildcard)
        .build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|entity, _| {
        assert_eq!(entity.id(), e.id());
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 47: Query_query_from_entity
#[test]
fn test_query_query_from_entity() {
    let world = World::new();

    let _qe = world.entity();
    let q1 = world
        .query_builder::<(Position, Velocity)>()
        .build();

    let _e1 = world.entity().add::<Position>();
    let e2 = world.entity().add::<Position>().add::<Velocity>();

    let count = std::cell::RefCell::new(0i32);
    q1.each_entity(|e, (_p, _v)| {
        *count.borrow_mut() += 1;
        assert_eq!(e.id(), e2.id());
    });
    assert_eq!(*count.borrow(), 1);

    let q2 = world.query::<()>().build();
    q2.each_entity(|e, ()| {
        if e.id() == e2.id() {
            *count.borrow_mut() += 1;
        }
    });
}

// Test 48: Query_run_w_iter_fini
#[test]
fn test_query_run_w_iter_fini() {
    let world = World::new();

    let q = world.query::<Position>().build();

    let count = std::cell::RefCell::new(0i32);
    q.run(|mut _it| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 49: Query_run_w_iter_fini_interrupt
#[test]
fn test_query_run_w_iter_fini_interrupt() {
    let world = World::new();

    let _e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add::<Tag>();
    let _e2 = world.entity().set(Position { x: 10, y: 20 });
    let _e3 = world.entity().set(Position { x: 10, y: 20 });

    let q = world.query::<Position>().build();

    let count = std::cell::RefCell::new(0i32);
    q.run(|mut it| {
        if it.next() {
            *count.borrow_mut() += 1;
        }
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 50: Query_run_w_iter_fini_empty
#[test]
fn test_query_run_w_iter_fini_empty() {
    let world = World::new();

    let q = world.query::<Position>().build();

    let count = std::cell::RefCell::new(0i32);
    q.run(|_it| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 51: Query_run_w_iter_fini_no_query
#[test]
fn test_query_run_w_iter_fini_no_query() {
    let world = World::new();

    let q = world.query::<()>().build();

    let count = std::cell::RefCell::new(0i32);
    q.run(|_it| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1);
}

// Test 52: Query_add_to_match_from_staged_query
#[test]
fn test_query_add_to_match_from_staged_query() {
    let world = World::new();

    let e = world.entity().add::<Position>();

    world.query::<Position>().build().each_entity(|entity, _pos: &Position| {
        entity.add::<Velocity>();
    });

    assert!(e.has(Position::id()));
}

// Test 53: Query_each_optional
#[test]
fn test_query_each_optional() {
    let world = World::new();

    let _e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 1 });

    let _e2 = world
        .entity()
        .set(Position { x: 30, y: 40 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 1 });

    let _e3 = world.entity().set(Position { x: 50, y: 60 });

    let _e4 = world.entity().set(Position { x: 70, y: 80 });

    let q = world
        .query::<(Position, Option<&Velocity>, Option<&Mass>)>()
        .build();

    let count = std::cell::RefCell::new(0i32);
    q.each(|(p, v, m): (&Position, Option<&Velocity>, Option<&Mass>)| {
        if v.is_some() && m.is_some() {
            assert!(p.x >= 10);
        } else {
            assert!(p.x >= 50);
        }
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 4);
}

// Test 54: Query_iter_type
#[test]
fn test_query_iter_type() {
    let world = World::new();

    let _e1 = world.entity().add::<Position>();
    let _e2 = world.entity().add::<Position>().add::<Velocity>();

    let q = world.query::<Position>().build();

    q.run(|mut it| {
        while it.next() {
            assert!(it.count() >= 1);
        }
    });
}

// Test 55: Query_pair_with_variable_src
#[test]
fn test_query_pair_with_variable_src() {
    let world = World::new();

    #[derive(Component)]
    struct Rel;

    #[derive(Component)]
    struct ThisComp {
        x: i32,
    }

    #[derive(Component)]
    struct OtherComp {
        x: i32,
    }

    let other = world.entity().set(OtherComp { x: 10 });

    for i in 0..3 {
        world
            .entity()
            .set(ThisComp { x: i })
            .add_id(Rel::id(), other.id());
    }

    let q = world.query::<(&Rel, &ThisComp, &OtherComp)>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each(|(_rel, _this, _other): (&Rel, &ThisComp, &OtherComp)| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 3);
}

// Test 56: Query_is_true
#[test]
fn test_query_is_true() {
    let world = World::new();

    let _e = world.entity().set(Position { x: 1, y: 2 });

    let q1 = world.query::<Position>().build();
    let q2 = world.query::<Velocity>().build();

    assert!(q1.is_true());
    assert!(!q2.is_true());
}

// Test 57: Query_count
#[test]
fn test_query_count() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let q = world.query::<Position>().build();

    assert_eq!(q.count(), 3);
}

// Test 58: Query with no components
#[test]
fn test_query_with_no_components() {
    let world = World::new();

    let _e1 = world.entity();
    let _e2 = world.entity();

    let q = world.query::<()>().build();

    assert!(q.count() >= 0);
}

// Test 59: Query with Tag only
#[test]
fn test_query_with_tag_only() {
    let world = World::new();

    let e1 = world.entity().add::<Tag>();
    let e2 = world.entity().add::<Tag>();

    let q = world.query::<Tag>().build();

    assert_eq!(q.count(), 2);
}

// Test 60: Query find with multiple results
#[test]
fn test_query_find_multiple_results() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 5, y: 5 });
    let e2 = world.entity().set(Position { x: 5, y: 5 });

    let q = world.query::<Position>().build();

    let result = q.find(|(p, _)| p.x == 5);
    assert!(result.is_some());
}

// Test 61: Query on empty world
#[test]
fn test_query_on_empty_world() {
    let world = World::new();

    let q = world.query::<Position>().build();

    assert_eq!(q.count(), 0);
    assert!(!q.is_true());
}

// Test 62: Query with multiple types
#[test]
fn test_query_with_multiple_types() {
    let world = World::new();

    let e = world
        .entity()
        .set(Position { x: 1, y: 2 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 5 });

    let q = world.query::<(Position, Velocity, Mass)>().build();

    assert_eq!(q.count(), 1);
}

// Test 63: Query each with mutable component
#[test]
fn test_query_each_mutable() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let q = world.query::<Position>().build();

    q.each(|p: &Position| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    });
}

// Test 64: Query with renamed entity
#[test]
fn test_query_with_renamed_entity() {
    let world = World::new();

    let e = world.entity_named("MyEntity").set(Position { x: 1, y: 2 });

    let q = world.query::<Position>().build();

    assert_eq!(q.count(), 1);
    assert_eq!(e.name(), "MyEntity");
}

// Test 65: Query with child entity
#[test]
fn test_query_with_child_entity() {
    let world = World::new();

    let parent = world.entity_named("Parent");
    let _child = world
        .entity_named("Child")
        .child_of(parent)
        .set(Position { x: 1, y: 2 });

    let q = world.query::<Position>().build();

    assert_eq!(q.count(), 1);
}

// Test 66: Query with prefab
#[test]
fn test_query_with_prefab() {
    let world = World::new();

    let prefab = world.prefab().set(Position { x: 1, y: 2 });
    let _instance = world.entity().add(flecs_ecs::sys::EcsIsA, prefab);

    let q = world.query::<Position>().build();

    assert_eq!(q.count(), 1);
}

// Test 67: Query first() on empty result
#[test]
fn test_query_first_empty() {
    let world = World::new();

    let q = world.query::<Position>().build();

    assert_eq!(q.first(), 0);
}

// Test 68: Query changed() detection
#[test]
fn test_query_changed_detection() {
    let world = World::new();

    let _e = world.entity().set(Position { x: 1, y: 2 });

    let q = world.query::<&Position>().build();

    assert!(q.changed());

    q.each(|_p: &Position| {});

    assert!(!q.changed());
}

// Test 69: Query iter() basic
#[test]
fn test_query_iter_basic() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 2 });

    let q = world.query::<Position>().build();

    let mut found = false;
    q.run(|mut it| {
        while it.next() {
            found = true;
            assert_eq!(it.count(), 1);
        }
    });

    assert!(found);
}

// Test 70: Query with default component
#[test]
fn test_query_with_default_component() {
    let world = World::new();

    let e = world.entity().add::<Tag>();

    let q = world.query::<Tag>().build();

    assert_eq!(q.count(), 1);
}

// Test 71: Query modified after creation
#[test]
fn test_query_modified_after_creation() {
    let world = World::new();

    let q = world.query::<Position>().build();

    let e = world.entity().set(Position { x: 1, y: 2 });

    assert_eq!(q.count(), 1);
    assert_eq!(q.first(), e.id());
}

// Test 72: Query each with early return
#[test]
fn test_query_each_with_control_flow() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });

    let q = world.query::<Position>().build();

    let count = std::cell::RefCell::new(0i32);
    q.each_entity(|_e, _p: &Position| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 2);
}

// Test 73: Query result consistency
#[test]
fn test_query_result_consistency() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });

    let q = world.query::<Position>().build();

    let count1 = q.count();
    let count2 = q.count();

    assert_eq!(count1, count2);
    assert_eq!(count1, 2);
}

// Test 74: Query with complex tuple
#[test]
fn test_query_with_complex_tuple() {
    let world = World::new();

    let e = world
        .entity()
        .set(Position { x: 1, y: 2 })
        .set(Velocity { x: 3, y: 4 });

    let q = world.query::<(Position, Velocity)>().build();

    q.each(|(p, v): (&Position, &Velocity)| {
        assert_eq!(p.x + v.x, 4);
        assert_eq!(p.y + v.y, 6);
    });
}

// Test 75: Query first on query with single result
#[test]
fn test_query_first_single_result() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 2 });

    let q = world.query::<Position>().build();

    assert_eq!(q.first(), e.id());
}

// Test 76: Query is_true on non-empty query
#[test]
fn test_query_is_true_non_empty() {
    let world = World::new();

    let _e = world.entity().set(Position { x: 1, y: 2 });

    let q = world.query::<Position>().build();

    assert!(q.is_true());
}

// Test 77: Query multiple entity iteration
#[test]
fn test_query_multiple_entity_iteration() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Position { x: 3, y: 4 });
    let e3 = world.entity().set(Position { x: 5, y: 6 });

    let q = world.query::<Position>().build();

    let entities = std::cell::RefCell::new(Vec::new());
    q.each_entity(|e, _p: &Position| {
        entities.borrow_mut().push(e.id());
    });

    let collected = entities.borrow();
    assert_eq!(collected.len(), 3);
}

// Test 78: Query clone consistency
#[test]
fn test_query_clone_consistency() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 2 });

    let q1 = world.query::<Position>().build();
    let q2 = q1.clone();

    assert_eq!(q1.count(), q2.count());
    assert_eq!(q1.first(), q2.first());
    assert_eq!(q1.first(), e.id());
}
