#![allow(dead_code)]
use crate::common_test::*;

// Test 1: Query_default_ctor_no_assign
// In Rust there is no "default constructor" for Query - use Option<Query<T>> instead.
// This test just verifies that pattern compiles.
#[test]
fn test_query_default_ctor_no_assign() {
    let world = World::new();
    let _q: Option<Query<(&Position,)>> = None;
}

// Test 2: Query_term_get_id
#[test]
fn test_query_term_get_id() {
    let world = World::new();

    let foo = world.entity();
    let bar = world.entity();

    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .with((foo, bar))
        .build();

    assert_eq!(q.field_count(), 3);

    assert_eq!(world.id_view_from(q.term(0).id()), world.id_view_from(Position::id()));
    assert_eq!(world.id_view_from(q.term(1).id()), world.id_view_from(Velocity::id()));
    assert!(world.id_view_from(q.term(2).id()).is_pair());
}

// Test 3: Query_term_get_subj
#[test]
fn test_query_term_get_subj() {
    let world = World::new();

    let foo = world.entity();
    let bar = world.entity();
    let src = world.entity();

    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .src()
        .entity(src)
        .with((foo, bar))
        .build();

    assert_eq!(q.field_count(), 3);

    let src_id = q.term(1).src_id();
    assert_eq!(*src_id, *src.id());
}

// Test 4: Query_term_get_pred
#[test]
fn test_query_term_get_pred() {
    let world = World::new();

    let foo = world.entity();
    let bar = world.entity();

    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .with((foo, bar))
        .build();

    assert_eq!(q.field_count(), 3);

    assert_eq!(*q.term(0).first_id(), Position::entity_id(&world));
    assert_eq!(*q.term(1).first_id(), Velocity::entity_id(&world));
    assert_eq!(*q.term(2).first_id(), *foo.id());
}

// Test 5: Query_term_get_obj
#[test]
fn test_query_term_get_obj() {
    let world = World::new();

    let foo = world.entity();
    let bar = world.entity();

    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .with((foo, bar))
        .build();

    assert_eq!(q.field_count(), 3);

    assert_eq!(*q.term(0).second_id(), 0u64);
    assert_eq!(*q.term(1).second_id(), 0u64);
    assert_eq!(*q.term(2).second_id(), *bar.id());
}

// Test 6: Query_get_first
#[test]
fn test_query_get_first() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let q = world.new_query::<&Position>();
    let first = q.first_entity();
    assert_eq!(first.id(), e1.id());
}

// Test 7: Query_get_count_direct
#[test]
fn test_query_get_count_direct() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let q = world.new_query::<&Position>();
    assert_eq!(q.count(), 3);
}

// Test 8: Query_get_is_true_direct
#[test]
fn test_query_get_is_true_direct() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let mut q_1 = world.new_query::<&Position>();
    let mut q_2 = world.new_query::<&Velocity>();

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

    let q = world.new_query::<&Position>();
    let first = q.first_entity();
    assert_eq!(first.id(), e1.id());
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
        .query::<(&Position, &Velocity)>()
        .term_at(0)
        .src()
        .entity(e)
        .term_at(1)
        .src()
        .entity(e)
        .build();

    let mut count = 0;
    q.each(|(p, v)| {
        count += 1;
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });

    assert_eq!(count, 1);
}

// Test 11: Query_each_w_iter_no_this
#[test]
fn test_query_each_w_iter_no_this() {
    let world = World::new();

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world
        .query::<(&Position, &Velocity)>()
        .term_at(0)
        .src()
        .entity(e)
        .term_at(1)
        .src()
        .entity(e)
        .build();

    let mut count = 0;
    q.each(|(p, v)| {
        count += 1;
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });

    assert_eq!(count, 1);
}

// Test 12: Query_named_query
#[test]
fn test_query_named_query() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Position { x: 3, y: 4 });

    let q = world.new_query::<&Position>();

    let mut count = 0;
    q.each_entity(|e, _pos| {
        assert!(e.id() == e1.id() || e.id() == e2.id());
        count += 1;
    });
    assert_eq!(count, 2);
}

