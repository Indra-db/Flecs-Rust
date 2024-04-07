use std::{ops::Deref, os::raw::c_void};

use crate::core::{Entity, IdT, World};

pub type FTime = f32;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
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
    pub fn to_entity(&self, world: &World) -> Entity {
        Entity::new_from_existing_raw(world.raw_world, self.0)
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
    pub(crate) each: Option<*mut c_void>,
    pub(crate) each_entity: Option<*mut c_void>,
    pub(crate) each_iter: Option<*mut c_void>,
    pub(crate) iter: Option<*mut c_void>,
    pub(crate) iter_only: Option<*mut c_void>,
    pub(crate) free_each: Option<EcsCtxFreeT>,
    pub(crate) free_each_entity: Option<EcsCtxFreeT>,
    pub(crate) free_each_iter: Option<EcsCtxFreeT>,
    pub(crate) free_iter: Option<EcsCtxFreeT>,
    pub(crate) free_iter_only: Option<EcsCtxFreeT>,
}

impl Drop for ObserverSystemBindingCtx {
    fn drop(&mut self) {
        if let Some(each) = self.each {
            if let Some(free_each) = self.free_each {
                free_each(each);
            }
        }
        if let Some(each_entity) = self.each_entity {
            if let Some(free_each_entity) = self.free_each_entity {
                free_each_entity(each_entity);
            }
        }
        if let Some(iter) = self.iter {
            if let Some(free_iter) = self.free_iter {
                free_iter(iter);
            }
        }
        if let Some(iter_only) = self.iter_only {
            if let Some(free_iter_only) = self.free_iter_only {
                free_iter_only(iter_only);
            }
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ObserverSystemBindingCtx {
    fn default() -> Self {
        Self {
            each: None,
            each_entity: None,
            each_iter: None,
            iter: None,
            iter_only: None,
            free_each: None,
            free_each_entity: None,
            free_each_iter: None,
            free_iter: None,
            free_iter_only: None,
        }
    }
}
impl ObserverSystemBindingCtx {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        each: Option<*mut c_void>,
        each_entity: Option<*mut c_void>,
        each_iter: Option<*mut c_void>,
        iter: Option<*mut c_void>,
        iter_only: Option<*mut c_void>,
        free_each: Option<EcsCtxFreeT>,
        free_each_entity: Option<EcsCtxFreeT>,
        free_each_iter: Option<EcsCtxFreeT>,
        free_iter: Option<EcsCtxFreeT>,
        free_iter_only: Option<EcsCtxFreeT>,
    ) -> Self {
        Self {
            each,
            each_entity,
            each_iter,
            iter,
            iter_only,
            free_each,
            free_each_entity,
            free_each_iter,
            free_iter,
            free_iter_only,
        }
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

pub struct ImplementsClone<T>(std::marker::PhantomData<T>);
pub struct ImplementsDefault<T>(std::marker::PhantomData<T>);
