use std::{ops::Deref, os::raw::c_void};

use crate::core::{Entity, IdT, IntoWorld};

pub type FTime = f32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub IdT);

impl EntityId {
    #[inline]
    pub fn new(id: IdT) -> Self {
        Self(id)
    }

    /// Convert the entity id to an entity without a world.
    /// This entity is not safe to do operations on.
    ///
    /// # Safety
    ///
    /// This entity is not safe to do operations on as it has no valig world reference
    pub fn to_entity_no_world(&self) -> Entity {
        Entity::from(self.0)
    }

    /// Convert the entity id to an entity with the given world.
    ///
    /// # Safety
    ///
    /// This entity is safe to do operations on if the entity belongs to the world
    ///
    /// # Arguments
    ///
    /// * `world` - The world the entity belongs to
    pub fn to_entity<'a>(&self, world: impl IntoWorld<'a>) -> Entity<'a> {
        Entity::new_from_existing_raw(world.world_ref(), self.0)
    }
}

impl From<EntityId> for IdT {
    #[inline]
    fn from(id: EntityId) -> Self {
        id.0
    }
}

impl From<IdT> for EntityId {
    #[inline]
    fn from(id: IdT) -> Self {
        Self(id)
    }
}

impl Deref for EntityId {
    type Target = IdT;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) type EcsCtxFreeT = extern "C" fn(*mut c_void);

#[doc(hidden)]
pub struct ObserverSystemBindingCtx {
    pub(crate) func: Option<*mut c_void>,
    pub(crate) free_func: Option<EcsCtxFreeT>,
}

impl Drop for ObserverSystemBindingCtx {
    fn drop(&mut self) {
        if let Some(func) = self.func {
            if let Some(free_func) = self.free_func {
                free_func(func);
            }
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ObserverSystemBindingCtx {
    fn default() -> Self {
        Self {
            func: None,
            free_func: None,
        }
    }
}
impl ObserverSystemBindingCtx {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(func: Option<*mut c_void>, free_func: Option<EcsCtxFreeT>) -> Self {
        Self { func, free_func }
    }
}

pub(crate) struct ObserverEntityBindingCtx {
    pub(crate) empty: Option<*mut c_void>,
    pub(crate) empty_entity: Option<*mut c_void>,
    pub(crate) payload: Option<*mut c_void>,
    pub(crate) payload_entity: Option<*mut c_void>,
    pub(crate) free_empty: Option<EcsCtxFreeT>,
    pub(crate) free_empty_entity: Option<EcsCtxFreeT>,
    pub(crate) free_payload: Option<EcsCtxFreeT>,
    pub(crate) free_payload_entity: Option<EcsCtxFreeT>,
}

impl Drop for ObserverEntityBindingCtx {
    fn drop(&mut self) {
        if let Some(empty) = self.empty {
            if let Some(free_empty) = self.free_empty {
                free_empty(empty);
            }
        }
        if let Some(entity) = self.empty_entity {
            if let Some(free_entity) = self.free_empty_entity {
                free_entity(entity);
            }
        }
        if let Some(payload) = self.payload {
            if let Some(free_payload) = self.free_payload {
                free_payload(payload);
            }
        }
        if let Some(payload_entity) = self.payload_entity {
            if let Some(free_payload_entity) = self.free_payload_entity {
                free_payload_entity(payload_entity);
            }
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ObserverEntityBindingCtx {
    fn default() -> Self {
        Self {
            empty: None,
            empty_entity: None,
            payload: None,
            payload_entity: None,
            free_empty: None,
            free_empty_entity: None,
            free_payload: None,
            free_payload_entity: None,
        }
    }
}

impl ObserverEntityBindingCtx {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        empty: Option<*mut c_void>,
        empty_entity: Option<*mut c_void>,
        payload: Option<*mut c_void>,
        payload_entity: Option<*mut c_void>,
        free_empty: Option<EcsCtxFreeT>,
        free_empty_entity: Option<EcsCtxFreeT>,
        free_payload: Option<EcsCtxFreeT>,
        free_payload_entity: Option<EcsCtxFreeT>,
    ) -> Self {
        Self {
            empty,
            empty_entity,
            payload,
            payload_entity,
            free_empty,
            free_empty_entity,
            free_payload,
            free_payload_entity,
        }
    }
}
