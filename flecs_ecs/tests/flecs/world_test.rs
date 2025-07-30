#![allow(dead_code)]
use flecs_ecs::prelude::*;
use flecs_ecs::sys;

#[test]
fn world_no_panic_clone_test() {
    let world = World::default();
    let world2 = world.clone();
    let _query = world.new_query::<()>();
    core::mem::drop(world);
    let _query2 = world2.new_query::<()>();
}

#[test]
#[should_panic]
fn world_reset_panic_lingering_world_refs() {
    let world = World::default();
    let _world2 = world.clone();
    world.reset();
}

#[test]
#[should_panic]
fn world_panic_lingering_query_handles() {
    let world = World::default();
    let _query = world.new_query::<()>();
    core::mem::drop(world);
}

#[test]
#[ignore = "this code is not applicable to rust as it is to cpp, since we deal with copying the world differently"]
fn world_finis_reentrancy() {
    #[derive(Debug, Clone, Copy, Component, Default)]
    struct A {
        a: i32,
    }

    let world = World::default();

    // declare on remove hook for component A:
    world.component::<A>().on_remove(|e, _| {
        let world = e.world();
        // This code runs on world destroy, since we did not remove this component manually before the world was destroyed.

        // before we make a copy of the world, the refcount has to be 1 since this is the special case where
        // we will be copying a world object precisely when the world is being destroyed.
        // see world::~world() code and notes.
        let hdr = world.ptr_mut() as *mut sys::ecs_header_t;
        unsafe {
            assert_eq!((*hdr).refcount, 1);
        };

        // obtain the entity's world. This increments the world's hdr refcount
        let hdr = e.world().ptr_mut() as *mut sys::ecs_header_t;

        unsafe {
            assert_eq!((*hdr).refcount, 2);
        }

        // here world_copy object wrapping c world is destroyed
        // therefore, world destroy will be called again wreaking havoc.
    });

    world.entity().add(A::id());

    // world will be destroyed here, and hook above will be called.
}
