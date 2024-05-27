use std::ffi::{c_char, CStr};

use crate::core::*;
use crate::sys;

/// attempts to register the component with the world. If it's already registered, it does nothing.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub(crate) fn try_register_component_impl<'a, T>(
    world: impl IntoWorld<'a>,
    name: *const c_char,
) -> EntityT
where
    T: ComponentId,
{
    #[cfg(not(feature = "flecs_multi_world_application"))]
    {
        register_component_single_world_application::<T>(world, name)
    }

    #[cfg(feature = "flecs_multi_world_application")]
    {
        register_component_multi_world_application::<T>(world, name)
    }
}

#[doc(hidden)]
pub fn register_component_single_world_application<'a, T>(
    world: impl IntoWorld<'a>,
    name: *const i8,
) -> u64
where
    T: ComponentId,
{
    let world_ptr = world.world_ptr_mut();

    if !T::is_registered() {
        register_component_data::<T>(world_ptr, name, false, false);

        if T::IS_ENUM {
            register_enum_data::<T>(world_ptr);
        }
    }

    // SAFETY: either it was registered just now or it was already registered
    unsafe { T::get_id_unchecked() }
}

#[doc(hidden)]
pub fn register_component_multi_world_application<'a, T>(
    world: impl IntoWorld<'a>,
    name: *const i8,
) -> u64
where
    T: ComponentId,
{
    let world_ptr = world.world_ptr_mut();
    let is_registered = T::is_registered();
    let is_registered_with_world = if is_registered {
        unsafe { is_component_registered_with_world::<T>(world_ptr) }
    } else {
        false
    };

    if !is_registered || !is_registered_with_world {
        let has_newly_registered =
            register_component_data::<T>(world_ptr, name, is_registered, is_registered_with_world);

        if T::IS_ENUM && has_newly_registered && !is_registered_with_world {
            register_enum_data::<T>(world_ptr);
        }
    }

    // SAFETY: either it was registered just now or it was already registered
    unsafe { T::get_id_unchecked() }
}

#[inline(always)]
/// attempts to register the component with the world. If it's already registered, it does nothing.
pub fn try_register_component<'a, T>(world: impl IntoWorld<'a>)
where
    T: ComponentId,
{
    try_register_component_impl::<T>(world, std::ptr::null());
}

#[inline(always)]
pub fn try_register_component_named<'a, T>(world: impl IntoWorld<'a>, name: &str) -> EntityT
where
    T: ComponentId,
{
    let name = compact_str::format_compact!("{}\0", name);

    try_register_component_impl::<T>(world, name.as_ptr() as *const c_char)
}

/// registers enum fields with the world.
pub(crate) fn register_enum_data<T>(world: *mut WorldT)
where
    T: ComponentId,
{
    //TODO we should convert this ecs_cpp functions to rust so if it ever changes, our solution won't break
    unsafe { sys::ecs_cpp_enum_init(world, T::get_id_unchecked()) };
    let enum_array_ptr = T::UnderlyingEnumType::__enum_data_mut();

    for (index, enum_item) in T::UnderlyingEnumType::iter().enumerate() {
        let name = enum_item.name_cstr();
        let entity_id: EntityT = unsafe {
            sys::ecs_cpp_enum_constant_register(
                world,
                T::get_id_unchecked(),
                T::UnderlyingEnumType::get_id_variant_of_index_unchecked(enum_item.enum_index()),
                name.as_ptr(),
                index as i32,
            )
        };
        if !T::UnderlyingEnumType::is_index_registered_as_entity(index) {
            unsafe { *enum_array_ptr.add(index) = entity_id };
        }
    }
}

/// registers the component with the world.
pub(crate) fn register_component_data<T>(
    world: *mut WorldT,
    name: *const c_char,
    is_comp_pre_registered: bool,
    is_comp_pre_registered_with_world: bool,
) -> bool
where
    T: ComponentId,
{
    // If the component is not registered with the world (indicating the
    // component has not yet been registered, or the component is used
    // across more than one binary), or if the id does not exists in the
    // world (indicating a multi-world application), register it.
    if !is_comp_pre_registered || !is_comp_pre_registered_with_world {
        let mut prev_scope: EntityT = 0;
        let mut prev_with: EntityT = 0;

        if !world.is_null() {
            prev_scope = unsafe { sys::ecs_set_scope(world, 0) };
            prev_with = unsafe { sys::ecs_set_with(world, 0) };
        }

        let id = if is_comp_pre_registered {
            // we know this is safe because we checked if the component is pre-registered
            unsafe { T::get_id_unchecked() }
        } else {
            0
        };

        register_componment_data_explicit::<T>(
            world,
            name,
            id,
            is_comp_pre_registered,
            is_comp_pre_registered_with_world,
        );

        if prev_with != 0 {
            unsafe { sys::ecs_set_with(world, prev_with) };
        }
        if prev_scope != 0 {
            unsafe { sys::ecs_set_scope(world, prev_scope) };
        }

        return true;
    }

    false
}

/// registers the component with the world.
pub(crate) fn register_componment_data_explicit<T>(
    world: *mut WorldT,
    name: *const c_char,
    id: EntityT,
    is_comp_pre_registered: bool,
    is_comp_pre_registered_with_world: bool,
) -> EntityT
where
    T: ComponentId,
{
    ecs_assert!(
        if id == 0 {
            !world.is_null()
        } else if is_comp_pre_registered {
             ecs_assert!(unsafe { T::get_id_unchecked() == id }, FlecsErrorCode::InconsistentComponentId);
             true
        } else {
            true
        },
        FlecsErrorCode::ComponentNotRegistered,
        name: *const c_char
    );

    let type_name = crate::core::type_name_cstring::<T>();
    let type_name_ptr = type_name.as_ptr();

    let symbol = get_symbol_name(id, world, type_name_ptr, is_comp_pre_registered_with_world);

    let name = if name.is_null() { type_name_ptr } else { name };

    let entity_desc = create_entity_desc(name, symbol, id);

    let entity = unsafe { flecs_ecs_sys::ecs_entity_init(world, &entity_desc) };

    let type_info = create_type_info::<T>();

    let component_desc = create_component_desc(entity, type_info);

    let entity = unsafe { flecs_ecs_sys::ecs_component_init(world, &component_desc) };

    ecs_assert!(
        if !is_comp_pre_registered {
            entity != 0 && unsafe { sys::ecs_exists(world, entity) }
        } else {
            true
        },
        FlecsErrorCode::InternalError
    );

    if !is_comp_pre_registered {
        T::__initialize(|| IdComponent { id: entity });
    }

    entity
}
