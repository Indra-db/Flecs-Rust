#![doc(hidden)]
use core::ffi::c_char;
use core::ffi::c_void;

use crate::core::*;
use crate::sys;

#[doc(hidden)]
pub fn internal_register_component<
    'a,
    const IS_NAMED: bool,
    const COMPONENT_REGISTRATION: bool,
    T,
>(
    world: impl WorldProvider<'a>,
    name: *const c_char,
) -> u64
where
    T: ComponentId,
{
    let world = world.world();
    let world_ptr = world.world_ptr_mut();

    let id = if IS_NAMED {
        register_component_data_named::<COMPONENT_REGISTRATION, T>(world, name)
    } else {
        register_component_data::<COMPONENT_REGISTRATION, T>(world)
    };

    if T::IS_ENUM {
        let underlying_enum_type_id = world.component_id::<T::UnderlyingTypeOfEnum>();
        register_enum_data::<T>(world_ptr, id, *underlying_enum_type_id);
    }
    id
}

#[doc(hidden)]
pub(crate) fn external_register_component<'a, const COMPONENT_REGISTRATION: bool, T>(
    world: impl WorldProvider<'a>,
    name: *const c_char,
) -> u64 {
    external_register_component_data::<COMPONENT_REGISTRATION, T>(world.world_ptr_mut(), name)
}

#[inline(always)]
/// attempts to register the component with the world. If it's already registered, it does nothing.
pub(crate) fn try_register_component<'a, const COMPONENT_REGISTRATION: bool, T>(
    world: impl WorldProvider<'a>,
) -> sys::ecs_entity_t
where
    T: ComponentId,
{
    const NAMED: bool = false;
    internal_register_component::<NAMED, COMPONENT_REGISTRATION, T>(world, core::ptr::null())
}

#[inline(always)]
pub(crate) fn try_register_component_named<'a, const COMPONENT_REGISTRATION: bool, T>(
    world: impl WorldProvider<'a>,
    name: &str,
) -> sys::ecs_entity_t
where
    T: ComponentId,
{
    let name = compact_str::format_compact!("{}\0", name);
    const NAMED: bool = true;
    internal_register_component::<NAMED, COMPONENT_REGISTRATION, T>(
        world,
        name.as_ptr() as *const c_char,
    )
}

/// registers enum fields with the world.
pub(crate) fn register_enum_data<T>(
    world: *mut sys::ecs_world_t,
    id: sys::ecs_entity_t,
    underlying_type_id: sys::ecs_entity_t,
) where
    T: ComponentId,
{
    //TODO we should convert this ecs_cpp functions to rust so if it ever changes, our solution won't break
    unsafe { sys::ecs_cpp_enum_init(world, id, underlying_type_id) };
    let enum_array_ptr = T::UnderlyingEnumType::__enum_data_mut();

    for (mut index, enum_item) in T::UnderlyingEnumType::iter().enumerate() {
        let name = enum_item.name_cstr();
        let entity_id: sys::ecs_entity_t = unsafe {
            sys::ecs_cpp_enum_constant_register(
                world,
                id,
                T::UnderlyingEnumType::id_variant_of_index_unchecked(enum_item.enum_index()),
                name.as_ptr(),
                &mut index as *mut usize as *mut c_void,
                underlying_type_id,
                core::mem::size_of::<T::UnderlyingTypeOfEnum>(),
            )
        };
        if !T::UnderlyingEnumType::is_index_registered_as_entity(index) {
            unsafe { *enum_array_ptr.add(index) = entity_id };
        }
    }
}

/// registers the component with the world.
pub(crate) fn register_component_data_named<const COMPONENT_REGISTRATION: bool, T>(
    world: WorldRef<'_>,
    name: *const c_char,
) -> sys::ecs_entity_t
where
    T: ComponentId,
{
    let worldref = world;
    let world = worldref.world_ptr_mut();

    let prev_scope = if !COMPONENT_REGISTRATION && unsafe { sys::ecs_get_scope(world) != 0 } {
        unsafe { sys::ecs_set_scope(world, 0) }
    } else {
        0
    };
    let prev_with = if !COMPONENT_REGISTRATION {
        unsafe { sys::ecs_set_with(world, 0) }
    } else {
        0
    };

    let id = register_componment_data_explicit::<T, false>(worldref, name);

    if !COMPONENT_REGISTRATION {
        if prev_with != 0 {
            unsafe { sys::ecs_set_with(world, prev_with) };
        }
        if prev_scope != 0 {
            unsafe { sys::ecs_set_scope(world, prev_scope) };
        }
    }
    id
}

/// registers the component with the world.
pub(crate) fn register_component_data<const COMPONENT_REGISTRATION: bool, T>(
    world: WorldRef<'_>,
) -> sys::ecs_entity_t
where
    T: ComponentId,
{
    let worldref = world;
    let world = worldref.world_ptr_mut();
    let prev_scope = if !COMPONENT_REGISTRATION && unsafe { sys::ecs_get_scope(world) != 0 } {
        unsafe { sys::ecs_set_scope(world, 0) }
    } else {
        0
    };

    let prev_with = if !COMPONENT_REGISTRATION {
        unsafe { sys::ecs_set_with(world, 0) }
    } else {
        0
    };

    let id = register_componment_data_explicit::<T, false>(worldref, core::ptr::null());

    if !COMPONENT_REGISTRATION {
        if prev_with != 0 {
            unsafe { sys::ecs_set_with(world, prev_with) };
        }
        if prev_scope != 0 {
            unsafe { sys::ecs_set_scope(world, prev_scope) };
        }
    }

    id
}