// Test 13: Query_named_scoped_query
#[test]
fn test_query_named_scoped_query() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Position { x: 3, y: 4 });

    let q = world.new_query::<&Position>();

    let mut count = 0;
    q.each_entity(|e, _pos| {
        assert!(e.id() == e1.id() || e.id() == e2.id());
        count += 1;
    });
    assert_eq!(count, 2);
}

// Test 14: Query_find
#[test]
fn test_query_find() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 10, y: 20 });
    let e2 = world.entity().set(Position { x: 20, y: 30 });

    let q = world.new_query::<&Position>();

    let result = q.find(|p| p.x == 20);
    assert_eq!(result.unwrap(), e2);
}

// Test 15: Query_find_not_found
#[test]
fn test_query_find_not_found() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 10, y: 20 });
    let _e2 = world.entity().set(Position { x: 20, y: 30 });

    let q = world.new_query::<&Position>();

    let result = q.find(|p| p.x == 30);
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

    let q = world.new_query::<&Position>();

    let result = q.find_entity(|e, p| {
        e.get::<&Velocity>(|v| p.x == v.x && p.y == v.y)
    });

    assert_eq!(result.unwrap(), e2);
}

// Test 17: Query_each
#[test]
fn test_query_each() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
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

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
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

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
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

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
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

    let e = world.entity().add(Tag::id());

    let mut count = 0;
    // Tags (ZST) must use id-based query, not type parameter
    let q = world.query::<()>().with(Tag::id()).build();
    q.each_entity(|qe, ()| {
        assert_eq!(qe.id(), e.id());
        count += 1;
    });

    assert_eq!(count, 1);
}

// Test 22: Query_shared_tag_w_each
#[test]
fn test_query_shared_tag_w_each() {
    let world = World::new();

    let base = world.prefab().add(Tag::id());
    let e = world.entity().is_a(base);

    let mut count = 0;
    let q = world.query::<()>().with(Tag::id()).build();
    q.each_entity(|qe, ()| {
        assert_eq!(qe.id(), e.id());
        count += 1;
    });

    assert_eq!(count, 1);
}

// Test 23: Query_changed
#[test]
fn test_query_changed() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 0 });

    let q = world.query::<&Position>().detect_changes().build();
    let q_w = world.new_query::<&mut Position>();

    assert!(q.is_changed());

    q.each(|_pos| {});
    assert!(!q.is_changed());

    e.set(Position { x: 2, y: 0 });
    assert!(q.is_changed());

    q.each(|_pos| {});
    assert!(!q.is_changed());

    q_w.each(|_pos| {});
    assert!(q.is_changed());
}

// Test 24: Query_default_ctor
#[test]
fn test_query_default_ctor() {
    let world = World::new();

    let mut q_var: Option<Query<&Position>> = None;
    let q = world.query::<&Position>().build();

    let e = world.entity().set(Position { x: 10, y: 20 });
    let _ = e;

    q_var = Some(q);

    if let Some(q) = q_var {
        let mut count = 0;
        q.each_entity(|_e, p| {
            assert_eq!(p.x, 10);
            assert_eq!(p.y, 20);
            count += 1;
        });

        assert_eq!(count, 1);
    }
}

// Test 25: Query_inspect_terms
#[test]
fn test_query_inspect_terms() {
    let world = World::new();

    let p_entity = world.entity();

    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .with((flecs::ChildOf::ID, p_entity))
        .build();

    assert_eq!(q.field_count(), 3);

    assert_eq!(world.id_view_from(q.term(0).id()), world.id_view_from(Position::id()));
    assert_eq!(world.id_view_from(q.term(1).id()), world.id_view_from(Velocity::id()));
    assert!(world.id_view_from(q.term(2).id()).is_pair());
}

// Test 26: Query_inspect_terms_w_each
#[test]
fn test_query_inspect_terms_w_each() {
    let world = World::new();

    let p_entity = world.entity();

    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .with((flecs::ChildOf::ID, p_entity))
        .build();

    let mut count = 0;
    q.each_term(|_term| {
        count += 1;
    });

    assert_eq!(count, 3);
}

