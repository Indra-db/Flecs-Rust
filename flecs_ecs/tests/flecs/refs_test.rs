#![allow(dead_code)]
use crate::common_test::*;

// ─── Basic ref access ────────────────────────────────────────────────────────

#[test]
fn refs_get_ref_by_ptr() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let mut r = e.cached_ref(id::<Position>());
    r.get(|pos| {
        assert!(pos.x == 10);
        assert!(pos.y == 20);
    });
}

#[test]
fn refs_get_ref_by_method() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let mut r = e.cached_ref(id::<Position>());
    r.get(|pos| {
        assert!(pos.x == 10);
        assert!(pos.y == 20);
    });
}

// ─── Ref stability after structural changes ──────────────────────────────────

#[test]
fn refs_ref_after_add() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let mut r = e.cached_ref(id::<Position>());

    e.add(id::<Velocity>());
    r.get(|pos| {
        assert!(pos.x == 10);
        assert!(pos.y == 20);
    });
}

#[test]
fn refs_ref_after_remove() {
    let world = World::new();

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 1 });

    let mut r = e.cached_ref(id::<Position>());

    e.remove(id::<Velocity>());
    r.get(|pos| {
        assert!(pos.x == 10);
        assert!(pos.y == 20);
    });
}

#[test]
fn refs_ref_after_set() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let mut r = e.cached_ref(id::<Position>());

    e.set(Velocity { x: 1, y: 1 });
    r.get(|pos| {
        assert!(pos.x == 10);
        assert!(pos.y == 20);
    });
}

#[test]
fn refs_ref_before_set() {
    let world = World::new();

    let e = world.entity();
    // Note: CachedRef created before the component is set.
    // ecs_ref_init_id is called with a valid entity so entity field is non-zero.
    let mut r = e.cached_ref(id::<Position>());

    e.set(Position { x: 10, y: 20 });

    r.get(|pos| {
        assert!(pos.x == 10);
        assert!(pos.y == 20);
    });
}

// ─── Mutable ref ─────────────────────────────────────────────────────────────

#[test]
fn refs_non_const_ref() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });
    let mut r = e.cached_ref(id::<Position>());
    r.get(|pos| {
        pos.x += 1;
    });

    e.get::<&Position>(|pos| {
        assert_eq!(pos.x, 11);
    });
}

// ─── Pair refs ───────────────────────────────────────────────────────────────

#[test]
fn refs_pair_ref() {
    let world = World::new();

    // C++: world.entity().set<Position, Tag>({10, 20}) → Position stored at (Position, Tag) pair.
    // Tag is a ZST so Position is the data; use set_pair::<Position, Tag>.
    let e = world
        .entity()
        .set_pair::<Position, Tag>(Position { x: 10, y: 20 });

    // (id::<Position>(), id::<Tag>()) → pair id, CastType = Position (first non-tag element).
    let mut r = e.cached_ref((id::<Position>(), id::<Tag>()));
    r.get(|pos| {
        pos.x += 1;
    });

    e.get::<&(Position, Tag)>(|pos| {
        assert_eq!(pos.x, 11);
    });
}

#[test]
fn refs_pair_ref_w_pair_type() {
    let world = World::new();

    // PositionTag = flecs::pair<Position, Tag> in C++ — same as (Position, Tag) pair.
    let e = world
        .entity()
        .set_pair::<Position, Tag>(Position { x: 10, y: 20 });

    let mut r = e.cached_ref((id::<Position>(), id::<Tag>()));
    r.get(|pos| {
        pos.x += 1;
    });

    e.get::<&(Position, Tag)>(|pos| {
        assert_eq!(pos.x, 11);
    });
}

#[test]
fn refs_pair_ref_w_pair_type_second() {
    let world = World::new();

    // TagPosition = flecs::pair<Tag, Position> — Tag is ZST first, Position is data second.
    let e = world
        .entity()
        .set_pair::<Tag, Position>(Position { x: 10, y: 20 });

    let mut r = e.cached_ref((id::<Tag>(), id::<Position>()));
    r.get(|pos| {
        pos.x += 1;
    });

    e.get::<&(Tag, Position)>(|pos| {
        assert_eq!(pos.x, 11);
    });
}

#[test]
fn refs_pair_ref_w_entity() {
    let world = World::new();

    // C++: set<Position>(tag, {10, 20}) — runtime entity as second, Position as first data.
    let tag = world.entity();
    let e = world
        .entity()
        .set_first::<Position>(Position { x: 10, y: 20 }, tag);

    // Build pair id from typed Position id and runtime entity id.
    // Use CachedRef::new directly with the pair id to get a typed CachedRef<Position>.
    let pos_id = *world.component_id::<Position>();
    let pair_id = Id::new(ecs_pair(pos_id, *tag.id()));
    let mut r = CachedRef::<Position>::new(&world, e.id(), pair_id);
    r.get(|pos| {
        pos.x += 1;
    });

    let ptr = e.get_first_untyped::<Position>(tag) as *const Position;
    let x = unsafe { (*ptr).x };
    assert_eq!(x, 11);
}

