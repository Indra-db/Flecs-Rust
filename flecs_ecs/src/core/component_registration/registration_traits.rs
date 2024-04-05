use std::{ffi::CStr, sync::OnceLock};

use crate::core::{Entity, EntityT, Enum, IdComponent, IdT, IntoWorld, Struct};

use super::{
    is_component_registered_with_world, try_register_component, try_register_component_named,
};

pub trait EmptyComponent {}
pub trait NotEmptyComponent {}

pub trait ECSComponentType {}

impl ECSComponentType for Enum {}
impl ECSComponentType for Struct {}

pub trait ComponentType<T: ECSComponentType> {}

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
/// The `ComponentId` trait is designed to maintain component IDs for a Rust type
/// in a manner that is consistent across different worlds (or instances).
/// When a component is utilized, this trait will determine whether it has already been registered.
/// If it hasn't, it registers the component with the current world.
///
/// If the ID has been previously established, the trait ensures the world recognizes it.
/// If the world doesn't, this implies the component was registered by a different world.
/// In such a case, the component is registered with the present world using the pre-existing ID.
/// If the ID is already known, the trait takes care of the component registration and checks for consistency in the input.
pub trait ComponentId: Sized + ComponentInfo {
    type UnderlyingType: ComponentId + Default + Clone;
    type UnderlyingEnumType: ComponentId + CachedEnumData + Default + Clone;

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
    fn __get_once_lock_data() -> &'static OnceLock<IdComponent>;

    // Not public API.
    #[doc(hidden)]
    #[inline(always)]
    fn __initialize<F: FnOnce() -> IdComponent>(f: F) -> &'static IdComponent {
        Self::__get_once_lock_data().get_or_init(f)
    }
}

pub trait ComponentInfo: Sized {
    const IS_ENUM: bool;
    const IS_TAG: bool;
    const NEEDS_DROP: bool = std::mem::needs_drop::<Self>();
}

pub trait CachedEnumData: ComponentType<Enum> {
    const SIZE_ENUM_FIELDS: u32;
    type VariantIterator: Iterator<Item = Self>;

    /// # Note
    /// this function is used to pass the name to the C API.
    fn get_cstr_name(&self) -> &CStr;

    fn get_enum_index(&self) -> usize;

    fn iter() -> Self::VariantIterator;

    /// # Note
    /// it only means that the enum is registered with a particular world, not necessarily yours.
    fn are_fields_registered_as_entities() -> bool {
        let mut result = true;
        let ptr = Self::__get_enum_data_ptr_mut();
        for i in 0..Self::SIZE_ENUM_FIELDS {
            unsafe {
                if *ptr.add(i as usize) == 0 {
                    result = false;
                    break;
                }
            }
        }
        result
    }

    fn is_field_registered_as_entity(&self) -> bool {
        let index = self.get_enum_index();
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) != 0 }
    }

    fn is_index_registered_as_entity(index: usize) -> bool {
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) != 0 }
    }

    /// # Safety
    ///
    /// This function is unsafe because it assumes the enum has been registered as a component with the world.
    /// if uncertain, use `try_register_component::<T>` to try and register it
    unsafe fn get_entity_id_from_enum_field(&self, world: impl IntoWorld) -> Entity {
        let index = self.get_enum_index();
        Entity::new_from_existing_raw(world, unsafe {
            *Self::__get_enum_data_ptr_mut().add(index)
        })
    }

    /// ## Safety
    /// this function assumes you're sure that the enum fields are registered previously
    /// if uncertain use `get_entity_id_from_enum_field`
    ///
    /// # Returns
    ///
    /// The Entity without the world attached to it. Use `Entity::new_from_existing_raw` to attach a world to it.
    /// use with caution if no world is attached to the entity, it will cause a panic when you try to operate on it.
    unsafe fn get_entity_id_from_enum_field_unchecked(&self) -> Entity {
        let index = self.get_enum_index();

        Entity::new_id_only(unsafe { *Self::__get_enum_data_ptr_mut().add(index) })
    }

    /// ## Safety
    /// This function is unsafe because it dereferences a raw pointer and you must ensure that the
    /// index is within the bounds of the number of variants in the enum.
    /// if uncertain, use `SIZE_ENUM_FIELDS` to check the number of variants.
    unsafe fn get_entity_id_from_enum_field_index(index: usize) -> u64 {
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) }
    }

    #[doc(hidden)]
    fn __get_enum_data_ptr_mut() -> *mut u64;
}

impl<T: ComponentInfo> ComponentInfo for &T {
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TAG: bool = T::IS_TAG;
}

impl<T: ComponentInfo> ComponentInfo for &mut T {
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TAG: bool = T::IS_TAG;
}

impl<T: ComponentId> ComponentId for &T {
    fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
        Self::UnderlyingType::__get_once_lock_data()
    }

    type UnderlyingType = T::UnderlyingType;

    type UnderlyingEnumType = T::UnderlyingEnumType;
}

impl<T: ComponentId> ComponentId for &mut T {
    fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
        Self::UnderlyingType::__get_once_lock_data()
    }

    type UnderlyingType = T::UnderlyingType;

    type UnderlyingEnumType = T::UnderlyingEnumType;
}
