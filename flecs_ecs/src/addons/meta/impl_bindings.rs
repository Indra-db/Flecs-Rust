use flecs_ecs::prelude::*;
use flecs_ecs::sys;

use super::Opaque;

unsafe impl<T> Send for Opaque<'static, T> {}

unsafe impl<T> Sync for Opaque<'static, T> {}

impl<T: ComponentId> FlecsConstantId for Opaque<'static, T> {
    const ID: u64 = ECS_OPAQUE;
}

impl<T: ComponentId> TagComponent for Opaque<'static, T> {}

impl<T: ComponentId> ComponentType<flecs_ecs::core::Struct> for Opaque<'static, T> {}

impl<T: ComponentId> ComponentInfo for Opaque<'static, T> {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType = FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = true;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
    const IS_GENERIC: bool = true;
}

impl<T: ComponentId> ComponentId for Opaque<'static, T> {
    type UnderlyingType = Opaque<'static, T>;
    type UnderlyingEnumType = NoneEnum;
type UnderlyingTypeOfEnum = NoneEnum;

    fn __register_lifecycle_hooks(_type_hooks: &mut sys::ecs_type_hooks_t) {}
    fn __register_default_hooks(_type_hooks: &mut sys::ecs_type_hooks_t) {}
    fn __register_clone_hooks(_type_hooks: &mut sys::ecs_type_hooks_t) {}

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }

    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl WorldProvider<'a>,
    ) -> sys::ecs_entity_t {
        ECS_OPAQUE
    }

    fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl WorldProvider<'a>,
        _name: &str,
    ) -> sys::ecs_entity_t {
        ECS_OPAQUE
    }

    fn is_registered_with_world<'a>(_: impl WorldProvider<'a>) -> bool {
        true
    }

    fn id<'a>(_world: impl WorldProvider<'a>) -> sys::ecs_id_t {
        ECS_OPAQUE
    }
}

impl<T: ComponentId> OnComponentRegistration for Opaque<'static, T> {
    fn on_component_registration(_world: WorldRef, _component_id: Entity) {}
}