pub(crate) fn external_register_component_data<const COMPONENT_REGISTRATION: bool, T>(
    world: *mut sys::ecs_world_t,
    name: *const c_char,
) -> sys::ecs_entity_t {
    let prev_scope = if !COMPONENT_REGISTRATION {
        unsafe { sys::ecs_set_scope(world, 0) }
    } else {
        0
    };

    let prev_with = if !COMPONENT_REGISTRATION {
        unsafe { sys::ecs_set_with(world, 0) }
    } else {
        0
    };

    let id = external_register_componment_data_explicit::<T>(world, name);

    if !COMPONENT_REGISTRATION {
        if prev_with != 0 {
            unsafe { sys::ecs_set_with(world, prev_with) };
        }

        if prev_scope != 0 {
            unsafe { sys::ecs_set_scope(world, prev_scope) };
        }
    }

    id
}

/// registers the component with the world.
pub(crate) fn register_componment_data_explicit<T, const ALLOCATE_TAG: bool>(
    world: WorldRef<'_>,
    name: *const c_char,
) -> sys::ecs_entity_t
where
    T: ComponentId,
{
    let worldref = world;
    let world = worldref.world_ptr_mut();

    let arr = worldref.components_array();
    let index = T::index() as usize;

    let c = if index < arr.len() { arr[index] } else { 0 };

    if c != 0 && unsafe { sys::ecs_is_alive(world, c) } {
        return c;
    }

    let only_type_name = crate::core::get_only_type_name::<T>();
    let only_type_name = compact_str::format_compact!("{}\0", only_type_name);

    let type_name = crate::core::type_name_cstring::<T>();
    let type_name_ptr = type_name.as_ptr();

    let mut user_name = name;
    let mut implicit_name = false;

    if user_name.is_null() {
        // If no name was provided, use the type name as the name.
        user_name = type_name_ptr;
        /* Keep track of whether name was explicitly set. If not, and
         * the component was already registered, just use the registered
         * name. The registered name may differ from the typename as the
         * registered name includes the flecs scope. This can in theory
         * be different from the C++ namespace though it is good
         * practice to keep them the same */
        implicit_name = true;
    }

    /* If component is registered by module, ensure it's registered in
     * the scope of the module. */
    let module = unsafe { sys::ecs_get_scope(world) };

    if module != 0 && implicit_name {
        user_name = only_type_name.as_ptr() as *const c_char;
    }

    //TODO should I do this?
    /* If the component is not yet registered, ensure no other component
     * or entity has been registered with this name. Ensure component is
     * looked up from root. */
    // let prev_scope = unsafe { sys::ecs_set_scope(world, 0) };
    // c = unsafe {
    //     sys::ecs_lookup_path_w_sep(
    //         world,
    //         0,
    //         user_name,
    //         SEPARATOR.as_ptr(),
    //         SEPARATOR.as_ptr(),
    //         false,
    //     )
    // };

    // TODO it feels like this should be investigated with the get only type name approach
    // TODO if it's valid. When two components with the same name, but different module get registered
    // this still gives id 0, so what's the point of this check?

    // If no name was provided first check if a type with the provided
    // symbol was already registered.
    let id = if name.is_null() {
        let prev_scope = unsafe { sys::ecs_set_scope(world, 0) };
        let id = unsafe { sys::ecs_lookup_symbol(world, type_name_ptr as *const _, false, false) };
        unsafe { sys::ecs_set_scope(world, prev_scope) };
        id
    } else {
        0
    };
    if id != 0 {
        return id;
    }

    let name = user_name;

    //TODO hack, otherwise importing will have mismatch symbol with the c components
    let entity_desc_name = if only_type_name.starts_with("Ecs") {
        only_type_name.as_ptr() as *const c_char
    } else {
        type_name_ptr
    };
    let entity_desc = create_entity_desc(name, entity_desc_name);

    let entity = unsafe { flecs_ecs_sys::ecs_entity_init(world, &entity_desc) };

    let type_info = create_type_info::<T, ALLOCATE_TAG>();

    let component_desc = create_component_desc(entity, type_info);

    let entity = unsafe { flecs_ecs_sys::ecs_component_init(world, &component_desc) };

    ecs_assert!(
        entity != 0 && unsafe { sys::ecs_exists(world, entity) },
        FlecsErrorCode::InternalError
    );

    entity
}

/// registers the component with the world.
pub(crate) fn external_register_componment_data_explicit<T>(
    world: *mut sys::ecs_world_t,
    name: *const c_char,
) -> sys::ecs_entity_t {
    let only_type_name = crate::core::get_only_type_name::<T>();
    let only_type_name = compact_str::format_compact!("{}\0", only_type_name);

    // If no name was provided first check if a type with the provided
    // symbol was already registered.
    let id = if name.is_null() {
        let prev_scope = unsafe { sys::ecs_set_scope(world, 0) };
        let id = unsafe {
            sys::ecs_lookup_symbol(world, only_type_name.as_ptr() as *const _, false, false)
        };
        unsafe { sys::ecs_set_scope(world, prev_scope) };
        id
    } else {
        0
    };
    if id != 0 {
        return id;
    }

    let type_name = crate::core::type_name_cstring::<T>();
    let type_name_ptr = type_name.as_ptr();

    let name = if name.is_null() { type_name_ptr } else { name };

    let entity_desc = create_entity_desc(name, type_name_ptr);

    let entity = unsafe { flecs_ecs_sys::ecs_entity_init(world, &entity_desc) };

    let type_info = external_create_type_info::<T>();

    let component_desc = create_component_desc(entity, type_info);

    let entity = unsafe { flecs_ecs_sys::ecs_component_init(world, &component_desc) };

    ecs_assert!(
        entity != 0 && unsafe { sys::ecs_exists(world, entity) },
        FlecsErrorCode::InternalError
    );

    entity
}
