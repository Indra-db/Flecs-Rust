//! Flecs uses an "OS API" for interacting with the rest of the world,
//! including operations such as memory allocation and logging.
//!
//! This module provides a basic structure for hooking into the initialization
//! of that API, which allows, for example, customizing how Flecs sends log
//! messages.
#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use std::sync::LazyLock;
use std::sync::Mutex;

struct OsApiHook(Box<dyn FnOnce(&mut flecs_ecs::sys::ecs_os_api_t)>);

/// SAFETY: the OS API hooks are only ever used once, from behind a [`Mutex`]
unsafe impl Send for OsApiHook {}

/// List of hooks to run during initialization of the Flecs OS API from Rust.
///
/// Run automatically, once and only once, when the first [`super::World`]
/// is created, or [`ensure_initialized`] is called directly.
static OS_API_HOOKS: LazyLock<Mutex<Option<Vec<OsApiHook>>>> =
    LazyLock::new(|| Mutex::new(Some(Default::default())));

/// Initialize the Flecs OS API if not initialized already.
///
/// If the OS API has already been initialized (e.g. by C code)
/// hooks will still run but have no effect on the OS API state.
///
/// This function is called from [`super::World`] constructors.
///
/// See also: [`add_init_hook`]
pub fn ensure_initialized() {
    let Some(hooks) = OS_API_HOOKS
        .lock()
        .expect("Internal OS API hook list lock should not be poisoned")
        .take()
    else {
        // Already initialized
        return;
    };

    let mut api = unsafe {
        flecs_ecs::sys::ecs_os_set_api_defaults();
        flecs_ecs::sys::ecs_os_get_api()
    };
    for h in hooks {
        (h.0)(&mut api);
    }
    unsafe {
        flecs_ecs::sys::ecs_os_set_api(&mut api as *mut _);
    };
}

/// Add a hook for modifying the Flecs OS API structure,
/// which runs during [`ensure_initialized`].
///
/// See also: [`try_add_init_hook`]
///
/// # Panics
/// Will panic if the OS API has already been initialized,
/// at which point such hooks cannot have any effect.
///
/// Note that when a hook is executing, the initialization flag
/// has already been set so no more hooks can be added, even though
/// the OS API is not quite finished initializing.
///
/// # Example
/// ```no_run
/// # // Flagged as no_run since doctests will soon become single-process,
/// # // which will break this test, since OS API state is process-global.
/// use flecs_ecs::prelude::*;
///
/// ecs_os_api::add_init_hook(Box::new(|api| {
///     unsafe extern "C" fn abort_() {
///         panic!("fatal error in flecs");
///     }
///
///     api.abort_ = Some(abort_);
/// }));
/// ```
pub fn add_init_hook(f: Box<dyn FnOnce(&mut flecs_ecs::sys::ecs_os_api_t)>) {
    if let Err(e) = try_add_init_hook(f) {
        panic!("{e}");
    }
}

/// Errors returned by [`try_add_init_hook`]
#[derive(Debug, PartialEq, Eq)]
pub enum AddInitHookError {
    /// Internal Flecs OS API hook list lock was poisoned
    LockPoisoned,
    /// Flecs OS API has already been initialized, adding hooks will have no effect now
    AlreadyInitialized,
}

impl core::fmt::Display for AddInitHookError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AddInitHookError::LockPoisoned => {
                write!(f, "Internal Flecs OS API hook list lock was poisoned")
            }
            AddInitHookError::AlreadyInitialized => write!(
                f,
                "Flecs OS API has already been initialized, adding hooks will have no effect now"
            ),
        }
    }
}

impl core::error::Error for AddInitHookError {}

/// If the Flecs OS API has not already been initialized, add a hook
/// for modifying it, which runs during [`ensure_initialized`].
///
/// See also: [`add_init_hook`]
pub fn try_add_init_hook(
    f: Box<dyn FnOnce(&mut flecs_ecs::sys::ecs_os_api_t)>,
) -> Result<(), AddInitHookError> {
    OS_API_HOOKS
        .lock()
        .map_err(|_| AddInitHookError::LockPoisoned)
        .and_then(|mut h| {
            h.as_mut()
                .map(|h| h.push(OsApiHook(f)))
                .ok_or(AddInitHookError::AlreadyInitialized)
        })
}