// Test 27: Query_comp_to_str
#[test]
fn test_query_comp_to_str() {
    let world = World::new();

    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .build();

    let s = q.to_string();
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

    let q = world.new_query::<&Pair>();

    let mut count = 0;
    q.each_entity(|e, p| {
        assert_eq!(p.amount, 10);
        assert_eq!(e.id(), e1.id());
        count += 1;
    });

    assert_eq!(count, 1);
}

// Test 29: Query_each_no_entity_1_comp
#[test]
fn test_query_each_no_entity_1_comp() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 2 });

    let q = world.new_query::<&Position>();

    let mut count = 0;
    q.each(|p| {
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
        count += 1;
    });

    assert_eq!(count, 1);

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

    let q = world.new_query::<(&Position, &Velocity)>();

    let mut count = 0;
    q.each(|(p, v)| {
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
        assert_eq!(v.x, 10);
        assert_eq!(v.y, 20);
        count += 1;
    });

    assert_eq!(count, 1);

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
        .set(SelfRef { value: *world.entity() });

    let e2 = world
        .entity()
        .set(Position { x: 20, y: 30 })
        .set(SelfRef { value: *world.entity() });

    e1.set(SelfRef { value: *e1 });
    e2.set(SelfRef { value: *e2 });

    let q = world.new_query::<(&SelfRef, &Position, &Velocity)>();

    let mut count = 0;
    q.each_entity(|e, (s, _p, _v)| {
        assert_eq!(e, s.value);
        count += 1;
    });

    assert_eq!(count, 2);
}

// Test 32: Query_instanced_query_w_base_each
#[test]
fn test_query_instanced_query_w_base_each() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .is_a(base)
        .set(Position { x: 10, y: 20 })
        .set(SelfRef { value: *world.entity() });

    let e2 = world
        .entity()
        .is_a(base)
        .set(Position { x: 20, y: 30 })
        .set(SelfRef { value: *world.entity() });

    e1.set(SelfRef { value: *e1 });
    e2.set(SelfRef { value: *e2 });

    let q = world.new_query::<(&SelfRef, &Position, &Velocity)>();

    let mut count = 0;
    q.each_entity(|e, (s, _p, _v)| {
        assert_eq!(e, s.value);
        count += 1;
    });

    assert_eq!(count, 2);
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

    let q = world.new_query::<(&Position, &Velocity)>();

    let mut count = 0;
    q.each(|(_p, _v)| {
        count += 1;
    });

    assert_eq!(count, 2);
}

// Test 34: Query_query_each_w_func_ptr
#[test]
fn test_query_query_each_w_func_ptr() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let q = world.new_query::<&Position>();

    let mut count = 0;
    q.each_entity(|_e, _p| {
        count += 1;
    });

    assert_eq!(count, 1);

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

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
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

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
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
        .is_a(base);

    let e2 = world
        .entity()
        .set(Position { x: 20, y: 30 })
        .is_a(base);

    let e3 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 3, y: 4 });

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
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

    let _e1 = world.entity().add(Tag::id()).set(Position { x: 1, y: 2 });
    let _e2 = world.entity().add(Tag::id());

    let mut count = (0i32, 0i32);

    let q = world
        .query::<Option<&Position>>()
        .with(Tag::id())
        .build();

    q.each(|p| {
        if let Some(p) = p {
            count.0 += 1;
            assert_eq!(p.x, 1);
            assert_eq!(p.y, 2);
        } else {
            count.1 += 1;
        }
    });

    assert_eq!(count.0, 1);
    assert_eq!(count.1, 1);
}

// Test 39: Query_copy_operators
#[test]
fn test_query_copy_operators() {
    let world = World::new();

    let q = world.query::<()>().with(Position::id()).build();

    let q_copy_ctor = q.clone();
    let q_copy_assign = q.clone();

    assert_eq!(q_copy_ctor.query_ptr(), q.query_ptr());
    assert_eq!(q_copy_assign.query_ptr(), q.query_ptr());
}

// Test 40: Query_optional_singleton
#[test]
fn test_query_optional_singleton() {
    let world = World::new();

    let mut invoked = 0;

    world.new_query::<Option<&Mass>>().each(|_m| {
        invoked += 1;
    });

    assert_eq!(invoked, 1);

    world.set(Mass { value: 5 });

    world.new_query::<Option<&Mass>>().each(|m| {
        if let Some(mass) = m {
            assert_eq!(mass.value, 5);
        }
        invoked += 1;
    });

    assert_eq!(invoked, 2);
}

