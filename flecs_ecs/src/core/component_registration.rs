//! Registering and working with components.

use flecs_ecs_derive::Component;
use flecs_ecs_sys::ecs_world_t;

use super::{
    c_types::{EntityT, IdT, WorldT},
    create_lifecycle_actions,
    enum_type::CachedEnumData,
    IntoWorld,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;
use crate::{
    core::SEPARATOR,
    ecs_assert,
    sys::{
        ecs_cpp_enum_constant_register, ecs_cpp_enum_init, ecs_exists, ecs_get_symbol,
        ecs_set_scope, ecs_set_with,
    },
};
use std::{ffi::CStr, os::raw::c_char, sync::OnceLock};
/// Component data that is cached by the `ComponentInfo` trait.
/// This data is used to register components with the world.
/// It is also used to ensure that components are registered consistently across different worlds.
#[derive(Clone, Debug, Default)]
pub struct ComponentId {
    pub id: u64,
}

pub struct Enum;
pub struct Struct;

#[derive(Clone, Debug, Default, Component)]
pub enum NoneEnum {
    #[default]
    None,
}

pub trait EmptyComponent: ComponentInfo {}
pub trait NotEmptyComponent: ComponentInfo {}
pub trait IsEnum: ComponentInfo {
    const IS_ENUM: bool;
}
pub trait ECSComponentType {}

impl ECSComponentType for Enum {}
impl ECSComponentType for Struct {}

pub trait ComponentType<T: ECSComponentType>: ComponentInfo {}

/// Trait that manages component IDs across multiple worlds & binaries.
///
/// proc macro Component should be used to implement this trait automatically
///
#[cfg_attr(doctest, doc = " ````no_test")]
/// ```
///     #[derive(Component)] //this will implement the trait for the type
///      struct Position {t
///          vec: Vec<i32>,
///      }
/// ```
///
/// The `ComponentInfo` trait is designed to maintain component IDs for a Rust type
/// in a manner that is consistent across different worlds (or instances).
/// When a component is utilized, this trait will determine whether it has already been registered.
/// If it hasn't, it registers the component with the current world.
///
/// If the ID has been previously established, the trait ensures the world recognizes it.
/// If the world doesn't, this implies the component was registered by a different world.
/// In such a case, the component is registered with the present world using the pre-existing ID.
/// If the ID is already known, the trait takes care of the component registration and checks for consistency in the input.
pub trait ComponentInfo: Sized {
    type UnderlyingType: ComponentInfo + Default + Clone;
    type UnderlyingEnumType: ComponentInfo + CachedEnumData + Default + Clone;
    const IS_ENUM: bool;
    const IS_TAG: bool;

    /// attempts to register the component with the world. If it's already registered, it does nothing.
    fn register_explicit(world: impl IntoWorld) {
        try_register_component::<Self::UnderlyingType>(world);
    }

    /// attempts to register the component with name with the world. If it's already registered, it does nothing.
    fn register_explicit_named(world: impl IntoWorld, name: &CStr) -> EntityT {
        try_register_component_named::<Self::UnderlyingType>(world, name)
    }

    /// checks if the component is registered with a world.
    #[inline(always)]
    fn is_registered() -> bool {
        Self::__get_once_lock_data().get().is_some()
    }

    /// checks if the component is registered with a world.
    /// # Safety
    /// This function is unsafe because it assumes world is not nullptr
    /// this is highly unlikely a world would be nullptr, hence this function is not marked as unsafe.
    /// this will be changed in the future where we get rid of the pointers.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn is_registered_with_world(world: impl IntoWorld) -> bool {
        if Self::is_registered() {
            unsafe {
                is_component_registered_with_world::<Self::UnderlyingType>(world.get_world_raw())
            }
        } else {
            false
        }
    }

    /// returns the component id of the component. If the component is not registered, it will register it.
    fn get_id(world: impl IntoWorld) -> IdT {
        try_register_component::<Self::UnderlyingType>(world);
        unsafe { Self::get_id_unchecked() }
    }

    /// returns the component id of the component.
    /// # Safety
    /// safe version is `get_id`
    /// this function is unsafe because it assumes that the component is registered,
    /// the lock data being initialized is not checked and will panic if it's not.
    /// does not check if the component is registered in the world, if not, it might cause problems depending on usage.
    /// only use this if you know what you are doing and you are sure the component is registered in the world
    #[inline(always)]
    unsafe fn get_id_unchecked() -> IdT {
        Self::__get_once_lock_data().get().unwrap_unchecked().id
    }

    // Not public API.
    #[doc(hidden)]
    fn __get_once_lock_data() -> &'static OnceLock<ComponentId>;

    // Not public API.
    #[doc(hidden)]
    #[inline(always)]
    fn __initialize<F: FnOnce() -> ComponentId>(f: F) -> &'static ComponentId {
        Self::__get_once_lock_data().get_or_init(f)
    }
}

//TODO need to support getting the id of a component if it's a pair type
/// attempts to register the component with the world. If it's already registered, it does nothing.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub(crate) fn try_register_component_impl<T>(world: impl IntoWorld, name: *const c_char) -> EntityT
where
    T: ComponentInfo,
{
    let world = world.get_world_raw_mut();
    let is_registered = T::is_registered();
    let is_registered_with_world = if is_registered {
        unsafe { is_component_registered_with_world::<T>(world) }
    } else {
        false
    };

    if !is_registered || !is_registered_with_world {
        let has_newly_registered =
            register_component_data::<T>(world, name, is_registered, is_registered_with_world);

        if T::IS_ENUM && has_newly_registered && !is_registered_with_world {
            unsafe { ecs_cpp_enum_init(world, T::get_id_unchecked()) };
            let enum_array_ptr = T::UnderlyingEnumType::__get_enum_data_ptr_mut();

            for (index, enum_item) in T::UnderlyingEnumType::iter().enumerate() {
                let name = enum_item.get_cstr_name();
                let entity_id: EntityT = unsafe {
                    ecs_cpp_enum_constant_register(
                        world,
                        T::get_id_unchecked(),
                        T::UnderlyingEnumType::get_entity_id_from_enum_field_index(
                            enum_item.get_enum_index(),
                        ),
                        name.as_ptr(),
                        index as i32,
                    )
                };
                if !T::UnderlyingEnumType::is_index_registered_as_entity(index) {
                    unsafe { *enum_array_ptr.add(index) = entity_id };
                }
            }
        }
    }

    unsafe { T::get_id_unchecked() }
}

