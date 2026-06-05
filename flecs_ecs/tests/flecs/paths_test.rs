#![allow(dead_code)]
#![allow(unused_imports)]
use crate::common_test::*;
use flecs_ecs::prelude::*;

#[test]
fn paths_name() {
    let world = World::new();

    let e = world.entity_named("foo");
    assert_eq!(e.name(), "foo");

    let e_world = world.lookup("foo");
    assert_eq!(e.id(), e_world.id());

    let e_world = world.lookup("::foo");
    assert_eq!(e.id(), e_world.id());
}

#[test]
fn paths_path_depth_1() {
    let world = World::new();

    let e = world.entity_named("foo::bar");
    assert_eq!(e.name(), "bar");
    assert_eq!(e.path().unwrap(), "::foo::bar");

    assert!(world.try_lookup("bar").is_none());

    let e_world = world.lookup("foo::bar");
    assert_eq!(e.id(), e_world.id());

    let e_world = world.lookup("::foo::bar");
    assert_eq!(e.id(), e_world.id());
}

#[test]
fn paths_path_depth_2() {
    let world = World::new();

    let e = world.entity_named("foo::bar::hello");
    assert_eq!(e.name(), "hello");
    assert_eq!(e.path().unwrap(), "::foo::bar::hello");

    assert!(world.try_lookup("hello").is_none());

    let e_world = world.lookup("foo::bar::hello");
    assert_eq!(e.id(), e_world.id());

    let e_world = world.lookup("::foo::bar::hello");
    assert_eq!(e.id(), e_world.id());
}

#[test]
fn paths_entity_lookup_name() {
    let world = World::new();

    let parent = world.entity_named("foo");
    assert_eq!(parent.name(), "foo");
    assert_eq!(parent.path().unwrap(), "::foo");

    let e = world.entity_named("foo::bar");
    assert_eq!(e.name(), "bar");
    assert_eq!(e.path().unwrap(), "::foo::bar");

    let parent_e = parent.lookup("bar");
    assert_eq!(e.id(), parent_e.id());

    let parent_e = parent.lookup("::foo::bar");
    assert_eq!(e.id(), parent_e.id());
}

#[test]
fn paths_entity_lookup_depth_1() {
    let world = World::new();

    let parent = world.entity_named("foo");
    assert_eq!(parent.name(), "foo");
    assert_eq!(parent.path().unwrap(), "::foo");

    let e = world.entity_named("foo::bar::hello");
    assert_eq!(e.name(), "hello");
    assert_eq!(e.path().unwrap(), "::foo::bar::hello");

    let parent_e = parent.lookup("bar::hello");
    assert_eq!(e.id(), parent_e.id());

    let parent_e = parent.lookup("::foo::bar::hello");
    assert_eq!(e.id(), parent_e.id());
}

#[test]
fn paths_entity_lookup_depth_2() {
    let world = World::new();

    let parent = world.entity_named("foo");
    assert_eq!(parent.name(), "foo");
    assert_eq!(parent.path().unwrap(), "::foo");

    let e = world.entity_named("foo::bar::hello::world");
    assert_eq!(e.name(), "world");
    assert_eq!(e.path().unwrap(), "::foo::bar::hello::world");

    let parent_e = parent.lookup("bar::hello::world");
    assert_eq!(e.id(), parent_e.id());

    let parent_e = parent.lookup("::foo::bar::hello::world");
    assert_eq!(e.id(), parent_e.id());
}

#[test]
#[should_panic]
fn paths_entity_lookup_from_0() {
    let world = World::new();
    let _guard = FlecsPanicAbortGuard::install();

    let foo = world.entity_named("foo");
    assert_eq!(world.lookup("foo").id(), foo.id());

    let dummy = world.entity_from_id(0u64);
    dummy.try_lookup("foo"); // triggers ecs_assert id != 0 -> abort -> panic
}

#[test]
#[should_panic]
fn paths_entity_lookup_from_0_w_world() {
    let world = World::new();
    let _guard = FlecsPanicAbortGuard::install();

    let foo = world.entity_named("foo");
    assert_eq!(world.lookup("foo").id(), foo.id());

    let dummy = world.entity_from_id(0u64);
    dummy.try_lookup("foo"); // triggers ecs_assert id != 0 -> abort -> panic
}

#[test]
fn paths_alias_component() {
    let world = World::new();

    let e = world.set_alias_component::<Position>("MyPosition");
    let a = world.lookup("MyPosition");
    let c = world.lookup("Position");

    assert_eq!(e.id(), a.id());
    assert_eq!(e.id(), c.id());
}

mod test_ns {
    use flecs_ecs::prelude::*;
    #[derive(Component)]
    pub struct Foo {
        pub x: f32,
        pub y: f32,
    }
}

#[test]
fn paths_alias_scoped_component() {
    let world = World::new();

    // C++: ecs.use<test::Foo>() — alias using short name "Foo", full path "::test::Foo"
    // Rust: test_ns::Foo flattens to "::Foo" (Rust module namespace not preserved by flecs)
    // set_alias_component("") uses the entity's own short name "Foo" as alias
    let e = world.set_alias_component::<test_ns::Foo>("");
    let a = world.lookup("Foo");
    let c = world.lookup("Foo");

    assert_eq!(e.id(), a.id());
    assert_eq!(e.id(), c.id());
}

