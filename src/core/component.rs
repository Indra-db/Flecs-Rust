use rand::random;
use std::ffi::CStr;
use std::sync::OnceLock;
use std::{any::type_name, os::raw::c_char};

use crate::core::c_binding::bindings::{ecs_cpp_component_register_explicit, ecs_get_path_w_sep};
use crate::{
    core::{
        c_binding::bindings::{ecs_get_symbol, ecs_set_scope, ecs_set_symbol},
        c_types::IdT,
        utility::errors::FlecsErrorCode,
        utility::functions::{get_full_type_name, get_only_type_name},
    },
    ecs_assert,
};

use super::c_binding::bindings::ecs_lookup_symbol;
use super::utility::functions::is_empty_type;
use super::{
    c_binding::bindings::{ecs_exists, ecs_set_with},
    c_types::{EntityT, WorldT},
    lifecycle_traits::register_lifecycle_actions,
};

#[derive(Debug)]
pub struct ComponentDescriptor {
    pub symbol: String,
    pub name: String,
    pub custom_id: Option<u64>,
    pub layout: std::alloc::Layout,
}

fn init<T: CachedComponentData>(
    entity: EntityT,
    allow_tag: bool,
    is_comp_pre_registered: bool,
) -> ComponentData {
    if is_comp_pre_registered {
        ecs_assert!(
            // we know this is safe because we checked it it's registered.
            unsafe { T::get_id_unchecked() } == entity,
            FlecsErrorCode::InconsistentComponentId,
            get_full_type_name::<T>()
        );
        ecs_assert!(
            // we know this is safe because we checked it it's registered.
            allow_tag == unsafe { T::get_allow_tag() },
            FlecsErrorCode::InvalidParameter
        );

        //this is safe because we're sure it's registered
        return unsafe { T::get_data_unchecked() }.clone();
    }

    let is_empty_and_tag_allowed = is_empty_type::<T>() && allow_tag;

    ComponentData {
        id: entity,
        size: if is_empty_and_tag_allowed {
            0
        } else {
            std::mem::size_of::<T>()
        },
        alignment: if is_empty_and_tag_allowed {
            0
        } else {
            std::mem::align_of::<T>()
        },
        allow_tag,
    }
}

//we might not need this if the cpp registration works for rust too, but we will see
//fn ecs_rust_component_register_explicit(
//    world: *mut WorldT,
//    s_id: EntityT,
//    id: EntityT,
//    name: *const c_char,
//    typename: &'static str,
//    symbol: &'static str,
//    size: usize,
//    aligment: usize,
//    is_component: bool,
//    is_existing: *mut bool,
//) {
//    static SEP: &'static [u8] = b"::\0";
//
//    let mut existing_name: &CStr = CStr::from_bytes_with_nul(b"\0").unwrap();
//    unsafe {
//        if *is_existing == true {
//            *is_existing = false;
//        }
//    }
//    let mut id = id;
//
//    if id != 0 {
//        if !name.is_null() {
//            // If no name was provided first check if a type with the provided
//            // symbol was already registered.
//            id = unsafe { ecs_lookup_symbol(world, symbol.as_ptr() as *const i8, false) };
//            if id != 0 {
//                unsafe {
//                    let sep = SEP.as_ptr() as *const i8;
//                    existing_name = CStr::from_ptr(ecs_get_path_w_sep(world, 0, id, sep, sep));
//                    name = existing_name;
//                    if !is_existing.is_null() {
//                        *is_existing = true;
//                    }
//                }
//            }
//        }
//    }
//}

//this is WIP. We can likely optimize this function by replacing the cpp func call by our own implementation
//TODO merge explicit and non explicit functions -> not necessary to have a similar impl as c++.
fn register_componment_data_explicit<T: CachedComponentData + Clone + Default>(
    world: *mut WorldT,
    name: *const c_char,
    allow_tag: bool,
    id: EntityT,
    is_componment: bool,
    existing: &mut bool,
    is_comp_pre_registered: bool,
) {
    let mut component_data: ComponentData = Default::default();
    if is_comp_pre_registered {
        // we know this is safe because we checked if the component is pre-registered
        component_data.id = unsafe { T::get_id_unchecked() };
    }

    if component_data.id != 0 {
        ecs_assert!(
            !world.is_null(),
            FlecsErrorCode::ComponentNotRegistered,
            name: *const c_char
        );
    } else {
        ecs_assert!(id == 0, FlecsErrorCode::InconsistentComponentId,);
    }

    //TODO evaluate if we can pass the ecs_exists result of the non explicit function.
    if !is_comp_pre_registered
        || (!world.is_null() && unsafe { !ecs_exists(world, component_data.id) })
    {
        component_data = init::<T>(
            if component_data.id == 0 {
                id
            } else {
                component_data.id
            },
            allow_tag,
            is_comp_pre_registered,
        );

        ecs_assert!(
            id == 0 || component_data.id == id,
            FlecsErrorCode::InternalError
        );

        let symbol = if id != 0 {
            let symbol_ptr = unsafe { ecs_get_symbol(world, id) };
            if symbol_ptr.is_null() {
                T::get_symbol_name()
            } else {
                unsafe { CStr::from_ptr(symbol_ptr).to_str() }.unwrap_or_else(|_| {
                    ecs_assert!(false, FlecsErrorCode::InternalError);
                    T::get_symbol_name()
                })
            }
        } else {
            T::get_symbol_name()
        };

        let type_name = get_full_type_name::<T>();

        let entity: EntityT = unsafe {
            //TODO check if this works for rust, likely not from the looks of it.
            ecs_cpp_component_register_explicit(
                world,
                component_data.id,
                id,
                name,
                type_name.as_ptr() as *const i8,
                symbol.as_ptr() as *const i8,
                component_data.size,
                component_data.alignment,
                is_componment,
                existing,
            )
        };

        component_data.id = entity;
        ecs_assert!(
            if !is_comp_pre_registered {
                component_data.id != 0 && unsafe { ecs_exists(world, component_data.id) }
            } else {
                true
            },
            FlecsErrorCode::InternalError
        );

        if !is_comp_pre_registered {
            T::__initialize(|| component_data);
        }
    }
}