#[test]
fn refs_pair_ref_second() {
    let world = World::new();

    // C++: set_second<Position>(tag, {10, 20}) — runtime entity as first, Position as second data.
    let tag = world.entity();
    let e = world
        .entity()
        .set_second::<Position>(tag, Position { x: 10, y: 20 });

    // Build pair id: (tag, Position).
    let pos_id = *world.component_id::<Position>();
    let pair_id = Id::new(ecs_pair(*tag.id(), pos_id));
    let mut r = CachedRef::<Position>::new(&world, e.id(), pair_id);
    r.get(|pos| {
        pos.x += 1;
    });

    let ptr = e.get_second_untyped::<Position>(tag) as *const Position;
    let x = unsafe { (*ptr).x };
    assert_eq!(x, 11);
}

// ─── Stage ref ───────────────────────────────────────────────────────────────

#[test]
fn refs_from_stage() {
    let world = World::new();
    // world.stage(0) gives the default stage (mirrors C++ world.get_stage(0)).
    let stage = world.stage(0);
    let e = stage.entity().set(Position { x: 10, y: 20 });
    let mut r = e.cached_ref(id::<Position>());
    r.get(|pos| {
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    });
}

// ─── Default constructor / construction from entity ──────────────────────────

#[test]
fn refs_default_ctor() {
    let world = World::new();

    // C++ pattern: flecs::ref<Position> p; ... p = e.get_ref<Position>();
    // Rust: just create it directly; no separate default-then-assign needed.
    let e = world.entity().set(Position { x: 10, y: 20 });

    let mut p = e.cached_ref(id::<Position>());
    p.get(|pos| {
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    });
}

#[test]
fn refs_ctor_from_entity() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    // Mirrors C++ flecs::ref<Position> p(e) — explicit construction via CachedRef::new.
    let mut p = CachedRef::<Position>::new(&world, e.id(), id::<Position>());
    p.get(|pos| {
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    });
}

// ─── bool / has semantics ────────────────────────────────────────────────────

#[test]
fn refs_implicit_operator_bool() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let mut p = e.cached_ref(id::<Position>());

    // C++ `test_assert(p)` — has() returns true when component is present.
    assert!(p.has());
}

#[test]
fn refs_try_get() {
    let world = World::new();

    // C++: flecs::ref<Position> p; test_assert(p.try_get() == nullptr);
    // An entity with no Position set; try_get should return None.
    let e = world.entity(); // no Position
    let mut p = e.cached_ref(id::<Position>());

    let result = p.try_get(|_pos| ());
    assert!(result.is_none());
}

#[test]
fn refs_try_get_after_delete() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let mut p = e.cached_ref(id::<Position>());

    // Before delete: should return Some.
    let result = p.try_get(|pos| {
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    });
    assert!(result.is_some());

    // Delete the entity (destruct consumes e).
    e.destruct();

    // After delete: entity no longer exists, try_get on the same CachedRef should return None.
    // ecs_ref_get_id returns null for deleted entities.
    let result2 = p.try_get(|_| ());
    assert!(result2.is_none());
}

#[test]
fn refs_has() {
    let world = World::new();

    let e = world.entity();

    {
        let mut p = e.cached_ref(id::<Position>());
        assert!(!p.has());
    }

    e.set(Position { x: 10, y: 20 });

    {
        let mut p = e.cached_ref(id::<Position>());
        assert!(p.has());
    }
}

#[test]
fn refs_bool_operator() {
    let world = World::new();

    let e = world.entity();

    {
        let mut p = e.cached_ref(id::<Position>());
        assert!(!p.has());
    }

    e.set(Position { x: 10, y: 20 });

    {
        let mut p = e.cached_ref(id::<Position>());
        assert!(p.has());
    }
}

// ─── Base type ref (get_ref_w_id) ────────────────────────────────────────────

// C++ tests Refs_base_type and Refs_empty_base_type use get_ref_w_id<Base>(world.id<Derived>())
// which creates a typed ref for Base but backed by the Derived component storage.
//
// Rust equivalent: CachedRef::new with the Derived component id but typed as Base.
// Base occupies the first sizeof(Base) bytes of Derived's memory layout.
// For the empty-base case, use CachedRef<c_void> since BaseEmpty is a ZST in Rust.

// Mirrors C++ struct Base { int x; } and struct Derived : Base { int y; }.
// In Rust: same field layout — Derived starts with x at offset 0, then y.
#[derive(Component, Clone, Copy)]
struct BaseStruct {
    pub x: i32,
}

#[derive(Component, Clone, Copy)]
struct DerivedStruct {
    pub x: i32,
    pub y: i32,
}

#[test]
fn refs_base_type() {
    let world = World::new();

    let e = world.entity().set(DerivedStruct { x: 10, y: 20 });

    // Create a ref typed as BaseStruct but backed by the DerivedStruct component storage.
    // ecs_ref_init_id / ecs_ref_get_id operate on the DerivedStruct column; the returned
    // pointer is cast to *mut BaseStruct.  Valid because BaseStruct's field x sits at
    // offset 0 of DerivedStruct.
    let mut r = CachedRef::<BaseStruct>::new(&world, e.id(), world.component_id::<DerivedStruct>());
    r.get(|b| {
        assert_eq!(b.x, 10);
    });
}

