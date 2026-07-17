#![doc(hidden)]
use core::ffi::c_void;

#[cfg(target_family = "wasm")]
type EcsCtxFreeT = unsafe extern "C" fn(*mut c_void);
#[cfg(not(target_family = "wasm"))]
type EcsCtxFreeT = unsafe extern "C-unwind" fn(*mut c_void);

pub(crate) struct ComponentBindingCtx {
    pub(crate) on_add: Option<*mut c_void>,
    pub(crate) on_remove: Option<*mut c_void>,
    pub(crate) on_set: Option<*mut c_void>,
    pub(crate) on_replace: Option<*mut c_void>,
    pub(crate) free_on_add: Option<EcsCtxFreeT>,
    pub(crate) free_on_remove: Option<EcsCtxFreeT>,
    pub(crate) free_on_set: Option<EcsCtxFreeT>,
    pub(crate) free_on_replace: Option<EcsCtxFreeT>,
    // fn pointers need no heap allocation, so no free_on_compare/free_on_equals
    pub(crate) on_compare: Option<*mut c_void>,
    pub(crate) on_equals: Option<*mut c_void>,
}

impl Drop for ComponentBindingCtx {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }

        if let Some(on_add) = self.on_add
            && let Some(free_on_add) = self.free_on_add
        {
            // SAFETY: on_add was allocated by the matching Box/leak that
            // produced free_on_add, and Drop runs at most once so this
            // pointer is freed exactly once here.
            unsafe { free_on_add(on_add) };
        }
        if let Some(on_remove) = self.on_remove
            && let Some(free_on_remove) = self.free_on_remove
        {
            // SAFETY: on_remove was allocated by the matching Box/leak that
            // produced free_on_remove, and Drop runs at most once so this
            // pointer is freed exactly once here.
            unsafe { free_on_remove(on_remove) };
        }
        if let Some(on_set) = self.on_set
            && let Some(free_on_set) = self.free_on_set
        {
            // SAFETY: on_set was allocated by the matching Box/leak that
            // produced free_on_set, and Drop runs at most once so this
            // pointer is freed exactly once here.
            unsafe { free_on_set(on_set) };
        }
        if let Some(on_replace) = self.on_replace
            && let Some(free_on_replace) = self.free_on_replace
        {
            // SAFETY: on_replace was allocated by the matching Box/leak that
            // produced free_on_replace, and Drop runs at most once so this
            // pointer is freed exactly once here.
            unsafe { free_on_replace(on_replace) };
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
            on_replace: None,
            free_on_add: None,
            free_on_remove: None,
            free_on_set: None,
            free_on_replace: None,
            on_compare: None,
            on_equals: None,
        }
    }
}