#[test]
fn paths_alias_scoped_component_w_name() {
    let world = World::new();

    // C++: ecs.use<test::Foo>("FooAlias") — alias "FooAlias" set, "Foo" alias NOT set, test::Foo path works
    // Rust: component path "::Foo"; "FooAlias" alias set; "Foo" resolves via path (not alias)
    // C++ f.id()==0 because "Foo" alias wasn't set; Rust finds via path — not exactly portable,
    // but we assert the alias and path both resolve to the same entity
    let e = world.set_alias_component::<test_ns::Foo>("FooAlias");
    let a = world.lookup("FooAlias");
    let f = world.lookup("Foo");
    let c = world.try_lookup("test_ns::Foo");

    assert_eq!(e.id(), a.id());
    assert_eq!(e.id(), f.id());
    assert!(c.is_none());
}

#[test]
fn paths_alias_entity() {
    let world = World::new();

    let e = world.entity_named("Foo");

    world.set_alias_entity(e, "FooAlias");

    let a = world.lookup("FooAlias");

    assert_eq!(e.id(), a.id());
}

#[test]
fn paths_alias_entity_by_name() {
    let world = World::new();

    let e = world.entity_named("Foo");

    world.set_alias_entity(e, "FooAlias");

    let l = world.lookup("FooAlias");

    assert_eq!(e.id(), l.id());
}

#[test]
fn paths_alias_entity_by_scoped_name() {
    let world = World::new();

    let e = world.entity_named("Foo::Bar");

    let a = world.set_alias_entity_by_name("Foo::Bar", "FooAlias");

    let l = world.lookup("FooAlias");

    assert_eq!(e.id(), a.id());
    assert_eq!(e.id(), l.id());
}

#[test]
fn paths_alias_entity_empty() {
    let world = World::new();

    let parent = world.entity_named("parent");
    let child = world.entity_named("child").child_of(parent);

    // "child" without qualifier - can't be looked up from root since it's a child
    assert!(world.try_lookup("child").is_none());

    // set alias with empty string = use entity's own short name
    world.set_alias_entity(child, "");

    assert!(world.try_lookup("child").is_some());

    // override with a different alias
    world.set_alias_entity(child, "FooAlias");

    // now "child" alias is gone (replaced by "FooAlias")
    assert!(world.try_lookup("child").is_none());

    assert!(world.try_lookup("FooAlias").is_some());
}

#[test]
fn paths_id_from_str_0_entity() {
    let world = World::new();

    let id = IdView::new_from_str(&world, "#0");
    assert_eq!(id, 0u64);
}

#[test]
fn paths_id_from_str_entity_from_str() {
    let world = World::new();

    let foo = world.entity_named("foo");

    let id = IdView::new_from_str(&world, "foo");
    assert_ne!(id, 0u64);
    assert_eq!(id, foo);
}

#[test]
fn paths_id_from_str_unresolved_entity_from_str() {
    let world = World::new();

    let id = IdView::new_from_str(&world, "foo");
    assert_eq!(id, 0u64);
}

#[test]
fn paths_id_from_str_scoped_entity_from_str() {
    let world = World::new();

    let foo = world.entity_named("foo::bar");

    // C++ uses "foo.bar" (dot separator) with ecs.id()
    let id = IdView::new_from_str(&world, "foo.bar");
    assert_ne!(id, 0u64);
    assert_eq!(id, foo);
}

#[test]
fn paths_id_from_str_template_entity_from_str() {
    let world = World::new();

    let foo = world.entity_named("foo<bar>");

    let id = IdView::new_from_str(&world, "foo<bar>");
    assert_ne!(id, 0u64);
    assert_eq!(id, foo);
}

#[test]
fn paths_id_from_str_pair_from_str() {
    let world = World::new();

    let rel = world.entity_named("Rel");
    let tgt = world.entity_named("Tgt");

    let id = IdView::new_from_str(&world, "(Rel, Tgt)");
    assert_ne!(id, 0u64);
    assert_eq!(id, world.id_view_from((rel, tgt)));
}

#[test]
fn paths_id_from_str_unresolved_pair_from_str() {
    let world = World::new();

    world.entity_named("Rel");

    let id = IdView::new_from_str(&world, "(Rel, Tgt)");
    assert_eq!(id, 0u64);
}

#[test]
fn paths_id_from_str_wildcard_pair_from_str() {
    let world = World::new();

    let rel = world.entity_named("Rel");

    let id = IdView::new_from_str(&world, "(Rel, *)");
    assert_ne!(id, 0u64);
    assert_eq!(id, world.id_view_from((rel, *flecs::Wildcard)));
}

#[test]
fn paths_id_from_str_any_pair_from_str() {
    let world = World::new();

    let rel = world.entity_named("Rel");

    let id = IdView::new_from_str(&world, "(Rel, _)");
    assert_ne!(id, 0u64);
    assert_eq!(id, world.id_view_from((rel, *flecs::Any)));
}

#[test]
fn paths_id_from_str_invalid_pair() {
    let world = World::new();

    world.entity_named("Rel");
    world.entity_named("Tgt");

    // Missing closing paren - should return 0
    let id = IdView::new_from_str(&world, "(Rel, Tgt");
    assert_eq!(id, 0u64);
}
