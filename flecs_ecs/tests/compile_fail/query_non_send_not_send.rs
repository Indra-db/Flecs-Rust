//! A `Query` over a `!Send` component must not be `Send`: sending it to
//! another thread would allow materializing component references there.

use std::rc::Rc;

use flecs_ecs::prelude::*;

#[derive(Component)]
struct NonSendHandle {
    value: Rc<i32>,
}

fn main() {
    let world = World::new();
    let query = world.new_query::<&NonSendHandle>();
    std::thread::spawn(move || {
        query.each(|handle| {
            let _ = &handle.value;
        });
    });
}
