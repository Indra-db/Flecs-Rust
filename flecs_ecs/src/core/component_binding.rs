use std::ffi::c_void;

type EcsCtxFreeT = unsafe extern "C" fn(*mut c_void);

pub(crate) struct ComponentBindingCtx {
    pub(crate) on_add: Option<*mut c_void>,
    pub(crate) on_remove: Option<*mut c_void>,
    pub(crate) on_set: Option<*mut c_void>,
    pub(crate) free_on_add: Option<EcsCtxFreeT>,
    pub(crate) free_on_remove: Option<EcsCtxFreeT>,
    pub(crate) free_on_set: Option<EcsCtxFreeT>,
}

impl Drop for ComponentBindingCtx {
    fn drop(&mut self) {
        if let Some(on_add) = self.on_add {
            if let Some(free_on_add) = self.free_on_add {
                unsafe { free_on_add(on_add) };
            }
        }
        if let Some(on_remove) = self.on_remove {
            if let Some(free_on_remove) = self.free_on_remove {
                unsafe { free_on_remove(on_remove) };
            }
        }
        if let Some(on_set) = self.on_set {
            if let Some(free_on_set) = self.free_on_set {
                unsafe { free_on_set(on_set) };
            }
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ComponentBindingCtx {
    fn default() -> Self {
        Self {
            on_add: None,
            on_remove: None,
            on_set: None,
            free_on_add: None,
            free_on_remove: None,
            free_on_set: None,
        }
    }
}
impl ComponentBindingCtx {
    pub(crate) fn new(
        on_add: Option<*mut c_void>,
        on_remove: Option<*mut c_void>,
        on_set: Option<*mut c_void>,
        free_on_add: Option<EcsCtxFreeT>,
        free_on_remove: Option<EcsCtxFreeT>,
        free_on_set: Option<EcsCtxFreeT>,
    ) -> Self {
        Self {
            on_add,
            on_remove,
            on_set,
            free_on_add,
            free_on_remove,
            free_on_set,
        }
    }
}