// Mirrors C++ struct BaseEmpty {} and struct DerivedFromEmpty : BaseEmpty { int y; }.
// BaseEmpty is a ZST in Rust so we cannot use CachedRef<BaseEmpty> (const assert fires).
// Use CachedRef<c_void> instead and cast the raw pointer.
#[derive(Component, Clone, Copy)]
struct DerivedFromEmptyStruct {
    pub y: i32,
}

#[test]
fn refs_empty_base_type() {
    let world = World::new();

    let e = world
        .entity()
        .set(DerivedFromEmptyStruct { y: 20 });

    // Untyped ref backed by DerivedFromEmptyStruct — mirrors get_ref_w_id<BaseEmpty>(derived_id).
    let mut r: CachedRef<core::ffi::c_void> =
        CachedRef::new(&world, e.id(), world.component_id::<DerivedFromEmptyStruct>());

    r.get(|ptr| {
        let d = unsafe { &*(ptr as *const DerivedFromEmptyStruct) };
        assert_eq!(d.y, 20);
    });
}

// ─── ref.component() ─────────────────────────────────────────────────────────

#[test]
fn refs_get_component() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let r = e.cached_ref(id::<Position>());
    // r.component() returns an IdView; compare its raw id to world.component_id::<Position>().
    assert_eq!(*r.component().id(), *world.component_id::<Position>());
}

// ─── Untyped ref ─────────────────────────────────────────────────────────────

#[test]
fn refs_untyped_get_ref_by_method() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    // Pass a raw Entity id (CastType = c_void) to get an untyped CachedRef.
    let pos_entity_id = world.component_id::<Position>();
    let mut r: CachedRef<core::ffi::c_void> = e.cached_ref(*pos_entity_id);
    r.get(|ptr| {
        let pos = unsafe { &*(ptr as *const Position) };
        assert!(pos.x == 10);
        assert!(pos.y == 20);
    });
}

#[test]
fn refs_untyped_pair_ref() {
    let world = World::new();

    let tag = world.entity();
    let e = world
        .entity()
        .set_first::<Position>(Position { x: 10, y: 20 }, tag);

    // Build untyped pair id and get an untyped CachedRef.
    let pos_id = *world.component_id::<Position>();
    let pair_id = Id::new(ecs_pair(pos_id, *tag.id()));
    let mut r: CachedRef<core::ffi::c_void> = e.cached_ref(pair_id);
    r.get(|ptr| {
        let pos = unsafe { &mut *(ptr as *mut Position) };
        pos.x += 1;
    });

    let ptr = e.get_first_untyped::<Position>(tag) as *const Position;
    let x = unsafe { (*ptr).x };
    assert_eq!(x, 11);
}

// ─── Untyped ref with runtime component ──────────────────────────────────────

#[test]
fn refs_untyped_runtime_component_ref() {
    let world = World::new();

    // Create a runtime component "RuntimePosition" with two i32 members,
    // mirroring the C++: world.component("Position").member(flecs::I32,"x").member(flecs::I32,"y")
    let position = world
        .component_untyped_named("RuntimePosition")
        .member(i32::id(), "x")
        .member(i32::id(), "y");

    let e = world.entity();

    // Runtime components have no Rust type info / constructor; use unchecked add.
    // SAFETY: we immediately write all fields via the cursor below, so storage
    // is fully initialized before any read.
    unsafe { e.add_id_unchecked(position) };

    // Write x=10, y=20 via meta cursor.
    let ptr = e.get_untyped_mut(position);
    // SAFETY: ptr is valid mutable storage for the RuntimePosition component;
    // position is a valid flecs type id with struct meta information.
    let mut cur = unsafe { world.cursor_id(position, ptr) };
    cur.push();
    cur.member("x");
    cur.set_int(10);
    cur.member("y");
    cur.set_int(20);
    cur.pop();

    // Obtain an untyped cached ref and read back through the cursor.
    let mut r: CachedRef<core::ffi::c_void> = e.cached_ref(position);
    r.get(|rptr| {
        // SAFETY: same as above — RuntimePosition storage with meta describing two i32 fields.
        let mut cur2 = unsafe { world.cursor_id(position, rptr) };
        cur2.push();
        cur2.member("x");
        assert_eq!(cur2.get_int(), 10);
        cur2.member("y");
        assert_eq!(cur2.get_int(), 20);
        cur2.pop();
    });
}

// ─── ref.world() ─────────────────────────────────────────────────────────────

#[test]
fn refs_ref_world() {
    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 20 });

    let r = e.cached_ref(id::<Position>());
    // r.world() returns a WorldRef; compare its raw pointer to &world's pointer.
    assert_eq!(
        r.world().world_ptr_mut() as *const _,
        (&world).world_ptr_mut() as *const _
    );
}
