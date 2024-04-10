use std::ffi::c_char;

use crate::core::*;
use crate::sys;

pub(crate) fn create_component_desc(
    entity: EntityT,
    type_info: flecs_ecs_sys::ecs_type_info_t,
) -> flecs_ecs_sys::ecs_component_desc_t {
    flecs_ecs_sys::ecs_component_desc_t {
        _canary: 0,
        entity,
        type_: type_info,
    }
}

pub(crate) fn create_type_info<T>() -> flecs_ecs_sys::ecs_type_info_t
where
    T: ComponentId,
{
    let size = std::mem::size_of::<T>();
    let alignment = if size != 0 {
        std::mem::align_of::<T>()
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

    let type_info: flecs_ecs_sys::ecs_type_info_t = flecs_ecs_sys::ecs_type_info_t {
        size: size as i32,
        alignment: alignment as i32,
        hooks,
        component: 0,
        name: std::ptr::null(),
    };
    type_info
}

pub(crate) fn create_entity_desc(
    name: *const c_char,
    symbol: *const c_char,
    id: EntityT,
) -> flecs_ecs_sys::ecs_entity_desc_t {
    let entity_desc: flecs_ecs_sys::ecs_entity_desc_t = flecs_ecs_sys::ecs_entity_desc_t {
        _canary: 0,
        id,
        name,
        sep: SEPARATOR.as_ptr(),
        root_sep: std::ptr::null(),
        symbol,
        use_low_id: true,
        add: [0; 32],
        add_expr: std::ptr::null(),
    };
    entity_desc
}

pub(crate) fn get_symbol_name(
    id: IdT,
    world: *mut sys::ecs_world_t,
    type_name_ptr: *const c_char,
    is_comp_pre_registered_with_world: bool,
) -> *const i8 {
    if id != 0 {
        let symbol_ptr = if is_comp_pre_registered_with_world {
            unsafe { sys::ecs_get_symbol(world, id) }
        } else {
            std::ptr::null()
        };
        if symbol_ptr.is_null() {
            type_name_ptr
        } else {
            symbol_ptr
        }
    } else {
        type_name_ptr
    }
}

/// checks if the component is registered with a world.
/// this function is unsafe because it assumes that the component is registered with a world, not necessarily the world passed in.
pub(crate) unsafe fn is_component_registered_with_world<T>(world: *const WorldT) -> bool
where
    T: ComponentId,
{
    // we know this is safe because we checked if world is not null & if the component is registered
    if !world.is_null() && unsafe { !sys::ecs_exists(world, T::get_id_unchecked()) } {
        return false;
    }

    true
}