/// attempts to register the component with the world. If it's already registered, it does nothing.
pub fn try_register_component<T>(world: impl IntoWorld)
where
    T: ComponentInfo,
{
    try_register_component_impl::<T>(world, std::ptr::null());
}

pub fn try_register_component_named<T>(world: impl IntoWorld, name: &CStr) -> EntityT
where
    T: ComponentInfo,
{
    try_register_component_impl::<T>(world, name.as_ptr())
}

/// registers the component with the world.
//this is WIP. We can likely optimize this function by replacing the cpp func call by our own implementation
//TODO merge explicit and non explicit functions -> not necessary to have a similar impl as c++.
//need to cleanup this function.
fn register_componment_data_explicit<T>(
    world: impl IntoWorld,
    name: *const c_char,
    id: EntityT,
    is_comp_pre_registered: bool,
    is_comp_pre_registered_with_world: bool,
) -> EntityT
where
    T: ComponentInfo,
{
    let world = world.get_world_raw_mut();

    ecs_assert!(
        if id == 0 {
            !world.is_null()
        } else {
            true
        },
        FlecsErrorCode::ComponentNotRegistered,
        name: *const c_char
    );

    let type_name = crate::core::get_type_name_cstring::<T>();
    let type_name_ptr = type_name.as_ptr();

    let symbol = get_symbol_name(id, world, type_name_ptr, is_comp_pre_registered_with_world);

    let name = if name.is_null() { type_name_ptr } else { name };

    let entity_desc = get_new_entity_desc(name, symbol, id);

    let entity = unsafe { flecs_ecs_sys::ecs_entity_init(world, &entity_desc) };

    let type_info = get_new_type_info::<T>();

    let component_desc = get_new_component_desc(entity, type_info);

    let entity = unsafe { flecs_ecs_sys::ecs_component_init(world, &component_desc) };

    ecs_assert!(
        if !is_comp_pre_registered {
            entity != 0 && unsafe { ecs_exists(world, entity) }
        } else {
            true
        },
        FlecsErrorCode::InternalError
    );

    if !is_comp_pre_registered {
        T::__initialize(|| ComponentId { id: entity });
    }

    entity
}

/// registers the component with the world.
pub(crate) fn register_component_data<T>(
    world: impl IntoWorld,
    name: *const c_char,
    is_comp_pre_registered: bool,
    is_comp_pre_registered_with_world: bool,
) -> bool
where
    T: ComponentInfo,
{
    let world = world.get_world_raw_mut();

    // If the component is not registered with the world (indicating the
    // component has not yet been registered, or the component is used
    // across more than one binary), or if the id does not exists in the
    // world (indicating a multi-world application), register it.
    if !is_comp_pre_registered || !is_comp_pre_registered_with_world {
        let mut prev_scope: EntityT = 0;
        let mut prev_with: EntityT = 0;

        if !world.is_null() {
            prev_scope = unsafe { ecs_set_scope(world, 0) };
            prev_with = unsafe { ecs_set_with(world, 0) };
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
            unsafe { ecs_set_with(world, prev_with) };
        }
        if prev_scope != 0 {
            unsafe { ecs_set_scope(world, prev_scope) };
        }

        return true;
    }

    false
}

////// Helper functions
fn get_new_component_desc(
    entity: EntityT,
    type_info: flecs_ecs_sys::ecs_type_info_t,
) -> flecs_ecs_sys::ecs_component_desc_t {
    
    flecs_ecs_sys::ecs_component_desc_t {
        _canary: 0,
        entity,
        type_: type_info,
    }
}

fn get_new_type_info<T>() -> flecs_ecs_sys::ecs_type_info_t
where
    T: ComponentInfo,
{
    let size = std::mem::size_of::<T>();
    let alignment = if size != 0 {
        std::mem::align_of::<T>()
    } else {
        0
    };

    let hooks = if size != 0 {
        // Register lifecycle callbacks, but only if the component has a
        // size. Components that don't have a size are tags, and tags don't
        // require construction/destruction/copy/move's.
        create_lifecycle_actions::<T::UnderlyingType>()
    } else {
        Default::default()
    };

    let type_info: flecs_ecs_sys::ecs_type_info_t = flecs_ecs_sys::ecs_type_info_t {
        size: size as i32,
        alignment: alignment as i32,
        hooks,
        component: 0,
        name: std::ptr::null(),
    };
    type_info
}

fn get_new_entity_desc(
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

fn get_symbol_name(
    id: IdT,
    world: *mut ecs_world_t,
    type_name_ptr: *const c_char,
    is_comp_pre_registered_with_world: bool,
) -> *const i8 {
    
    if id != 0 {
        let symbol_ptr = if is_comp_pre_registered_with_world {
            unsafe { ecs_get_symbol(world, id) }
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
    T: ComponentInfo,
{
    // we know this is safe because we checked if world is not null & if the component is registered
    if !world.is_null() && unsafe { !ecs_exists(world, T::get_id_unchecked()) } {
        return false;
    }

    true
}
