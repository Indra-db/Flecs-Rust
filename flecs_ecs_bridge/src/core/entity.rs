use std::{ops::Deref, os::raw::c_void};

use super::{
    c_binding::bindings::{ecs_get_world, ecs_new_w_id},
    c_types::{EntityT, IdT, WorldT},
    entity_view::EntityView,
    id::Id,
};

pub struct Entity {
    entity_view: EntityView,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            entity_view: EntityView::default(),
        }
    }
}

impl Deref for Entity {
    type Target = EntityView;

    fn deref(&self) -> &Self::Target {
        &self.entity_view
    }
}

impl Entity {
    /// Create new entity.
    /// ### Safety
    /// This function is unsafe because it assumes that the world is not null.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(world: *mut WorldT) -> Self {
        Self {
            entity_view: EntityView::new_from_existing(world, unsafe { ecs_new_w_id(world, 0) }),
        }
    }

    /// Wrap an existing entity id.
    /// # Arguments
    /// * `world` - The world the entity belongs to.
    /// * `id` - The entity id.
    pub fn new_from_existing(world: *mut WorldT, id: IdT) -> Self {
        Self {
            entity_view: EntityView::new_from_existing(world, id),
        }
    }

    // Explicit conversion from flecs::entity_t to Entity
    pub const fn new_only_id(id: EntityT) -> Self {
        Self {
            entity_view: EntityView::new_only_id(id),
        }
    }
}
