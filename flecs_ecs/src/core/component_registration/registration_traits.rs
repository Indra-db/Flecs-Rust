use crate::core::*;
use std::{ffi::CStr, sync::OnceLock};

pub trait EmptyComponent {}

impl<T, U> EmptyComponent for (T, U)
where
    T: EmptyComponent,
    U: EmptyComponent,
{
}

#[diagnostic::on_unimplemented(
    message = "the size of type `{Self}` should not be zero, should not a tag.",
    label = "Supports only non-empty components"
)]
pub trait NotEmptyComponent {}

impl<T> NotEmptyComponent for &T where T: NotEmptyComponent {}
impl<T> NotEmptyComponent for &mut T where T: NotEmptyComponent {}
impl<T> NotEmptyComponent for Option<&T> where T: NotEmptyComponent {}
impl<T> NotEmptyComponent for Option<&mut T> where T: NotEmptyComponent {}

pub trait ECSComponentType {}

impl ECSComponentType for Enum {}
impl ECSComponentType for Struct {}

pub trait ComponentType<T: ECSComponentType> {}

impl<T> ComponentType<Enum> for &T where T: ComponentType<Enum> {}
impl<T> ComponentType<Enum> for &mut T where T: ComponentType<Enum> {}
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
    fn register_explicit_named<'a>(world: impl IntoWorld<'a>, name: &str) -> EntityT {
        try_register_component_named::<Self::UnderlyingType>(world, name)
    }

    /// checks if the component is registered with a world.
    #[inline(always)]
    fn is_registered() -> bool {
        Self::UnderlyingType::__get_once_lock_data().get().is_some()
    }

    /// checks if the component is registered with a world.
    /// # Safety
    /// This function is unsafe because it assumes world is not nullptr
    /// this is highly unlikely a world would be nullptr, hence this function is not marked as unsafe.
    /// this will be changed in the future where we get rid of the pointers.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn is_registered_with_world<'a>(world: impl IntoWorld<'a>) -> bool {
        if Self::UnderlyingType::is_registered() {
            unsafe { is_component_registered_with_world::<Self::UnderlyingType>(world.world_ptr()) }
        } else {
            false
        }
    }

    /// returns the component id of the component. If the component is not registered, it will register it.
    fn get_id<'a>(world: impl IntoWorld<'a>) -> IdT {
        #[cfg(feature = "flecs_manual_registration")]
        {
            ecs_assert!(
                {
                    if !Self::is_registered() || !Self::is_registered_with_world(world.world()) {
                        false
                    } else {
                        true
                    }
                },
                FlecsErrorCode::InvalidOperation,
                "Component {} is not registered with the world before usage",
                Self::name()
            );
            unsafe { Self::UnderlyingType::get_id_unchecked() }
        }
        #[cfg(not(feature = "flecs_manual_registration"))]
        {
            try_register_component::<Self::UnderlyingType>(world);
            unsafe { Self::UnderlyingType::get_id_unchecked() }
        }
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
        Self::UnderlyingType::__get_once_lock_data()
            .get()
            .unwrap_unchecked()
            .id
    }

    // Not public API.
    #[doc(hidden)]
    fn __get_once_lock_data() -> &'static OnceLock<IdComponent>;

    // Not public API.
    #[doc(hidden)]
    #[inline(always)]
    fn __initialize<F: FnOnce() -> IdComponent>(f: F) -> &'static IdComponent {
        Self::UnderlyingType::__get_once_lock_data().get_or_init(f)
    }

    // Not public API.
    #[doc(hidden)]
    fn __register_lifecycle_hooks(_type_hooks: &mut TypeHooksT) {}

    // Not public API.
    #[doc(hidden)]
    fn __register_default_hooks(_type_hooks: &mut TypeHooksT) {}

    // Not public API.
    #[doc(hidden)]
    fn __register_clone_hooks(_type_hooks: &mut TypeHooksT) {}

    #[doc(hidden)]
    /// this is cursed, but it's the only way to reset the lock data for the component without making the static mutable.
    /// this is ONLY used for benchmarking purposes.
    fn __reset_one_lock_data() {
        #[allow(invalid_reference_casting)]
        {
            let lock: &'static mut OnceLock<IdComponent> = unsafe {
                &mut *(Self::UnderlyingType::__get_once_lock_data() as *const OnceLock<IdComponent>
                    as *mut OnceLock<IdComponent>)
            };

            lock.take();
        }
    }
}

