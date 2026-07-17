//! A `QueryIter` (e.g. from `iter_stage`) holds raw world/iterator pointers
//! and must not cross threads, even scoped ones.

use flecs_ecs::prelude::*;

#[derive(Component)]
struct Position {
    x: f32,
}

fn main() {
    let world = World::new();
    world.entity().set(Position { x: 0.0 });

    let query = world.new_query::<&Position>();
    let iter = query.iterable();
    std::thread::scope(|s| {
        s.spawn(move || {
            drop(iter);
        });
    });
}
