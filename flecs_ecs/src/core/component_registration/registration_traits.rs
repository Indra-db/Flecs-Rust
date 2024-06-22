use crate::core::*;
use std::{
    ffi::CStr,
    sync::atomic::{AtomicU32, Ordering},
};

pub trait TagComponent {}

impl<T, U> TagComponent for (T, U)
where
    T: TagComponent,
    U: TagComponent,
{
}

#[diagnostic::on_unimplemented(
    message = "the size of type `{Self}` should not be zero, should not be a tag.",
    label = "Supports only non-empty components"
)]
pub trait DataComponent {}

impl<T> DataComponent for &T where T: DataComponent {}
impl<T> DataComponent for &mut T where T: DataComponent {}
impl<T> DataComponent for Option<&T> where T: DataComponent {}
impl<T> DataComponent for Option<&mut T> where T: DataComponent {}
impl<T, U> DataComponent for (T, U)
where
    T: ComponentId,
    U: ComponentId,
    (T, U): ComponentOrPairId,
    <(T, U) as ComponentOrPairId>::CastType: DataComponent,
    registration_types::ConditionalTypePairSelector<
        <<(T, U) as ComponentOrPairId>::First as registration_traits::ComponentInfo>::TagType,
        T,
        U,
    >: registration_traits::FlecsPairType,
{
}

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
    #[doc(hidden)]
    #[inline(always)]
    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        world: impl IntoWorld<'a>,
    ) -> EntityT {
        if !Self::IS_GENERIC {
            let index = Self::index() as usize;
            let world = world.world();
            let components_array = world.components_array();
            let len = components_array.len();

            if len > index {
                if components_array[index] == 0 {
                    if MANUAL_REGISTRATION_CHECK {
                        #[cfg(feature = "flecs_manual_registration")]
                        {
                            ecs_assert!(
                                false,
                                FlecsErrorCode::InvalidOperation,
                                "Component {} is not registered with the world before usage",
                                Self::name()
                            );
                        }
                    }
                    let id = try_register_component::<Self>(world);
                    components_array[index] = id;
                    return id;
                }
                components_array[index]
            } else {
                components_array.reserve(len);
                let capacity = components_array.capacity();
                unsafe {
                    std::ptr::write_bytes(
                        components_array.as_mut_ptr().add(len),
                        0,
                        capacity - len,
                    );
                    components_array.set_len(capacity);
                }
                let id = try_register_component::<Self>(world);
                components_array[index] = id;
                id
            }
        } else {
            let world = world.world();
            let components_map = world.components_map();
            *(components_map
                .entry(std::any::TypeId::of::<Self>())
                .or_insert_with(|| {
                    if MANUAL_REGISTRATION_CHECK {
                        #[cfg(feature = "flecs_manual_registration")]
                        {
                            ecs_assert!(
                                false,
                                FlecsErrorCode::InvalidOperation,
                                "Component {} is not registered with the world before usage",
                                Self::name()
                            );
                        }
                    }
                    try_register_component::<Self>(world)
                }))
        }
    }

    /// attempts to register the component with name with the world. If it's already registered, it does nothing.
    #[inline(always)]
    #[doc(hidden)]
    fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        world: impl IntoWorld<'a>,
        name: &str,
    ) -> EntityT {
        if !Self::IS_GENERIC {
            let index = Self::index() as usize;
            let world = world.world();
            let components_array = world.components_array();
            let len = components_array.len();

            if len > index {
                if components_array[index] == 0 {
                    if MANUAL_REGISTRATION_CHECK {
                        #[cfg(feature = "flecs_manual_registration")]
                        {
                            ecs_assert!(
                                false,
                                FlecsErrorCode::InvalidOperation,
                                "Component {} is not registered with the world before usage",
                                Self::name()
                            );
                        }
                    }
                    let id = try_register_component_named::<Self>(world, name);
                    components_array[index] = id;
                    return id;
                }
                components_array[index]
            } else {
                components_array.reserve(len);
                let capacity = components_array.capacity();
                unsafe {
                    std::ptr::write_bytes(
                        components_array.as_mut_ptr().add(len),
                        0,
                        capacity - len,
                    );
                    components_array.set_len(capacity);
                }
                let id = try_register_component_named::<Self>(world, name);
                components_array[index] = id;
                id
            }
        } else {
            let world = world.world();
            let components_map = world.components_map();
            *(components_map
                .entry(std::any::TypeId::of::<Self>())
                .or_insert_with(|| {
                    if MANUAL_REGISTRATION_CHECK {
                        #[cfg(feature = "flecs_manual_registration")]
                        {
                            ecs_assert!(
                                false,
                                FlecsErrorCode::InvalidOperation,
                                "Component {} is not registered with the world before usage",
                                Self::name()
                            );
                        }
                    }
                    try_register_component_named::<Self::UnderlyingType>(world, name)
                }))
        }
    }

    /// checks if the component is registered with a world.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    #[doc(hidden)]
    fn is_registered_with_world<'a>(world: impl IntoWorld<'a>) -> bool {
        if !Self::IS_GENERIC {
            let index = Self::index();
            let world = world.world();
            let components_array = world.components_array();
            let len = components_array.len();

            if len > index as usize {
                components_array[index as usize] != 0
            } else {
                false
            }
        } else {
            let world = world.world();
            let components_map = world.components_map();
            components_map.contains_key(&std::any::TypeId::of::<Self>())
        }
    }

    /// returns the component id of the component. If the component is not registered, it will register it.
    #[inline(always)]
    fn id<'a>(world: impl IntoWorld<'a>) -> EntityT {
        Self::UnderlyingType::__register_or_get_id::<true>(world)
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
    #[inline(always)]
    fn fetch_new_index() -> u32 {
        static INDEX_POOL: AtomicU32 = AtomicU32::new(1);
        INDEX_POOL.fetch_add(1, Ordering::Relaxed)
    }

    #[doc(hidden)]
    #[inline(always)]
    fn get_or_init_index(id: &AtomicU32) -> u32 {
        match id.fetch_update(Ordering::Acquire, Ordering::Relaxed, |v| {
            if v != u32::MAX {
                None
            } else {
                Some(Self::fetch_new_index())
            }
        }) {
            Ok(_) => id.load(Ordering::Acquire),
            Err(old) => old,
        }
    }

    #[doc(hidden)]
    fn index() -> u32;
}