/// this function is unsafe because it assumes that the component is registered with a world, not necessarily the world passed in.
unsafe fn is_component_registered_with_world<T: CachedComponentData>(world: *const WorldT) -> bool {
    // we know this is safe because we checked if world is not null & if the component is registered
    if !world.is_null() && unsafe { !ecs_exists(world, T::get_id_unchecked()) } {
        return false;
    }

    true
}

///TODO remove this comment, similar to id func
fn register_component_data<T: CachedComponentData + Clone + Default>(
    world: *mut WorldT,
    name: *const c_char,
    allow_tag: bool,
    is_comp_pre_registered: bool,
) {
    //this is safe because we checked if the component is pre-registered
    if !is_comp_pre_registered || unsafe { !is_component_registered_with_world::<T>(world) } {
        let mut prev_scope: EntityT = 0;
        let mut prev_with: EntityT = 0;

        if !world.is_null() {
            prev_scope = unsafe { ecs_set_scope(world, 0) };
            prev_with = unsafe { ecs_set_with(world, 0) };
        }

        let mut existing = false;
        register_componment_data_explicit::<T>(
            world,
            name,
            allow_tag,
            0,
            true,
            &mut existing,
            is_comp_pre_registered,
        );

        if unsafe { T::get_size() } != 0 && !existing {
            // we know this is safe because the component should be registered by now
            register_lifecycle_actions::<T>(world, unsafe { T::get_id_unchecked() })
        }

        if prev_with != 0 {
            unsafe { ecs_set_with(world, prev_with) };
        }
        if prev_scope != 0 {
            unsafe { ecs_set_scope(world, prev_scope) };
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ComponentData {
    pub id: u64,
    pub size: usize,
    pub alignment: usize,
    pub allow_tag: bool,
}

pub fn test() -> ComponentData {
    ComponentData {
        id: random(),
        size: 0,
        alignment: 0,
        allow_tag: false,
    }
}

//TODO consider adding safe functions, although it's likely never going to be used by the end user, only internally here.
// if that's the case, we can #[doc(hidden)] the unsafe functions and only expose the safe ones.
pub trait CachedComponentData: Clone + Default {
    //fn get_data() -> &'static ComponentData {
    //    let once_lock = Self::get_once_lock_data();
    //    once_lock.get_or_init(|| test())
    //}

    fn get_once_lock_data() -> &'static OnceLock<ComponentData> {
        static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
        &ONCE_LOCK
    }

    fn is_registered() -> bool {
        !Self::get_once_lock_data().get().is_none()
    }

    // Not public API.
    #[doc(hidden)]
    fn __initialize<F: FnOnce() -> ComponentData>(f: F) -> &'static ComponentData {
        Self::get_once_lock_data().get_or_init(f)
    }

    /// this function is unsafe because it assumes that the component is registered,
    /// the lock data being initialized is not checked.
    unsafe fn get_data_unchecked() -> &'static ComponentData {
        Self::get_once_lock_data().get().unwrap()
    }

    /// does not check if the component is registered in the world, if not, it might cause problems depending on usage.
    /// only use this if you know what you are doing and you are sure the component is registered in the world
    unsafe fn get_id_unchecked() -> IdT {
        Self::get_data_unchecked().id
    }

    //for the future: get_id_explicit will
    fn get_id(world: *mut WorldT) -> IdT {
        let is_registered = Self::is_registered();

        //TODO we can pass if it's registered with world.
        if !is_registered || unsafe { !is_component_registered_with_world::<Self>(world) } {
            register_component_data::<Self>(world, std::ptr::null(), true, is_registered);
        }

        //this is safe because we checked if the component is registered / registered it
        unsafe { Self::get_id_unchecked() }
    }

    /// this function is unsafe because it assumes that the component is registered,
    unsafe fn get_size() -> usize {
        Self::get_data_unchecked().size
    }

    /// this function is unsafe because it assumes that the component is registered,
    unsafe fn get_alignment() -> usize {
        Self::get_data_unchecked().alignment
    }

    /// this function is unsafe because it assumes that the component is registered,
    unsafe fn get_allow_tag() -> bool {
        Self::get_data_unchecked().allow_tag
    }

    /// returns [module].[type]
    fn get_symbol_name() -> &'static str {
        use std::any::type_name;
        static SYMBOL_NAME: OnceLock<String> = OnceLock::new();
        SYMBOL_NAME.get_or_init(|| {
            let name = type_name::<Self>();
            name.replace("::", ".")
        })
    }
}

macro_rules! impl_cached_component_data  {
    ($($t:ty),*) => {
        $(
            impl CachedComponentData for $t {
                fn get_once_lock_data() -> &'static OnceLock<ComponentData> {
                    static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
                    &ONCE_LOCK
                }
            }
        )*
    };
}
