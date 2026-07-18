//! A `Query` must not be `Send` even when its components are: iterating it
//! on another thread would race with unsynchronized world mutation on the
//! owning thread.

use flecs_ecs::prelude::*;

#[derive(Component)]
struct Position {
    x: f32,
}

fn main() {
    let world = World::new();
    world.entity().set(Position { x: 0.0 });

    let query = world.new_query::<&Position>();
    std::thread::spawn(move || {
        query.each(|pos| {
            let _ = pos.x;
        });
    });
}