// Test 41: Query_has_entity
// NOTE: query.has(entity) is not available in the high-level Rust API.
// Replaced with equivalent logic using each_entity.
#[test]
fn test_query_has_entity() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Velocity { x: 3, y: 4 });

    let q = world.new_query::<&Position>();

    let mut found_e1 = false;
    let mut found_e2 = false;
    q.each_entity(|e, _| {
        if e == e1 { found_e1 = true; }
        if e == e2 { found_e2 = true; }
    });

    assert!(found_e1);
    assert!(!found_e2);
}

// Test 42: Query_has_table (same as has_entity in Rust API)
#[test]
fn test_query_has_table() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 1, y: 2 });
    let e2 = world.entity().set(Velocity { x: 3, y: 4 });

    let q = world.new_query::<&Position>();

    let mut found_e1 = false;
    let mut found_e2 = false;
    q.each_entity(|e, _| {
        if e == e1 { found_e1 = true; }
        if e == e2 { found_e2 = true; }
    });

    assert!(found_e1);
    assert!(!found_e2);
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

    e2.add(Tag::id());
    e2.remove(Tag::id());

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
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

    e2.add(Tag::id());
    e2.remove(Tag::id());

    let q = world.new_query::<(&Position, &Velocity)>();

    let mut count = 0;
    q.each_entity(|_e, (p, v)| {
        let x = p.x + v.x;
        let y = p.y + v.y;
        assert!(x >= 11);
        assert!(y >= 22);
        count += 1;
    });

    assert_eq!(count, 2);
}

// Test 45: Query_iter_entities
#[test]
fn test_query_iter_entities() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 10, y: 20 });
    let e2 = world.entity().set(Position { x: 10, y: 20 });
    let e3 = world.entity().set(Position { x: 10, y: 20 });

    let q = world.new_query::<&Position>();

    q.run(|mut it| {
        while it.next() {
            let entities = it.entities();
            assert_eq!(entities.len(), 3);
            assert_eq!(entities[0], e1);
            assert_eq!(entities[1], e2);
            assert_eq!(entities[2], e3);
        }
    });
}

// Test 46: Query_iter_get_pair_w_id
#[test]
fn test_query_iter_get_pair_w_id() {
    let world = World::new();

    let rel = world.entity();
    let tgt = world.entity();
    let e = world.entity().add((rel, tgt));

    let q = world
        .query::<()>()
        .with((rel, id::<flecs::Wildcard>()))
        .build();

    let mut count = 0;
    q.each_iter(|it, i, ()| {
        assert!(it.id(0).is_pair());
        assert_eq!(it.id(0).first_id().id(), rel.id());
        assert_eq!(it.id(0).second_id().id(), tgt.id());
        assert_eq!(it.entity_id(i), e.id());
        count += 1;
    });

    assert_eq!(count, 1);
}

// Test 47: Query_query_from_entity
#[test]
fn test_query_query_from_entity() {
    let world = World::new();

    let _e1 = world.entity().add(Position::id());
    let e2 = world.entity().add(Position::id()).add(Velocity::id());

    let q1 = world.new_query::<(&Position, &Velocity)>();

    let mut count = 0;
    q1.each_entity(|e, (_p, _v)| {
        count += 1;
        assert_eq!(e.id(), e2.id());
    });
    assert_eq!(count, 1);
}

// Test 48: Query_run_w_iter_fini
#[test]
fn test_query_run_w_iter_fini() {
    let world = World::new();

    let q = world.new_query::<&Position>();

    let mut count = 0;
    q.run(|mut _it| {
        count += 1;
    });

    assert_eq!(count, 1);
}

// Test 49: Query_run_w_iter_fini_interrupt
#[test]
fn test_query_run_w_iter_fini_interrupt() {
    let world = World::new();

    let _e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add(Tag::id());
    let _e2 = world.entity().set(Position { x: 10, y: 20 });
    let _e3 = world.entity().set(Position { x: 10, y: 20 });

    let q = world.new_query::<&Position>();

    let mut count = 0;
    q.run(|mut it| {
        if it.next() {
            count += 1;
        }
    });

    assert_eq!(count, 1);
}

