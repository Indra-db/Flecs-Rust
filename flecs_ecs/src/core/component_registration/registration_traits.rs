use std::{ffi::CStr, sync::OnceLock};

use crate::core::{
    ConditionalTypeSelector, DefaultCloneDummy, Entity, EntityT, Enum, IdComponent, IdT, IntoWorld,
    Struct, TypeHooksT,
};

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
pub trait ComponentId: Sized + ComponentInfo + 'static {
    type UnderlyingType: ComponentId;
    type UnderlyingEnumType: ComponentId + CachedEnumData;

    /// attempts to register the component with the world. If it's already registered, it does nothing.
    fn register_explicit<'a>(world: impl IntoWorld<'a>) {
        try_register_component::<Self::UnderlyingType>(world);
    }

    /// attempts to register the component with name with the world. If it's already registered, it does nothing.
    fn register_explicit_named<'a>(world: impl IntoWorld<'a>, name: &CStr) -> EntityT {
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
    fn is_registered_with_world<'a>(world: impl IntoWorld<'a>) -> bool {
        if Self::is_registered() {
            unsafe {
                is_component_registered_with_world::<Self::UnderlyingType>(world.get_world_raw())
            }
        } else {
            false
        }
    }

    /// returns the component id of the component. If the component is not registered, it will register it.
    fn get_id<'a>(world: impl IntoWorld<'a>) -> IdT {
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

    // Not public API.
    #[doc(hidden)]
    fn __register_lifecycle_hooks() -> TypeHooksT {
        Default::default()
    }
}

pub trait ComponentInfo: Sized {
    const IS_ENUM: bool;
    const IS_TAG: bool;
    const NEEDS_DROP: bool = std::mem::needs_drop::<Self>();
}

pub trait CachedEnumData: ComponentType<Enum> + ComponentId {
    const SIZE_ENUM_FIELDS: u32;
    type VariantIterator: Iterator<Item = Self>;

    /// # Note
    /// this function is used to pass the name to the C API.
    fn name_cstr(&self) -> &CStr;

    fn enum_index(&self) -> usize;

    fn iter() -> Self::VariantIterator;

    /// # Note
    /// it only means that the enum is registered with a particular world, not necessarily yours.
    fn are_fields_registered_as_entities() -> bool {
        let mut result = true;
        let ptr = Self::__enum_data_mut();
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
        let index = self.enum_index();
        unsafe { *Self::__enum_data_mut().add(index) != 0 }
    }

    fn is_index_registered_as_entity(index: usize) -> bool {
        unsafe { *Self::__enum_data_mut().add(index) != 0 }
    }

    /// get the entity id of the variant of the enum. This function will register the enum with the world if it's not registered.
    fn get_id_variant<'a>(&self, world: impl IntoWorld<'a>) -> Entity<'a> {
        try_register_component::<Self>(world.world_ref());
        let index = self.enum_index();
        Entity::new_from_existing_raw(world, unsafe { *Self::__enum_data_mut().add(index) })
    }

    /// # Safety
    ///
    /// This function is unsafe because it assumes the enum has been registered as a component with the world.
    /// if uncertain, use `try_register_component::<T>` to try and register it
    unsafe fn get_id_variant_unchecked<'a>(&self, world: impl IntoWorld<'a>) -> Entity<'a> {
        let index = self.enum_index();
        Entity::new_from_existing_raw(world, unsafe { *Self::__enum_data_mut().add(index) })
    }

    fn get_id_variant_of_index(index: usize) -> Option<u64> {
        if index < Self::SIZE_ENUM_FIELDS as usize {
            Some(unsafe { *Self::__enum_data_mut().add(index) })
        } else {
            None
        }
    }

    /// ## Safety
    /// This function is unsafe because it dereferences a raw pointer and you must ensure that the
    /// index is within the bounds of the number of variants in the enum.
    /// if uncertain, use `SIZE_ENUM_FIELDS` to check the number of variants.
    unsafe fn get_id_variant_of_index_unchecked(index: usize) -> u64 {
        unsafe { *Self::__enum_data_mut().add(index) }
    }

    #[doc(hidden)]
    fn __enum_data_mut() -> *mut u64;
}

impl<T: ComponentInfo> ComponentInfo for &T {
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TAG: bool = T::IS_TAG;
}

impl<T: ComponentInfo> ComponentInfo for &mut T {
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TAG: bool = T::IS_TAG;
}

impl<T: ComponentId> ComponentId for &'static T {
    fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
        Self::UnderlyingType::__get_once_lock_data()
    }

    type UnderlyingType = T::UnderlyingType;

    type UnderlyingEnumType = T::UnderlyingEnumType;
}

impl<T: ComponentId> ComponentId for &'static mut T {
    fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
        Self::UnderlyingType::__get_once_lock_data()
    }

    type UnderlyingType = T::UnderlyingType;

    type UnderlyingEnumType = T::UnderlyingEnumType;
}

pub trait DefaultCloneType {
    type Type: Default + Clone;
}

impl<T> DefaultCloneType for ConditionalTypeSelector<false, T> {
    type Type = DefaultCloneDummy;
}

impl<T> DefaultCloneType for ConditionalTypeSelector<true, T>
where
    T: Default + Clone,
{
    type Type = T;
}
