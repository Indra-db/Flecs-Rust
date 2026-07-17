//! A `QueryHandle` over a `!Send` component tuple must not be `Send`:
//! sending it to another thread would allow materializing component
//! references there via a stage.

use std::rc::Rc;

use flecs_ecs::prelude::*;

#[derive(Component)]
struct NonSendHandle {
    value: Rc<i32>,
}

fn main() {
    let world = World::new();
    let handle = world.new_query::<&NonSendHandle>().handle();
    std::thread::spawn(move || {
        drop(handle);
    });
}
