use super::c_binding::bindings::ecs_init;
use super::c_types::WorldT;
use super::component::CachedComponentData;
use super::id::Id;

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

    /// Get id from a type.
    fn get_id<T: CachedComponentData>(&self) -> Id {
        Id::new(self.world, T::get_id())
    }

    /// get pair id from relationship, object.
    fn get_id_pair<T: CachedComponentData, U: CachedComponentData>(&self) -> Id {
        Id::new_world_pair(self.world, T::get_id(), U::get_id())
    }
}
