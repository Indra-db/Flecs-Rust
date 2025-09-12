#[cfg(feature = "flecs_app")]
pub mod app;

#[cfg(feature = "flecs_doc")]
pub mod doc;

#[cfg(feature = "flecs_module")]
pub mod module;

#[cfg(feature = "flecs_system")]
pub mod system;

#[cfg(feature = "flecs_pipeline")]
pub mod pipeline;

#[cfg(feature = "flecs_stats")]
pub mod stats;

#[cfg(feature = "flecs_timer")]
pub mod timer;

#[cfg(feature = "flecs_meta")]
pub mod meta;

#[cfg(feature = "flecs_script")]
pub mod script;

#[cfg(feature = "flecs_json")]
pub mod json;

#[cfg(feature = "flecs_units")]
pub mod units;

#[cfg(feature = "flecs_metrics")]
pub mod metrics;

#[cfg(feature = "flecs_alerts")]
pub mod alerts;

// this is not feature gated to flecs_meta so calling `.meta()` on a component will always work despite meta being disabled.
pub trait Meta<Component> {
    fn meta(component: flecs_ecs::core::Component<Component>);
}

impl<T: Meta<T>> crate::core::Component<'_, T> {
    pub fn meta(self) -> Self {
        #[cfg(feature = "flecs_meta")]
        {
            T::meta(self);
        }
        self
    }
}

#[allow(unused_macros)]
macro_rules! create_pre_registered_extern_component {
    ($struct_name:ident, $static_id:ident) => {
        create_pre_registered_extern_component!($struct_name, $static_id, "");
    };
    ($struct_name:ident, $static_id:ident, $doc:tt) => {
        #[derive(Debug, Default)]
        #[allow(clippy::empty_docs)]
        #[doc = $doc]
        pub struct $struct_name;

        impl From<$struct_name> for flecs_ecs::core::Entity {
            #[inline]
            fn from(_view: $struct_name) -> Self {
                flecs_ecs::core::Entity(unsafe { $static_id })
            }
        }

        impl Deref for $struct_name {
            type Target = u64;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                unsafe { &*addr_of!($static_id) }
            }
        }

        impl PartialEq<u64> for $struct_name {
            #[inline]
            fn eq(&self, other: &u64) -> bool {
                unsafe { $static_id == *other }
            }
        }

        impl PartialEq<$struct_name> for u64 {
            #[inline]
            fn eq(&self, _other: &$struct_name) -> bool {
                *self == unsafe { $static_id }
            }
        }

        impl ComponentInfo for $struct_name {
            const IS_GENERIC: bool = false;
            const IS_ENUM: bool = false;
            const IS_TAG: bool = true;
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = false;
            const IMPLS_PARTIAL_ORD: bool = false;
            const IMPLS_PARTIAL_EQ: bool = false;
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
            type TagType =
                flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsATag;
        }

        impl TagComponent for $struct_name {}

        impl ComponentType<Struct> for $struct_name {}

        impl ComponentId for $struct_name {
            type UnderlyingType = $struct_name;
            type UnderlyingEnumType = NoneEnum;
            type UnderlyingTypeOfEnum = NoneEnum;

            fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl WorldProvider<'a>,
            ) -> sys::ecs_entity_t {
                unsafe { $static_id }
            }

            fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl WorldProvider<'a>,
                _name: &str,
            ) -> sys::ecs_entity_t {
                unsafe { $static_id }
            }

            fn is_registered_with_world<'a>(_: impl WorldProvider<'a>) -> bool {
                true
            }

            fn entity_id<'a>(_world: impl WorldProvider<'a>) -> sys::ecs_id_t {
                unsafe { $static_id }
            }

            #[inline(always)]
            fn index() -> u32 {
                static INDEX: core::sync::atomic::AtomicU32 =
                    core::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }
        }

        impl InternalComponentHooks for $struct_name {}

        impl OnComponentRegistration for $struct_name {}
    };
}

