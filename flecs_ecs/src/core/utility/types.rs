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

// these types are not yet supported in the current version of the library.
// these types are not yet supported in the current version of the library.
// these types are not yet supported in the current version of the library.
// these types are not yet supported in the current version of the library.

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
