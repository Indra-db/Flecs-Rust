#![doc(hidden)]

use core::ffi::c_char;

use crate::core::*;
use crate::sys;

pub(crate) fn create_component_desc(
    entity: sys::ecs_entity_t,
    type_info: flecs_ecs_sys::ecs_type_info_t,
) -> flecs_ecs_sys::ecs_component_desc_t {
    flecs_ecs_sys::ecs_component_desc_t {
        _canary: 0,
        entity,
        type_: type_info,
    }
}

pub(crate) fn create_type_info<T, const ALLOCATE_TAG: bool>() -> flecs_ecs_sys::ecs_type_info_t
where
    T: ComponentId,
{
    let size = {
        let size = core::mem::size_of::<T>();
        if ALLOCATE_TAG && size == 0 { 1 } else { size }
    };

    let alignment = if size != 0 {
        core::mem::align_of::<T>()
    } else {
        0
    };

    let mut hooks = Default::default();
    if size != 0 && T::NEEDS_DROP {
        // Register lifecycle callbacks, but only if the component has a
        // size and requires initialization of heap memory / needs drop.
        // Components that don't have a size are tags, and tags don't
        // require construction/destruction/copy/move's.
        T::__register_lifecycle_hooks(&mut hooks);
    }
    // we only want to implement it if it implements the default trait
    // in the case it does not, flecs registers a ctor hook that zeros out the memory.
    // on rust side we check for safety and panic if the component does not implement the default trait where needed.
    if T::IMPLS_DEFAULT {
        T::__register_default_hooks(&mut hooks);
    }

    T::__register_clone_hooks(&mut hooks);

    // if (!T::IMPLS_DEFAULT && !T::IS_ENUM) || !T::IMPLS_CLONE {
    //     let mut registered_hooks = RegistersPanicHooks::default();
    //     if !T::IMPLS_DEFAULT {
    //         registered_hooks.ctor = true;
    //     }
    //     if !T::IMPLS_CLONE {
    //         registered_hooks.copy = true;
    //     }
    //     let box_registers_panic_hooks = Box::<RegistersPanicHooks>::new(registered_hooks);
    //     let box_registers_panic_hooks_ptr = Box::into_raw(box_registers_panic_hooks);
    //     // we registered a panic hook
    //     hooks.binding_ctx = box_registers_panic_hooks_ptr as *mut core::ffi::c_void;
    //     hooks.binding_ctx_free = Some(register_panic_hooks_free_ctx);
    // }

    let type_info: flecs_ecs_sys::ecs_type_info_t = flecs_ecs_sys::ecs_type_info_t {
        size: size as i32,
        alignment: alignment as i32,
        hooks,
        component: 0,
        name: core::ptr::null(),
    };
    type_info
}

pub(crate) fn create_entity_desc(
    name: *const c_char,
    symbol: *const c_char,
) -> flecs_ecs_sys::ecs_entity_desc_t {
    let entity_desc: flecs_ecs_sys::ecs_entity_desc_t = flecs_ecs_sys::ecs_entity_desc_t {
        _canary: 0,
        id: 0,
        parent: 0,
        name,
        sep: SEPARATOR.as_ptr(),
        root_sep: core::ptr::null(),
        symbol,
        use_low_id: true,
        add: core::ptr::null(),
        add_expr: core::ptr::null(),
        set: core::ptr::null(),
    };
    entity_desc
}

pub(crate) fn external_create_type_info<T>() -> flecs_ecs_sys::ecs_type_info_t {
    let size = core::mem::size_of::<T>();
    let alignment = if size != 0 {
        core::mem::align_of::<T>()
    } else {
        0
    };
    let mut hooks = Default::default();
    if size != 0 && const { core::mem::needs_drop::<T>() } {
        // Register lifecycle callbacks, but only if the component has a
        // size and requires initialization of heap memory / needs drop.
        // Components that don't have a size are tags, and tags don't
        // require construction/destruction/copy/move's.
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<T>(&mut hooks);
    }

    let type_info: flecs_ecs_sys::ecs_type_info_t = flecs_ecs_sys::ecs_type_info_t {
        size: size as i32,
        alignment: alignment as i32,
        hooks,
        component: 0,
        name: core::ptr::null(),
    };
    type_info
}
