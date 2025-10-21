//! Type `()` implementation as a Flecs component

use crate::core::{ComponentInfo, InternalComponentHooks, OnComponentRegistration};
use crate::sys;

impl InternalComponentHooks for () {}

impl OnComponentRegistration for () {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentId for () {
    type UnderlyingType = ();
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    type UnderlyingTypeOfEnum = flecs_ecs::core::component_registration::NoneEnum;

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }

    fn __register_lifecycle_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<()>(type_hooks);
    }

    fn __register_default_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        const IMPLS_DEFAULT: bool = <() as ComponentInfo>::IMPLS_DEFAULT;

        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,()>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >( type_hooks);
        }
    }

    fn __register_clone_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        const IMPLS_CLONE: bool = <() as ComponentInfo>::IMPLS_CLONE;

        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,()>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >( type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<()>(
                type_hooks,
            );
        }
    }
}

impl flecs_ecs::core::TagComponent for () {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for () {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo for () {
    const IS_GENERIC: bool = false;
    const IS_ENUM: bool = false;
    const IS_TAG: bool = true;
    const IMPLS_CLONE: bool = { flecs_ecs::core::utility::types::ImplementsClone::<()>::IMPLS };
    const IMPLS_DEFAULT: bool = { flecs_ecs::core::utility::types::ImplementsDefault::<()>::IMPLS };
    const IMPLS_PARTIAL_EQ: bool =
        { flecs_ecs::core::utility::types::ImplementsPartialEq::<()>::IMPLS };
    const IMPLS_PARTIAL_ORD: bool =
        { flecs_ecs::core::utility::types::ImplementsPartialOrd::<()>::IMPLS };
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
    type TagType = flecs_ecs::core::component_registration::registration_traits::FlecsIsATag;
}
