#![doc(hidden)]
use flecs_ecs_derive::Component;

use super::{
    ComponentId, ComponentInfo, ComponentType, DataComponent, FlecsFirstIsNotATag,
    OnComponentRegistration,
};
use crate::core::{
    ECS_I32_T, Entity, FlecsConstantId, WorldProvider, WorldRef, register_copy_lifecycle_action,
    register_ctor_lifecycle_actions,
};
use flecs_ecs_sys as sys;

pub struct Enum;

pub struct Struct;

#[derive(Component)]
#[repr(C)]
pub enum NoneEnum {
    None = 1,
}

#[derive(Default, Clone)]
pub struct FlecsNoneDefaultDummy;

#[derive(Clone)]
pub struct FlecsNoneCloneDummy;

pub struct ConditionalTypeSelector<const B: bool, T> {
    phantom: core::marker::PhantomData<T>,
}

pub struct ConditionalTypePairSelector<T, First, Second>
where
    First: ComponentId,
    Second: ComponentId,
{
    phantom: core::marker::PhantomData<(T, First, Second)>,
}

impl FlecsConstantId for i32 {
    const ID: u64 = ECS_I32_T;
}
impl DataComponent for i32 {}

impl ComponentType<flecs_ecs::core::Struct> for i32 {}

impl ComponentInfo for i32 {
    const IS_GENERIC: bool = false;
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType = FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = false;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}
impl ComponentId for i32 {
    type UnderlyingType = i32;
    type UnderlyingEnumType = NoneEnum;
type UnderlyingTypeOfEnum = NoneEnum;
    #[inline(always)]
    fn index() -> u32 {
        static INDEX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }
    fn __register_lifecycle_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        crate::core::register_lifecycle_actions::<i32>(type_hooks);
    }
    fn __register_default_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        register_ctor_lifecycle_actions::<i32>(type_hooks);
    }
    fn __register_clone_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        register_copy_lifecycle_action::<i32>(type_hooks);
    }
    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl WorldProvider<'a>,
    ) -> sys::ecs_entity_t {
        ECS_I32_T
    }
    fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl WorldProvider<'a>,
        _name: &str,
    ) -> sys::ecs_entity_t {
        ECS_I32_T
    }
    fn is_registered_with_world<'a>(_: impl WorldProvider<'a>) -> bool {
        true
    }
    fn id<'a>(_world: impl WorldProvider<'a>) -> sys::ecs_id_t {
        ECS_I32_T
    }
}
impl OnComponentRegistration for i32 {
    fn on_component_registration(_world: WorldRef, _component_id: Entity) {}
}
