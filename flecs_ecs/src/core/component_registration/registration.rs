#![doc(hidden)]
use core::ffi::c_char;
use core::ffi::c_void;

use crate::core::*;
use crate::sys;

struct ScopeWithGuard {
    world: *mut sys::ecs_world_t,
    prev_scope: Option<sys::ecs_entity_t>,
    prev_with: Option<sys::ecs_id_t>,
}

impl ScopeWithGuard {
    #[inline(always)]
    fn new(world: *mut sys::ecs_world_t, disable_component_scoping: bool) -> Self {
        if !disable_component_scoping {
            return Self {
                world,
                prev_scope: None,
                prev_with: None,
            };
        }

        let prev_scope = unsafe {
            let current_scope = sys::ecs_get_scope(world);
            if current_scope != 0 {
                sys::ecs_set_scope(world, 0);
                Some(current_scope)
            } else {
                None
            }
        };

        let prev_with = unsafe {
            let previous_with = sys::ecs_set_with(world, 0);
            if previous_with != 0 {
                Some(previous_with)
            } else {
                None
            }
        };

        Self {
            world,
            prev_scope,
            prev_with,
        }
    }
}

impl Drop for ScopeWithGuard {
    fn drop(&mut self) {
        if let Some(prev_with) = self.prev_with {
            unsafe { sys::ecs_set_with(self.world, prev_with) };
        }

        if let Some(prev_scope) = self.prev_scope {
            unsafe { sys::ecs_set_scope(self.world, prev_scope) };
        }
    }
}

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
) -> u64
where
    T: 'static,
{
    let world = world.world();
    external_register_component_data::<COMPONENT_REGISTRATION, T>(world, name) as u64
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
    let enum_size = const { core::mem::size_of::<T::UnderlyingTypeOfEnum>() };
    for enum_item in T::UnderlyingEnumType::iter() {
        let name = enum_item.name_cstr();
        let enum_index = enum_item.enum_index();
        let mut array_index = enum_index as usize;
        let entity_id: sys::ecs_entity_t = unsafe {
            sys::ecs_cpp_enum_constant_register(
                world,
                id,
                T::UnderlyingEnumType::id_variant_of_index_unchecked(enum_index),
                name.as_ptr(),
                &mut array_index as *mut usize as *mut c_void,
                underlying_type_id,
                enum_size,
            )
        };
        store_enum_entity_if_needed::<T>(enum_array_ptr, array_index, entity_id);
    }
}

#[inline(always)]
fn store_enum_entity_if_needed<T: ComponentId>(
    enum_array_ptr: *mut sys::ecs_entity_t,
    index: usize,
    entity_id: sys::ecs_entity_t,
) {
    if !T::UnderlyingEnumType::is_index_registered_as_entity(index) {
        unsafe { *enum_array_ptr.add(index) = entity_id };
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

    let _scope = ScopeWithGuard::new(world, !COMPONENT_REGISTRATION);

    let id = register_component_data_explicit::<T, false>(worldref, name);
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

    let _scope = ScopeWithGuard::new(world, !COMPONENT_REGISTRATION);

    let id = register_component_data_explicit::<T, false>(worldref, core::ptr::null());

    id
}

pub(crate) fn external_register_component_data<const COMPONENT_REGISTRATION: bool, T>(
    world: WorldRef<'_>,
    name: *const c_char,
) -> sys::ecs_entity_t
where
    T: 'static,
{
    let world_ptr = world.world_ptr_mut();
    let _scope = ScopeWithGuard::new(world_ptr, !COMPONENT_REGISTRATION);

    external_register_component_data_explicit::<T>(world, name)
}

/// registers the component with the world.
pub(crate) fn register_component_data_explicit<T, const ALLOCATE_TAG: bool>(
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

    let type_name_without_scope = if T::IS_GENERIC {
        crate::core::get_type_name_without_scope_generic::<T>()
    } else {
        crate::core::get_type_name_without_scope::<T>()
    };

    let type_name_without_scope =
        compact_str::format_compact!("{}\0", type_name_without_scope.as_str());

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
        user_name = type_name_without_scope.as_ptr() as *const c_char;
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
    let entity_desc_name = if type_name_without_scope.starts_with("Ecs") {
        type_name_without_scope.as_ptr() as *const c_char
    } else {
        type_name_ptr
    };
    let type_info = create_type_info::<T, ALLOCATE_TAG>();

    finalize_component_registration(world, name, entity_desc_name, type_info)
}

pub(crate) fn external_register_component_data_explicit<T>(
    world: WorldRef<'_>,
    name: *const c_char,
) -> sys::ecs_entity_t
where
    T: 'static,
{
    let world_ptr = world.world_ptr_mut();

    let type_name_without_scope = crate::core::get_type_name_without_scope_generic::<T>();
    let type_name_without_scope =
        compact_str::format_compact!("{}\0", type_name_without_scope.as_str());

    let id = if name.is_null() {
        let prev_scope = unsafe { sys::ecs_set_scope(world_ptr, 0) };
        let id = unsafe {
            sys::ecs_lookup_symbol(
                world_ptr,
                type_name_without_scope.as_ptr() as *const _,
                false,
                false,
            )
        };
        unsafe { sys::ecs_set_scope(world_ptr, prev_scope) };
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

    let type_info = external_create_type_info::<T>();

    finalize_component_registration(world_ptr, name, type_name_ptr, type_info)
}

fn finalize_component_registration(
    world: *mut sys::ecs_world_t,
    name: *const c_char,
    entity_desc_name: *const c_char,
    type_info: sys::ecs_type_info_t,
) -> sys::ecs_entity_t {
    let entity_desc = create_entity_desc(name, entity_desc_name);

    let entity = unsafe { flecs_ecs_sys::ecs_entity_init(world, &entity_desc) };

    let component_desc = create_component_desc(entity, type_info);

    #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    {
        let entity = unsafe { flecs_ecs_sys::ecs_component_init(world, &component_desc) };
        ecs_assert!(
            entity != 0 && unsafe { sys::ecs_exists(world, entity) },
            FlecsErrorCode::InternalError
        );

        entity
    }
    #[cfg(not(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts")))]
    {
        unsafe { flecs_ecs_sys::ecs_component_init(world, &component_desc) }
    }
}
