use super::c_binding::bindings::ecs_init;
use super::c_types::WorldT;

pub struct World {
    pub world: *mut WorldT,
}

impl World {
    pub fn new() -> Self {
        Self {
            world: unsafe { ecs_init() },
        }
    }

    pub fn new_from_world(world: *mut WorldT) -> Self {
        Self { world }
    }
}
