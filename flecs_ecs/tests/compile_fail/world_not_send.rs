//! `World` must not be `Send`: a clone shares unsynchronized state
//! (`WorldCtx`) with the original and may hold `!Send` components.

use flecs_ecs::prelude::*;

fn main() {
    let world = World::new();
    let world_clone = world.clone();
    std::thread::spawn(move || {
        world_clone.entity();
    });
}
