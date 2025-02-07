//! This test needs to be a separate process, since the OS API is process-global.

use ecs_os_api::try_add_init_hook;
use flecs_ecs::prelude::*;

#[test]
fn hooks() {
    use core::sync::atomic::{AtomicU32, Ordering};
    use flecs_ecs::prelude::*;

    let n = Box::leak(Box::new(AtomicU32::new(0)));

    try_add_init_hook(Box::new(|_| {
        n.fetch_add(1, Ordering::SeqCst);
    }))
    .unwrap();

    // Hooks do not run until the first World is created
    assert_eq!(n.load(Ordering::SeqCst), 0);

    let _w = World::new();

    // Hooks should have run now
    assert_eq!(n.load(Ordering::SeqCst), 1);

    let _w2 = World::new();

    // Hooks should only run once
    assert_eq!(n.load(Ordering::SeqCst), 1);

    // Late hooks should fail
    try_add_init_hook(Box::new(|_| {
        n.fetch_add(2, Ordering::SeqCst);
    }))
    .unwrap_err();

    // Late hooks should have no effect
    assert_eq!(n.load(Ordering::SeqCst), 1);
}
