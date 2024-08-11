use crate::core::{IntoId, WorldProvider};
use crate::sys;

use super::entity_id::EntityId;

pub(crate) trait PrivateMutEntityView: for<'a> WorldProvider<'a> + EntityId + Sized {
    unsafe fn add_id_unchecked(self, id: impl IntoId) -> Self {
        let id = *id.into();
        let world = self.world_ptr_mut();

        unsafe { sys::ecs_add_id(world, *self.entity_id(), id) }
        self
    }
}
