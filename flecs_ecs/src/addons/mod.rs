//! Flecs addons extend the core ECS with additional functionality.
//!
//! This module provides access to all optional Flecs addons, which can be enabled
//! through Cargo features. Each addon is designed to be modular and can be used
//! independently or in combination with others.
//!
//! # Available Addons
//!
//! ## Application & Execution
//!
//! - **[`app`]** - Application main loop wrapper with hooks for modules
//!   - Feature: `flecs_app`
//!   - Used for: Managing application lifecycle, integration with emscripten/WebGL
//!
//! - **[`system`]** - Systems for automatic query iteration
//!   - Feature: `flecs_system`
//!   - Used for: Game logic, data processing, scheduled operations
//!
//! - **[`pipeline`]** - System ordering and scheduling
//!   - Feature: `flecs_pipeline`
//!   - Used for: Multi-phase execution, dependency management
//!
//! - **[`timer`]** - Periodic and one-shot timers
//!   - Feature: `flecs_timer`
//!   - Used for: Time-based triggers, cooldowns, scheduled events
//!
//! ## Organization & Structure
//!
//! - **[`module`]** - Reusable code units with automatic namespacing
//!   - Feature: `flecs_module`
//!   - Used for: Organizing code into reusable packages
//!
//! ## Reflection & Serialization
//!
//! - **[`meta`]** - Component reflection and introspection
//!   - Feature: `flecs_meta`
//!   - Used for: Runtime type information, generic serialization
//!
//! - **[`json`]** - JSON serialization/deserialization
//!   - Feature: `flecs_json`
//!   - Used for: Data exchange, persistence, REST API
//!
//! - **[`script`]** - Flecs script language support
//!   - Feature: `flecs_script`
//!   - Used for: Declarative entity/component definition, data-driven design
//!
//! ## Monitoring & Debugging
//!
//! - **[`doc`]** - Entity documentation with brief/detailed descriptions
//!   - Feature: `flecs_doc`
//!   - Used for: Documenting entities, tooling integration
//!
//! - **[`stats`]** - High-resolution performance statistics
//!   - Feature: `flecs_stats`
//!   - Used for: Profiling, performance monitoring, explorer integration
//!
//! - **[`metrics`]** - Extract and track component metrics
//!   - Feature: `flecs_metrics`
//!   - Used for: Monitoring component values, counters, gauges
//!
//! - **[`alerts`]** - Condition-based alerts
//!   - Feature: `flecs_alerts`
//!   - Used for: Detecting problematic states, validation
//!
//! ## Remote Access
//!
//! - **REST API** - HTTP server for remote data access
//!   - Feature: `flecs_rest`
//!   - Used for: Web-based UIs, remote inspection, Flecs Explorer
//!
//! ## Utilities
//!
//! - **[`units`]** - Standard measurement units (SI units, etc.)
//!   - Feature: `flecs_units`
//!   - Used for: Quantity types with proper units
//!
//! # Feature Flags
//!
//! Each addon is gated behind a Cargo feature flag. Enable the features you need
//! in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! flecs_ecs = { version = "0.1", features = [
//!     "flecs_app",
//!     "flecs_system",
//!     "flecs_pipeline",
//!     "flecs_timer",
//!     "flecs_module",
//!     "flecs_meta",
//!     "flecs_json",
//!     "flecs_doc",
//!     "flecs_stats",
//!     "flecs_metrics",
//!     "flecs_alerts",
//!     "flecs_rest",
//!     "flecs_script",
//!     "flecs_units",
//! ] }
//! ```
//!
//! Or use the `flecs_default` feature to enable commonly used addons:
//!
//! ```toml
//! [dependencies]
//! flecs_ecs = { version = "0.1", features = ["flecs_default"] }
//! ```
//!
//! # Quick Start Example
//!
//! ```
//! use flecs_ecs::prelude::*;
//!
//! #[derive(Component)]
//! struct Position { x: f32, y: f32 }
//!
//! #[derive(Component)]
//! struct Velocity { x: f32, y: f32 }
//!
//! let world = World::new();
//!
//! // System addon: process entities each frame
//! world.system::<(&Velocity, &mut Position)>()
//!     .each(|(vel, pos)| {
//!         pos.x += vel.x;
//!         pos.y += vel.y;
//!     });
//!
//! // Timer addon: run at intervals
//! world.system::<()>()
//!     .set_interval(1.0)
//!     .run(|_it| println!("One second elapsed"));
//!
//! // App addon: run the main loop
//! world.app()
//!     .set_target_fps(60.0)
//!     .run();
//! ```
//!
//! # See also
//!
//! - [Flecs docs](https://www.flecs.dev/flecs/md_docs_2Quickstart.html)
//! - [Examples](https://github.com/Indra-db/Flecs-Rust/tree/main/flecs_ecs/examples/flecs) - Code examples

#[cfg(feature = "flecs_app")]
pub mod app;
#[cfg(feature = "flecs_app")]
pub use app::*;

#[cfg(feature = "flecs_doc")]
pub mod doc;
#[cfg(feature = "flecs_doc")]
pub use doc::*;

#[cfg(feature = "flecs_module")]
pub mod module;
#[cfg(feature = "flecs_module")]
pub use module::*;

#[cfg(feature = "flecs_system")]
pub mod system;
#[cfg(feature = "flecs_system")]
pub use system::*;

#[cfg(feature = "flecs_pipeline")]
pub mod pipeline;
#[cfg(feature = "flecs_pipeline")]
pub use pipeline::*;

#[cfg(feature = "flecs_stats")]
pub mod stats;
#[cfg(feature = "flecs_stats")]
pub use stats::*;

#[cfg(feature = "flecs_timer")]
pub mod timer;
#[cfg(feature = "flecs_timer")]
pub use timer::*;

#[cfg(feature = "flecs_meta")]
pub mod meta;
#[cfg(feature = "flecs_meta")]
pub use meta::*;

#[cfg(feature = "flecs_script")]
pub mod script;
#[cfg(feature = "flecs_script")]
pub use script::*;

#[cfg(feature = "flecs_json")]
pub mod json;
#[cfg(feature = "flecs_json")]
pub use json::*;

#[cfg(feature = "flecs_units")]
pub mod units;

#[cfg(feature = "flecs_metrics")]
pub mod metrics;

#[cfg(feature = "flecs_alerts")]
pub mod alerts;
#[cfg(feature = "flecs_alerts")]
pub use alerts::*;

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
                unsafe { &*core::ptr::addr_of!($static_id) }
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
            type TagType = flecs_ecs::core::component_registration::FlecsIsATag;
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
                unsafe { $static_id }
            }

            fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl crate::core::WorldProvider<'a>,
                _name: &str,
            ) -> sys::ecs_entity_t {
                unsafe { $static_id }
            }

            fn is_registered_with_world<'a>(_: impl crate::core::WorldProvider<'a>) -> bool {
                true
            }

            fn entity_id<'a>(_world: impl crate::core::WorldProvider<'a>) -> sys::ecs_id_t {
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
            type TagType = FlecsNotATag;
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
