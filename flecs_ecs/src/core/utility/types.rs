use std::os::raw::c_void;

use crate::core::{
    c_types::{InOutKind, OperKind},
    component_registration::CachedComponentData,
};

use super::traits::{InOutType, OperType};

pub type FTime = f32;

pub(crate) type EcsCtxFreeT = extern "C" fn(*mut c_void);

pub(crate) struct ObserverSystemBindingCtx {
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

impl<T> InOutType for &mut T
where
    T: CachedComponentData,
{
    const IN_OUT: InOutKind = InOutKind::InOutDefault;
    type Type = T;
}

impl<T> InOutType for &T
where
    T: CachedComponentData,
{
    type Type = T;
    const IN_OUT: InOutKind = InOutKind::In;
}

impl<T> OperType for &mut T
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::And;
}

impl<T> OperType for &T
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::And;
}

impl<T> OperType for Option<&T>
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::Optional;
}

impl<T> OperType for Option<&mut T>
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::Optional;
}
