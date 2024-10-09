#![doc(hidden)]
use std::os::raw::c_void;

pub type FTime = f32;

pub(crate) type EcsCtxFreeT = extern "C-unwind" fn(*mut c_void);

// #[doc(hidden)]
// pub struct ReactorBindingType {
//     pub(crate) callback: Option<*mut c_void>,
//     pub(crate) free_callback: Option<EcsCtxFreeT>,
// }

// impl Drop for ReactorBindingType {
//     fn drop(&mut self) {
//         if let Some(callback) = self.callback {
//             if let Some(free_callback) = self.free_callback {
//                 free_callback(callback);
//             }
//         }
//     }
// }

// impl Default for ReactorBindingType {
//     fn default() -> Self {
//         Self {
//             callback: None,
//             free_callback: None,
//         }
//     }
// }

// impl ReactorBindingType {
//     pub(crate) fn new(callback: Option<*mut c_void>, free_callback: Option<EcsCtxFreeT>) -> Self {
//         Self {
//             callback,
//             free_callback,
//         }
//     }
// }
// pub(crate) enum TypeBinding {
//     Each(ReactorBindingType),
//     EachEntity(ReactorBindingType),
//     EachIter(ReactorBindingType),
//     Run(ReactorBindingType),
//     RunIter(ReactorBindingType),
//     RunEach(ReactorBindingType),
//     RunEachEntity(ReactorBindingType),
// }

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
        if std::thread::panicking() {
            return;
        }

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