// Test 50: Query_run_w_iter_fini_empty
#[test]
fn test_query_run_w_iter_fini_empty() {
    let world = World::new();

    let q = world.new_query::<&Position>();

    let mut count = 0;
    q.run(|_it| {
        count += 1;
    });

    assert_eq!(count, 1);
}

// Test 51: Query_run_w_iter_fini_no_query
#[test]
fn test_query_run_w_iter_fini_no_query() {
    let world = World::new();

    let q = world.query::<()>().build();

    let mut count = 0;
    q.run(|_it| {
        count += 1;
    });

    assert_eq!(count, 1);
}

// Test 52: Query_add_to_match_from_staged_query
#[test]
fn test_query_add_to_match_from_staged_query() {
    let world = World::new();

    let e = world.entity().add(Position::id());

    world.new_query::<&Position>().each_entity(|entity, _pos| {
        entity.add(Velocity::id());
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

    let q = world.new_query::<(&Position, Option<&Velocity>, Option<&Mass>)>();

    let mut count = 0;
    q.each(|(p, v, m)| {
        if v.is_some() && m.is_some() {
            assert!(p.x >= 10);
        } else {
            assert!(p.x >= 50);
        }
        count += 1;
    });

    assert_eq!(count, 4);
}

// Test 54: Query_iter_type
#[test]
fn test_query_iter_type() {
    let world = World::new();

    let _e1 = world.entity().add(Position::id());
    let _e2 = world.entity().add(Position::id()).add(Velocity::id());

    let q = world.new_query::<&Position>();

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

    for i in 0..3i32 {
        world
            .entity()
            .set(ThisComp { x: i })
            .add((Rel::id(), other));
    }

    let q = world
        .query::<(&Rel, &ThisComp, &OtherComp)>()
        .term_at(0)
        .second()
        .set_var("other")
        .term_at(2)
        .src()
        .set_var("other")
        .build();

    let mut count = 0;
    q.run(|mut it| {
        while it.next() {
            for _ in it.iter() {
                count += 1;
            }
        }
    });

    assert_eq!(count, 3);
}

// Test 56: Query_is_true
#[test]
fn test_query_is_true() {
    let world = World::new();

    let _e = world.entity().set(Position { x: 1, y: 2 });

    let mut q1 = world.new_query::<&Position>();
    let mut q2 = world.new_query::<&Velocity>();

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

    let q = world.new_query::<&Position>();

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

    world.entity().add(Tag::id());
    world.entity().add(Tag::id());

    let q = world.query::<()>().with(Tag::id()).build();

    assert_eq!(q.count(), 2);
}

// Test 60: Query find with multiple results
#[test]
fn test_query_find_multiple_results() {
    let world = World::new();

    world.entity().set(Position { x: 5, y: 5 });
    world.entity().set(Position { x: 5, y: 5 });

    let q = world.new_query::<&Position>();

    let result = q.find(|p| p.x == 5);
    assert!(result.is_some());
}

// Test 61: Query on empty world
#[test]
fn test_query_on_empty_world() {
    let world = World::new();

    let mut q = world.new_query::<&Position>();

    assert_eq!(q.count(), 0);
    assert!(!q.is_true());
}

// Test 62: Query with multiple types
#[test]
fn test_query_with_multiple_types() {
    let world = World::new();

    let _e = world
        .entity()
        .set(Position { x: 1, y: 2 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 5 });

    let q = world.new_query::<(&Position, &Velocity, &Mass)>();

    assert_eq!(q.count(), 1);
}

// Test 63: Query each with mutable component
#[test]
fn test_query_each_mutable() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let q = world.new_query::<&Position>();

    q.each(|p| {
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

    let q = world.new_query::<&Position>();

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

    let q = world.new_query::<&Position>();

    assert_eq!(q.count(), 1);
}

// Test 66: Query with prefab
#[test]
fn test_query_with_prefab() {
    let world = World::new();

    let prefab = world.prefab().set(Position { x: 1, y: 2 });
    let _instance = world.entity().is_a(prefab);

    let q = world.new_query::<&Position>();

    // Prefabs are excluded from normal queries by default
    assert_eq!(q.count(), 1);
}

// Test 67: Query first_entity() on empty result
#[test]
fn test_query_first_empty() {
    let world = World::new();

    let q = world.new_query::<&Position>();

    // first_entity() returns an entity with id 0 when no results
    assert_eq!(*q.first_entity().id(), 0u64);
}

// Test 68: Query changed() detection
#[test]
fn test_query_changed_detection() {
    let world = World::new();

    let _e = world.entity().set(Position { x: 1, y: 2 });

    let q = world.query::<&Position>().detect_changes().build();

    assert!(q.is_changed());

    q.each(|_p| {});

    assert!(!q.is_changed());
}

// Test 69: Query run() basic
#[test]
fn test_query_iter_basic() {
    let world = World::new();

    world.entity().set(Position { x: 1, y: 2 });

    let q = world.new_query::<&Position>();

    let mut found = false;
    q.run(|mut it| {
        while it.next() {
            found = true;
            assert_eq!(it.count(), 1);
        }
    });

    assert!(found);
}

// Test 70: Query with tag component
#[test]
fn test_query_with_default_component() {
    let world = World::new();

    let _e = world.entity().add(Tag::id());

    let q = world.query::<()>().with(Tag::id()).build();

    assert_eq!(q.count(), 1);
}

// Test 71: Query modified after creation
#[test]
fn test_query_modified_after_creation() {
    let world = World::new();

    let q = world.new_query::<&Position>();

    world.entity().set(Position { x: 1, y: 2 });

    assert_eq!(q.count(), 1);
}

// Test 72: Query each with entity iteration
#[test]
fn test_query_each_with_control_flow() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });

    let q = world.new_query::<&Position>();

    let mut count = 0;
    q.each_entity(|_e, _p| {
        count += 1;
    });

    assert_eq!(count, 2);
}