pub trait ComponentInfo: Sized {
    const IS_ENUM: bool;
    const IS_TAG: bool;
    const NEEDS_DROP: bool = std::mem::needs_drop::<Self>();
    const IMPLS_CLONE: bool;
    const IMPLS_DEFAULT: bool;
    const IS_REF: bool;
    const IS_MUT: bool;
    type TagType;
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
    fn get_id_variant<'a>(&self, world: impl IntoWorld<'a>) -> EntityView<'a> {
        try_register_component::<Self>(world.world());
        let index = self.enum_index();
        EntityView::new_from(world, unsafe { *Self::__enum_data_mut().add(index) })
    }

    /// # Safety
    ///
    /// This function is unsafe because it assumes the enum has been registered as a component with the world.
    /// if uncertain, use `try_register_component::<T>` to try and register it
    unsafe fn get_id_variant_unchecked<'a>(&self, world: impl IntoWorld<'a>) -> EntityView<'a> {
        let index = self.enum_index();
        EntityView::new_from(world, unsafe { *Self::__enum_data_mut().add(index) })
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
    const IMPLS_CLONE: bool = T::IMPLS_CLONE;
    const IMPLS_DEFAULT: bool = T::IMPLS_DEFAULT;
    const IS_REF: bool = true;
    const IS_MUT: bool = false;
    type TagType = T::TagType;
}

impl<T: ComponentInfo> ComponentInfo for &mut T {
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TAG: bool = T::IS_TAG;
    const IMPLS_CLONE: bool = T::IMPLS_CLONE;
    const IMPLS_DEFAULT: bool = T::IMPLS_DEFAULT;
    const IS_REF: bool = false;
    const IS_MUT: bool = true;
    type TagType = T::TagType;
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

pub trait FlecsDefaultType {
    type Type: Default;
}

pub trait FlecsCloneType {
    type Type: Clone;
}

pub trait FlecsPairType {
    type Type: ComponentId + NotEmptyComponent;
    const IS_FIRST: bool;
}

impl<T> FlecsDefaultType for ConditionalTypeSelector<false, T> {
    type Type = FlecsNoneDefaultDummy;
}

impl<T> FlecsDefaultType for ConditionalTypeSelector<true, T>
where
    T: Default,
{
    type Type = T;
}

impl<T> FlecsCloneType for ConditionalTypeSelector<false, T> {
    type Type = FlecsNoneCloneDummy;
}

impl<T> FlecsCloneType for ConditionalTypeSelector<true, T>
where
    T: Clone,
{
    type Type = T;
}

pub struct FlecsFirstIsNotATag;
pub struct FlecsFirstIsATag;

impl<T> FlecsPairType for ConditionalTypePairSelector<FlecsFirstIsNotATag, T>
where
    T: IntoComponentId,
    T::First: NotEmptyComponent + ComponentId,
{
    type Type = T::First;
    const IS_FIRST: bool = true;
}

impl<U> FlecsPairType for ConditionalTypePairSelector<FlecsFirstIsATag, U>
where
    U: IntoComponentId,
    U::Second: NotEmptyComponent + ComponentId,
{
    type Type = U::Second;
    const IS_FIRST: bool = false;
}

pub trait FlecsCastType: IntoComponentId {
    type CastType: NotEmptyComponent + ComponentId;
    const IS_FIRST: bool;
}

impl<T> FlecsCastType for T
where
    T: IntoComponentId,
    flecs_ecs::core::ConditionalTypePairSelector<<T::First as ComponentInfo>::TagType, T>:
        flecs_ecs::core::FlecsPairType,
    <T as flecs_ecs::core::IntoComponentId>::First: ComponentInfo,
{
    type CastType =
        <ConditionalTypePairSelector<<T::First as ComponentInfo>::TagType, T> as FlecsPairType>::Type;
    const IS_FIRST : bool = <ConditionalTypePairSelector<<T::First as ComponentInfo>::TagType, T> as FlecsPairType>::IS_FIRST;
}
