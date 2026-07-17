//! A multithreaded system over a `!Send` component must not compile:
//! `par_each` hands component references to worker threads.

use std::rc::Rc;

use flecs_ecs::prelude::*;

#[derive(Component)]
struct NonSendHandle {
    value: Rc<i32>,
}

fn main() {
    let world = World::new();
    world
        .system::<&mut NonSendHandle>()
        .par_each(|handle| {
            let _ = &handle.value;
        });
}
