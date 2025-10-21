use super::*;

// Macro to implement all the necessary traits for a pre-registered component struct.
// Use this when you want to define the struct with custom documentation separately.
macro_rules! impl_pre_registered_component {
    ($struct_name:ident, $const_name:ident) => {
        impl crate::core::utility::traits::FlecsConstantId for $struct_name {
            const ID: u64 = $const_name;
        }

        impl From<$struct_name> for flecs_ecs::core::Entity {
            #[inline]
            fn from(_view: $struct_name) -> Self {
                flecs_ecs::core::Entity(
                    <$struct_name as crate::core::utility::traits::FlecsConstantId>::ID,
                )
            }
        }

        impl Deref for $struct_name {
            type Target = u64;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &<Self as crate::core::utility::traits::FlecsConstantId>::ID
            }
        }

        impl PartialEq<u64> for $struct_name {
            #[inline]
            fn eq(&self, other: &u64) -> bool {
                <Self as crate::core::utility::traits::FlecsConstantId>::ID == *other
            }
        }

        impl PartialEq<$struct_name> for u64 {
            #[inline]
            fn eq(&self, _other: &$struct_name) -> bool {
                *self == <$struct_name as crate::core::utility::traits::FlecsConstantId>::ID
            }
        }

        impl crate::core::ComponentInfo for $struct_name {
            const IS_GENERIC: bool = false;
            const IS_ENUM: bool = false;
            const IS_TAG: bool = true;
            const IMPLS_CLONE: bool = true;
            const IMPLS_DEFAULT: bool = true;
            const IMPLS_PARTIAL_EQ: bool = false;
            const IMPLS_PARTIAL_ORD: bool = false;
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
            type TagType =
                flecs_ecs::core::component_registration::registration_traits::FlecsIsATag;
        }

        impl crate::core::TagComponent for $struct_name {}

        impl crate::core::ComponentType<crate::core::Struct> for $struct_name {}

        impl crate::core::ComponentId for $struct_name {
            type UnderlyingType = $struct_name;
            type UnderlyingEnumType = crate::core::NoneEnum;
            type UnderlyingTypeOfEnum = crate::core::NoneEnum;

            fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl crate::core::WorldProvider<'a>,
            ) -> sys::ecs_entity_t {
                $const_name
            }

            fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl crate::core::WorldProvider<'a>,
                _name: &str,
            ) -> sys::ecs_entity_t {
                $const_name
            }

            fn is_registered_with_world<'a>(_: impl crate::core::WorldProvider<'a>) -> bool {
                true
            }

            fn entity_id<'a>(_world: impl crate::core::WorldProvider<'a>) -> sys::ecs_id_t {
                $const_name
            }

            #[inline(always)]
            fn index() -> u32 {
                static INDEX: core::sync::atomic::AtomicU32 =
                    core::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }
        }

        impl InternalComponentHooks for $struct_name {
            fn internal_on_component_registration(
                _world: crate::core::WorldRef,
                _component_id: crate::core::Entity,
            ) {
            }
        }

        impl OnComponentRegistration for $struct_name {
            fn on_component_registration(
                _world: crate::core::WorldRef,
                _component_id: crate::core::Entity,
            ) {
            }
        }
    };
}

pub(crate) use impl_pre_registered_component;

// Macro to define a pre-registered component struct with all necessary trait implementations.
// This macro creates the struct definition and then calls impl_pre_registered_component to add the implementations.
macro_rules! create_pre_registered_component {
    ($struct_name:ident, $const_name:ident) => {
        create_pre_registered_component!($struct_name, $const_name, "");
    };
    ($struct_name:ident, $const_name:ident, $doc:tt) => {
        #[derive(Debug, Default, Clone)]
        #[allow(clippy::empty_docs)]
        #[doc = $doc]
        pub struct $struct_name;

        impl_pre_registered_component!($struct_name, $const_name);
    };
}

pub(crate) use create_pre_registered_component;

// Component-trait specialization: same as create_pre_registered_component but also
// implements FlecsComponentTrait. Use this for items documented on the
// "Component traits" page so they can be identified at compile time.
macro_rules! create_component_trait {
    ($struct_name:ident, $const_name:ident) => {
        create_component_trait!($struct_name, $const_name, "");
    };
    ($struct_name:ident, $const_name:ident, $doc:tt) => {
        create_pre_registered_component!($struct_name, $const_name, $doc);
        impl crate::core::flecs::FlecsComponentTrait for $struct_name {}
    };
}

pub(crate) use create_component_trait;

// Macro to implement component trait on an existing struct.
// Use this when you want to define the struct with custom documentation separately.
macro_rules! impl_component_trait {
    ($struct_name:ident, $const_name:ident) => {
        impl_pre_registered_component!($struct_name, $const_name);
        impl crate::core::flecs::FlecsComponentTrait for $struct_name {}
    };
}

pub(crate) use impl_component_trait;
