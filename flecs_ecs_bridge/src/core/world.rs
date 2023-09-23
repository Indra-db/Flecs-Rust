use super::c_binding::bindings::{ecs_init, ecs_stage_is_readonly};
use super::c_types::{EntityT, WorldT};
use super::component::CachedComponentData;
use super::id::Id;

pub struct World {
    pub world: *mut WorldT,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
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
        Id::new_from_existing(self.world, T::get_id(self.world))
    }

    /// get pair id from relationship, object.
    fn get_id_pair<T: CachedComponentData, U: CachedComponentData>(&self) -> Id {
        Id::new_world_pair(self.world, T::get_id(self.world), U::get_id(self.world))
    }

    /// get pair id from relationship, object.
    fn get_id_pair_with_id<T: CachedComponentData>(&self, id: EntityT) -> Id {
        Id::new_world_pair(self.world, T::get_id(self.world), id)
    }

    /// get pair id from relationship, object.
    fn get_id_pair_from_ids(&self, id: EntityT, id2: EntityT) -> Id {
        Id::new_world_pair(self.world, id, id2)
    }

    //TODO DOCUMENT
    pub fn is_readonly(&self) -> bool {
        unsafe { ecs_stage_is_readonly(self.world) }
    }
}