#[allow(unused_imports)]
pub(crate) use create_pre_registered_extern_component;

use crate::core::*;
use crate::sys;

#[macro_export]
macro_rules! impl_component_traits_primitive_type {
    ($name:ident, $id:ident) => {
        impl FlecsConstantId for $name {
            const ID: u64 = $id;
        }
        impl DataComponent for $name {}

        impl ComponentType<flecs_ecs::core::Struct> for $name {}

        impl ComponentInfo for $name {
            const IS_GENERIC: bool = false;
            const IS_ENUM: bool = false;
            const IS_TAG: bool = false;
            type TagType = FlecsFirstIsNotATag;
            const IMPLS_CLONE: bool = true;
            const IMPLS_DEFAULT: bool = false;
            const IMPLS_PARTIAL_ORD: bool = true;
            const IMPLS_PARTIAL_EQ: bool = true;
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
        }

        impl ComponentId for $name {
            type UnderlyingType = $name;
            type UnderlyingEnumType = NoneEnum;
            type UnderlyingTypeOfEnum = NoneEnum;

            #[inline(always)]
            fn index() -> u32 {
                static INDEX: core::sync::atomic::AtomicU32 =
                    core::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }

            fn __register_lifecycle_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
                register_lifecycle_actions::<$name>(type_hooks);
            }
            fn __register_default_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
                register_ctor_lifecycle_actions::<$name>(type_hooks);
            }
            fn __register_clone_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
                register_copy_lifecycle_action::<$name>(type_hooks);
            }
            fn __register_compare_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
                register_partial_ord_lifecycle_action::<$name>(type_hooks);
            }
            fn __register_equals_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
                register_partial_eq_lifecycle_action::<$name>(type_hooks);
            }

            fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl WorldProvider<'a>,
            ) -> sys::ecs_entity_t {
                $id
            }

            fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl WorldProvider<'a>,
                _name: &str,
            ) -> sys::ecs_entity_t {
                $id
            }

            fn is_registered_with_world<'a>(_: impl WorldProvider<'a>) -> bool {
                true
            }

            fn entity_id<'a>(_world: impl WorldProvider<'a>) -> sys::ecs_id_t {
                $id
            }
        }

        impl InternalComponentHooks for $name {}

        impl OnComponentRegistration for $name {}
    };
}

#[cfg(any(feature = "flecs_meta", not(feature = "flecs_rust_no_enum_reflection")))]
impl_component_traits_primitive_type!(u8, ECS_U8_T);
#[cfg(any(feature = "flecs_meta", not(feature = "flecs_rust_no_enum_reflection")))]
impl_component_traits_primitive_type!(u16, ECS_U16_T);
#[cfg(any(feature = "flecs_meta", not(feature = "flecs_rust_no_enum_reflection")))]
impl_component_traits_primitive_type!(u32, ECS_U32_T);
#[cfg(any(feature = "flecs_meta", not(feature = "flecs_rust_no_enum_reflection")))]
impl_component_traits_primitive_type!(u64, ECS_U64_T);
#[cfg(any(feature = "flecs_meta", not(feature = "flecs_rust_no_enum_reflection")))]
impl_component_traits_primitive_type!(usize, ECS_UPTR_T);
#[cfg(any(feature = "flecs_meta", not(feature = "flecs_rust_no_enum_reflection")))]
impl_component_traits_primitive_type!(i8, ECS_I8_T);
#[cfg(any(feature = "flecs_meta", not(feature = "flecs_rust_no_enum_reflection")))]
impl_component_traits_primitive_type!(i16, ECS_I16_T);
//underlying enum type should impl it for `fn to_constant`
impl_component_traits_primitive_type!(i32, ECS_I32_T);
#[cfg(any(feature = "flecs_meta", not(feature = "flecs_rust_no_enum_reflection")))]
impl_component_traits_primitive_type!(i64, ECS_I64_T);
