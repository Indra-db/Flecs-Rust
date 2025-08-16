use flecs_ecs::prelude::*;
use flecs_ecs::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::string::String;

use crate::impl_component_traits_primitive_type;

//uint and int types exposed in addons/mod.rs for enum registration
impl_component_traits_primitive_type!(bool, ECS_BOOL_T);
impl_component_traits_primitive_type!(char, ECS_CHAR_T);
impl_component_traits_primitive_type!(isize, ECS_IPTR_T);
impl_component_traits_primitive_type!(f32, ECS_F32_T);
impl_component_traits_primitive_type!(f64, ECS_F64_T);
impl_component_traits_primitive_type!(Entity, ECS_ENTITY_T);
//impl_component_traits_primitive_type!(String, ECS_STRING_T);

impl FlecsConstantId for EntityView<'static> {
    const ID: u64 = ECS_ENTITY_T;
}

unsafe impl Send for EntityView<'static> {}

unsafe impl Sync for EntityView<'static> {}

impl DataComponent for EntityView<'static> {}

impl ComponentType<flecs_ecs::core::Struct> for EntityView<'static> {}

impl ComponentInfo for EntityView<'static> {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType = FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool =
        { flecs_ecs::core::utility::types::ImplementsDefault::<EntityView<'static>>::IMPLS };
    const IMPLS_PARTIAL_EQ: bool =
        { flecs_ecs::core::utility::types::ImplementsPartialEq::<EntityView<'static>>::IMPLS };
    const IMPLS_PARTIAL_ORD: bool =
        { flecs_ecs::core::utility::types::ImplementsPartialOrd::<EntityView<'static>>::IMPLS };
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
    const IS_GENERIC: bool = false;
}

impl ComponentId for EntityView<'static> {
    type UnderlyingType = EntityView<'static>;
    type UnderlyingEnumType = NoneEnum;
    type UnderlyingTypeOfEnum = NoneEnum;

    fn __register_lifecycle_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        register_lifecycle_actions::<EntityView<'static>>(type_hooks);
    }
    fn __register_default_hooks(_type_hooks: &mut sys::ecs_type_hooks_t) {}

    fn __register_clone_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        register_copy_lifecycle_action::<EntityView<'static>>(type_hooks);
    }

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }

    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl WorldProvider<'a>,
    ) -> sys::ecs_entity_t {
        ECS_ENTITY_T
    }

    fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl WorldProvider<'a>,
        _name: &str,
    ) -> sys::ecs_entity_t {
        ECS_ENTITY_T
    }

    fn is_registered_with_world<'a>(_: impl WorldProvider<'a>) -> bool {
        true
    }

    fn entity_id<'a>(_world: impl WorldProvider<'a>) -> sys::ecs_id_t {
        ECS_ENTITY_T
    }
}

impl InternalComponentHooks for EntityView<'static> {}

impl OnComponentRegistration for EntityView<'static> {}

// Recursive expansion of Component macro
// =======================================

impl flecs_ecs::core::DataComponent for String {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for String {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo for String {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType =
        flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = { flecs_ecs::core::utility::types::ImplementsClone::<String>::IMPLS };
    const IMPLS_DEFAULT: bool =
        { flecs_ecs::core::utility::types::ImplementsDefault::<String>::IMPLS };
    const IMPLS_PARTIAL_EQ: bool = true;
    const IMPLS_PARTIAL_ORD: bool = true;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
    const IS_GENERIC: bool = false;
}
impl flecs_ecs::core::component_registration::registration_traits::ComponentId for String {
    type UnderlyingType = String;
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    type UnderlyingTypeOfEnum = flecs_ecs::core::component_registration::NoneEnum;

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }

    fn __register_lifecycle_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<String>(type_hooks);
    }
    fn __register_default_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_DEFAULT: bool = String::IMPLS_DEFAULT;
        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,String>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
        }
    }
    fn __register_clone_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_CLONE: bool = String::IMPLS_CLONE;
        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,String>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<String>(
                type_hooks,
            );
        }
    }
}

impl InternalComponentHooks for String {}

impl OnComponentRegistration for String {}
