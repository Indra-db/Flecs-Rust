//! Periodically tracks statistics for the world and systems.

use crate::core::{InternalComponentHooks, OnComponentRegistration, World, WorldProvider};
use crate::sys;

#[cfg(feature = "flecs_module")]
use super::super::module::Module;

/// Component that stores world statistics
pub type WorldStats = sys::EcsWorldStats;
/// Component that stores system/pipeline statistics
pub type PipelineStats = sys::EcsPipelineStats;
/// Component with world summary stats
pub type WorldSummary = sys::EcsWorldSummary;
/// Component with system stats
pub type SystemStats = sys::EcsSystemStats;

/// Memory statistics components
pub type EntitiesMemory = sys::ecs_entities_memory_t;
pub type ComponentIndexMemory = sys::ecs_component_index_memory_t;
pub type QueryMemory = sys::ecs_query_memory_t;
pub type ComponentMemory = sys::ecs_component_memory_t;
pub type TableMemory = sys::ecs_table_memory_t;
pub type MiscMemory = sys::ecs_misc_memory_t;
pub type TableHistogram = sys::ecs_table_histogram_t;
pub type AllocatorMemory = sys::ecs_allocator_memory_t;
pub type WorldMemory = sys::EcsWorldMemory;

#[derive(Debug, Clone, Copy, Default)]
pub struct Stats;

#[cfg(feature = "flecs_module")]
impl Module for Stats {
    fn module(world: &World) {
        #[cfg(feature = "flecs_units")]
        world.import::<super::super::units::Units>();
        unsafe { sys::FlecsStatsImport(world.ptr_mut()) };
        world.component::<WorldSummary>();
        world.component::<WorldStats>();
        world.component::<SystemStats>();
        world.component::<PipelineStats>();
    }
}

///////////////////////////
// trait implementations
///////////////////////////

macro_rules! impl_stats_component {
    ($type:ty) => {
        impl flecs_ecs::core::DataComponent for $type {}

        impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for $type {}

        impl flecs_ecs::core::component_registration::ComponentInfo for $type {
            const IS_GENERIC: bool = false;
            const IS_ENUM: bool = false;
            const IS_TAG: bool = false;
            type TagType = flecs_ecs::core::component_registration::FlecsNotATag;
            const IMPLS_CLONE: bool = true;
            const IMPLS_DEFAULT: bool = false;
            const IMPLS_PARTIAL_ORD: bool = false;
            const IMPLS_PARTIAL_EQ: bool = false;
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
        }

        impl flecs_ecs::core::component_registration::ComponentId for $type
        where
            Self: 'static,
        {
            type UnderlyingType = $type;
            type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
            type UnderlyingTypeOfEnum = flecs_ecs::core::component_registration::NoneEnum;

            #[inline(always)]
            fn index() -> u32 {
                static INDEX: core::sync::atomic::AtomicU32 =
                    core::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }

            fn __register_lifecycle_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
                flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<$type>(type_hooks);
            }

            fn __register_default_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_DEFAULT: bool = <$type>::IMPLS_DEFAULT;
                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,$type>as flecs_ecs::core::component_registration::FlecsDefaultType> ::Type, >(type_hooks);
                }
            }

            fn __register_clone_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_CLONE: bool = <$type>::IMPLS_CLONE;
                if IMPLS_CLONE {
                    flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,$type>as flecs_ecs::core::component_registration::FlecsCloneType> ::Type, >(type_hooks);
                } else {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<
                        $type,
                    >(type_hooks);
                }
            }
        }

        impl InternalComponentHooks for $type {}

        impl OnComponentRegistration for $type {}
    };
}

// Apply the macro to all stats component types
impl_stats_component!(sys::EcsWorldStats);
impl_stats_component!(sys::EcsPipelineStats);
impl_stats_component!(sys::EcsWorldSummary);
impl_stats_component!(sys::EcsSystemStats);
impl_stats_component!(sys::ecs_entities_memory_t);
impl_stats_component!(sys::ecs_component_index_memory_t);
impl_stats_component!(sys::ecs_query_memory_t);
impl_stats_component!(sys::ecs_component_memory_t);
impl_stats_component!(sys::ecs_table_memory_t);
impl_stats_component!(sys::ecs_misc_memory_t);
impl_stats_component!(sys::ecs_table_histogram_t);
impl_stats_component!(sys::ecs_allocator_memory_t);
impl_stats_component!(sys::EcsWorldMemory);

///////////////////////////////////////////////
///////////////////////////////////////////////

impl flecs_ecs::core::DataComponent for Stats {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for Stats {}

impl flecs_ecs::core::component_registration::ComponentInfo for Stats {
    const IS_GENERIC: bool = false;
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType = flecs_ecs::core::component_registration::FlecsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = true;
    const IMPLS_PARTIAL_ORD: bool = false;
    const IMPLS_PARTIAL_EQ: bool = false;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}
impl flecs_ecs::core::component_registration::ComponentId for Stats
where
    Self: 'static,
{
    type UnderlyingType = Stats;
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    type UnderlyingTypeOfEnum = flecs_ecs::core::component_registration::NoneEnum;

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }
    fn __register_lifecycle_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<Stats>(type_hooks);
    }
    fn __register_default_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        use flecs_ecs::core::component_registration::ComponentInfo;
        const IMPLS_DEFAULT: bool = Stats::IMPLS_DEFAULT;
        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,Stats>as flecs_ecs::core::component_registration::FlecsDefaultType> ::Type, >(type_hooks);
        }
    }
    fn __register_clone_hooks(type_hooks: &mut sys::ecs_type_hooks_t) {
        use flecs_ecs::core::component_registration::ComponentInfo;
        const IMPLS_CLONE: bool = Stats::IMPLS_CLONE;
        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,Stats>as flecs_ecs::core::component_registration::FlecsCloneType> ::Type, >(type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<Stats>(
                type_hooks,
            );
        }
    }

    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        world: impl WorldProvider<'a>,
    ) -> sys::ecs_entity_t {
        Self::__register_or_get_id_named::<MANUAL_REGISTRATION_CHECK>(world, "flecs::stats")
    }
}

impl InternalComponentHooks for Stats {}

impl OnComponentRegistration for Stats {}