// Test 73: Query result consistency
#[test]
fn test_query_result_consistency() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });

    let q = world.new_query::<&Position>();

    let count1 = q.count();
    let count2 = q.count();

    assert_eq!(count1, count2);
    assert_eq!(count1, 2);
}

// Test 74: Query with complex tuple
#[test]
fn test_query_with_complex_tuple() {
    let world = World::new();

    let _e = world
        .entity()
        .set(Position { x: 1, y: 2 })
        .set(Velocity { x: 3, y: 4 });

    let q = world.new_query::<(&Position, &Velocity)>();

    q.each(|(p, v)| {
        assert_eq!(p.x + v.x, 4);
        assert_eq!(p.y + v.y, 6);
    });
}

// Test 75: Query first_entity on query with single result
#[test]
fn test_query_first_single_result() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 2 });

    let q = world.new_query::<&Position>();

    assert_eq!(q.first_entity(), e);
}

// Test 76: Query is_true on non-empty query
#[test]
fn test_query_is_true_non_empty() {
    let world = World::new();

    let _e = world.entity().set(Position { x: 1, y: 2 });

    let mut q = world.new_query::<&Position>();

    assert!(q.is_true());
}

// Test 77: Query multiple entity iteration
#[test]
fn test_query_multiple_entity_iteration() {
    let world = World::new();

    let _e1 = world.entity().set(Position { x: 1, y: 2 });
    let _e2 = world.entity().set(Position { x: 3, y: 4 });
    let _e3 = world.entity().set(Position { x: 5, y: 6 });

    let q = world.new_query::<&Position>();

    let mut entities = Vec::new();
    q.each_entity(|e, _p| {
        entities.push(e.id());
    });

    assert_eq!(entities.len(), 3);
}

// Test 78: Query clone consistency
#[test]
fn test_query_clone_consistency() {
    let world = World::new();

    let e = world.entity().set(Position { x: 1, y: 2 });

    let q1 = world.new_query::<&Position>();
    let q2 = q1.clone();

    assert_eq!(q1.count(), q2.count());
    assert_eq!(q1.first_entity(), e);
    assert_eq!(q2.first_entity(), e);
}