pub trait ComponentInfo: Sized {
    const IS_GENERIC: bool;
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
    const IS_GENERIC: bool = T::IS_GENERIC;
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TAG: bool = T::IS_TAG;
    const IMPLS_CLONE: bool = T::IMPLS_CLONE;
    const IMPLS_DEFAULT: bool = T::IMPLS_DEFAULT;
    const IS_REF: bool = true;
    const IS_MUT: bool = false;
    type TagType = T::TagType;
}

impl<T: ComponentInfo> ComponentInfo for &mut T {
    const IS_GENERIC: bool = T::IS_GENERIC;
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TAG: bool = T::IS_TAG;
    const IMPLS_CLONE: bool = T::IMPLS_CLONE;
    const IMPLS_DEFAULT: bool = T::IMPLS_DEFAULT;
    const IS_REF: bool = false;
    const IS_MUT: bool = true;
    type TagType = T::TagType;
}

impl<T: ComponentId> ComponentId for &'static T {
    #[inline(always)]
    fn index() -> u32 {
        T::UnderlyingType::index()
    }

    type UnderlyingType = T::UnderlyingType;

    type UnderlyingEnumType = T::UnderlyingEnumType;
}

impl<T: ComponentId> ComponentId for &'static mut T {
    #[inline(always)]
    fn index() -> u32 {
        T::UnderlyingType::index()
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
    type Type: ComponentId;
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

impl<T, U> FlecsPairType for ConditionalTypePairSelector<FlecsFirstIsNotATag, T, U>
where
    T: ComponentId,
    U: ComponentId,
{
    type Type = T;
    const IS_FIRST: bool = true;
}

impl<T, U> FlecsPairType for ConditionalTypePairSelector<FlecsFirstIsATag, T, U>
where
    T: ComponentId,
    U: ComponentId,
{
    type Type = U;
    const IS_FIRST: bool = false;
}
