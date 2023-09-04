use super::c_types::*;

pub struct Entity {
    pub world: *mut WorldT,
    pub id: EntityT,
}

impl Entity {
    pub const fn new(world: *mut WorldT, id: EntityT) -> Self {
        Self { world, id }
    }

    pub const fn new_only_id(id: EntityT) -> Self {
        Self {
            world: std::ptr::null_mut(),
            id,
        }
    }
}
