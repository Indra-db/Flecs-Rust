#![allow(dead_code)]
use flecs_ecs::core::World;

#[test]
fn world_no_panic_clone_test() {
    let world = World::default();
    let world2 = world.clone();
    let _query = world.new_query::<()>();
    std::mem::drop(world);
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
    std::mem::drop(world);
}
