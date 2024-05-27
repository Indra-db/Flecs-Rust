use std::{ops::Deref, sync::OnceLock};

use crate::core::*;

pub trait FlecsTrait {}

#[macro_export]
macro_rules! create_pre_registered_component {
    ($struct_name:ident, $const_name:ident) => {
        #[derive(Debug, Default)]
        pub struct $struct_name;

        impl FlecsConstantId for $struct_name {
            const ID: u64 = $const_name;
        }

        impl FlecsTrait for $struct_name {}

        impl Deref for $struct_name {
            type Target = u64;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &Self::ID
            }
        }

        impl ComponentInfo for $struct_name {
            const IS_ENUM: bool = false;
            const IS_TAG: bool = true;
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = false;
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
            type TagType =
                flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsATag;
        }

        impl EmptyComponent for $struct_name {}

        impl ComponentType<Struct> for $struct_name {}

        impl ComponentId for $struct_name {
            type UnderlyingType = $struct_name;
            type UnderlyingEnumType = NoneEnum;

            fn register_explicit<'a>(_world: impl IntoWorld<'a>) {}

            fn register_explicit_named<'a>(_world: impl IntoWorld<'a>, _name: &str) -> EntityT {
                $const_name
            }

            fn is_registered() -> bool {
                true
            }

            fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
                true
            }

            fn get_id<'a>(_world: impl IntoWorld<'a>) -> IdT {
                $const_name
            }

            unsafe fn get_id_unchecked() -> IdT {
                $const_name
            }

            fn __get_once_lock_data() -> &'static OnceLock<IdComponent> {
                static ONCE_LOCK: OnceLock<IdComponent> = OnceLock::new();
                &ONCE_LOCK
            }
        }
    };
}

// Term id flags
create_pre_registered_component!(Self_, ECS_SELF);
create_pre_registered_component!(Up, ECS_UP);
create_pre_registered_component!(Trav, ECS_TRAV);
create_pre_registered_component!(Cascade, ECS_CASCADE);
create_pre_registered_component!(Desc, ECS_DESC);
create_pre_registered_component!(IsVariable, ECS_IS_VARIABLE);
create_pre_registered_component!(IsEntity, ECS_IS_ENTITY);
create_pre_registered_component!(IsName, ECS_IS_NAME);
create_pre_registered_component!(TraverseFlags, ECS_TRAVERSE_FLAGS);
create_pre_registered_component!(TermRefFlags, ECS_TERM_REF_FLAGS);

pub mod term_flags {
    use super::*;
    create_pre_registered_component!(MatchAny, MATCH_ANY);
    create_pre_registered_component!(MatchAnySrc, MATCH_ANY_SRC);
    create_pre_registered_component!(Transitive, TRANSITIVE);
    create_pre_registered_component!(Reflexive, REFLEXIVE);
    create_pre_registered_component!(IdInherited, ID_INHERITED);
    create_pre_registered_component!(IsTrivial, IS_TRIVIAL);
    create_pre_registered_component!(NoData, NO_DATA);
    create_pre_registered_component!(IsCacheable, IS_CACHEABLE);
    create_pre_registered_component!(IsScope, IS_SCOPE);
    create_pre_registered_component!(IsMember, IS_MEMBER);
    create_pre_registered_component!(IsToggle, IS_TOGGLE);
}

pub mod query_flags {
    use super::*;
    create_pre_registered_component!(MatchPrefab, ECS_QUERY_MATCH_PREFAB);
    create_pre_registered_component!(MatchDisabled, ECS_QUERY_MATCH_DISABLED);
    create_pre_registered_component!(MatchEmptyTables, ECS_QUERY_MATCH_EMPTY_TABLES);
    create_pre_registered_component!(NoData, ECS_QUERY_NO_DATA);
    create_pre_registered_component!(IsInstanced, ECS_QUERY_IS_INSTANCED);
    create_pre_registered_component!(AllowUnresolvedByName, ECS_QUERY_ALLOW_UNRESOLVED_BY_NAME);
    create_pre_registered_component!(TableOnly, ECS_QUERY_TABLE_ONLY);
}

pub mod id_flags {
    use super::*;
    // Indicates that the id is a pair.
    create_pre_registered_component!(Pair, ECS_PAIR);
    // Automatically override component when it is inherited
    create_pre_registered_component!(AutoOverride, ECS_AUTO_OVERRIDE);
    // Adds bitset to storage which allows component to be enabled/disabled
    create_pre_registered_component!(Toggle, ECS_TOGGLE);
    // Include all components from entity to which AND is applied
    create_pre_registered_component!(And, ECS_AND);
}

// Builtin component ids
pub type Component = crate::sys::EcsComponent;
pub type Identifier = crate::sys::EcsIdentifier;
pub type Poly = crate::sys::EcsPoly;
pub type DefaultChildComponent = crate::sys::EcsDefaultChildComponent;

crate::impl_component_traits_binding_type_w_id!(Component, ECS_COMPONENT);
crate::impl_component_traits_binding_type_w_id!(Identifier, ECS_IDENTIFIER);
crate::impl_component_traits_binding_type_w_id!(Poly, ECS_POLY);
crate::impl_component_traits_binding_type_w_id!(DefaultChildComponent, ECS_DEFAULT_CHILD_COMPONENT);

// Poly target components
create_pre_registered_component!(Query, ECS_QUERY);
create_pre_registered_component!(Observer, ECS_OBSERVER);

// Core scopes & entities
create_pre_registered_component!(EcsWorld, ECS_WORLD);
create_pre_registered_component!(Flecs, ECS_FLECS);
create_pre_registered_component!(FlecsCore, ECS_FLECS_CORE);
create_pre_registered_component!(FlecsInternals, ECS_FLECS_INTERNALS);
create_pre_registered_component!(EcsModule, ECS_MODULE);
create_pre_registered_component!(Private, ECS_PRIVATE);
create_pre_registered_component!(Prefab, ECS_PREFAB);
create_pre_registered_component!(Disabled, ECS_DISABLED);
create_pre_registered_component!(NotQueryable, ECS_NOT_QUERYABLE);
create_pre_registered_component!(SlotOf, ECS_SLOT_OF);
create_pre_registered_component!(Flag, ECS_FLAG);
create_pre_registered_component!(Monitor, ECS_MONITOR);
create_pre_registered_component!(Empty, ECS_EMPTY);

// Component traits
create_pre_registered_component!(Wildcard, ECS_WILDCARD);
create_pre_registered_component!(Any, ECS_ANY);
create_pre_registered_component!(This_, ECS_THIS);
create_pre_registered_component!(Variable, ECS_VARIABLE);
create_pre_registered_component!(Transitive, ECS_TRANSITIVE);
create_pre_registered_component!(Reflexive, ECS_REFLEXIVE);
create_pre_registered_component!(Symmetric, ECS_SYMMETRIC);
create_pre_registered_component!(Final, ECS_FINAL);
create_pre_registered_component!(DontInherit, ECS_DONT_INHERIT);
create_pre_registered_component!(PairIsTag, ECS_PAIR_IS_TAG);
create_pre_registered_component!(Exclusive, ECS_EXCLUSIVE);
create_pre_registered_component!(Acyclic, ECS_ACYCLIC);
create_pre_registered_component!(Traversable, ECS_TRAVERSABLE);
create_pre_registered_component!(With, ECS_WITH);
create_pre_registered_component!(OneOf, ECS_ONE_OF);
create_pre_registered_component!(CanToggle, ECS_CAN_TOGGLE);

// OnInstantiate traits
create_pre_registered_component!(OnInstantiate, ECS_ON_INSTANTIATE);
create_pre_registered_component!(Override, ECS_OVERRIDE);
create_pre_registered_component!(Inherit, ECS_INHERIT);

// OnDelete/OnDeleteTarget traits
create_pre_registered_component!(OnDelete, ECS_ON_DELETE);
create_pre_registered_component!(OnDeleteTarget, ECS_ON_DELETE_TARGET);
create_pre_registered_component!(Remove, ECS_REMOVE);
create_pre_registered_component!(Delete, ECS_DELETE);
create_pre_registered_component!(Panic, ECS_PANIC);

// Builtin relationships
create_pre_registered_component!(ChildOf, ECS_CHILD_OF);
create_pre_registered_component!(IsA, ECS_IS_A);
create_pre_registered_component!(DependsOn, ECS_DEPENDS_ON);

// Identifier tags
create_pre_registered_component!(Name, ECS_NAME);
create_pre_registered_component!(Symbol, ECS_SYMBOL);
create_pre_registered_component!(Alias, ECS_ALIAS);

// Events
create_pre_registered_component!(OnAdd, ECS_ON_ADD);
create_pre_registered_component!(OnRemove, ECS_ON_REMOVE);
create_pre_registered_component!(OnSet, ECS_ON_SET);
create_pre_registered_component!(Unset, ECS_UNSET);
create_pre_registered_component!(OnTableCreate, ECS_ON_TABLE_CREATE);
create_pre_registered_component!(OnTableDelete, ECS_ON_TABLE_DELETE);
create_pre_registered_component!(OnTableEmpty, ECS_ON_TABLE_EMPTY);
create_pre_registered_component!(OnTableFill, ECS_ON_TABLE_FILL);

// System
#[cfg(feature = "flecs_system")]
pub mod system {
    use super::*;
    pub type TickSource = crate::sys::EcsTickSource;
    crate::impl_component_traits_binding_type_w_id!(TickSource, ECS_TICK_SOURCE);

    create_pre_registered_component!(System, ECS_SYSTEM);
}

#[cfg(feature = "flecs_timer")]
pub mod timer {
    use super::*;

    pub type Timer = crate::sys::EcsTimer;
    crate::impl_component_traits_binding_type_w_id!(Timer, ECS_TIMER);

    pub type RateFilter = crate::sys::EcsRateFilter;
    crate::impl_component_traits_binding_type_w_id!(RateFilter, ECS_RATE_FILTER);
}

create_pre_registered_component!(Sparse, ECS_SPARSE);
create_pre_registered_component!(Union, ECS_UNION);

// Builtin predicate ids (used by rule engine)
create_pre_registered_component!(PredEq, ECS_PRED_EQ);
create_pre_registered_component!(PredMatch, ECS_PRED_MATCH);
create_pre_registered_component!(PredLookup, ECS_PRED_LOOKUP);
create_pre_registered_component!(ScopeOpen, ECS_SCOPE_OPEN);
create_pre_registered_component!(ScopeClose, ECS_SCOPE_CLOSE);

// Systems
#[cfg(feature = "flecs_system")]
pub mod pipeline {
    use super::*;
    create_pre_registered_component!(Pipeline, ECS_PIPELINE);
    create_pre_registered_component!(OnStart, ECS_ON_START);
    create_pre_registered_component!(PreFrame, ECS_PRE_FRAME);
    create_pre_registered_component!(OnLoad, ECS_ON_LOAD);
    create_pre_registered_component!(PostLoad, ECS_POST_LOAD);
    create_pre_registered_component!(PreUpdate, ECS_PRE_UPDATE);
    create_pre_registered_component!(OnUpdate, ECS_ON_UPDATE);
    create_pre_registered_component!(OnValidate, ECS_ON_VALIDATE);
    create_pre_registered_component!(PostUpdate, ECS_POST_UPDATE);
    create_pre_registered_component!(PreStore, ECS_PRE_STORE);
    create_pre_registered_component!(OnStore, ECS_ON_STORE);
    create_pre_registered_component!(PostFrame, ECS_POST_FRAME);
    create_pre_registered_component!(Phase, ECS_PHASE);
}

#[cfg(feature = "flecs_meta")]
pub mod meta {
    use super::*;
    // Meta primitive components (don't use low ids to save id space)
    create_pre_registered_component!(Bool, ECS_BOOL_T);
    create_pre_registered_component!(Char, ECS_CHAR_T);
    create_pre_registered_component!(Byte, ECS_BYTE_T);
    create_pre_registered_component!(U8, ECS_U8_T);
    create_pre_registered_component!(U32, ECS_U32_T);
    create_pre_registered_component!(U64, ECS_U64_T);
    create_pre_registered_component!(UPtr, ECS_UPTR_T);
    create_pre_registered_component!(I8, ECS_I8_T);
    create_pre_registered_component!(I16, ECS_I16_T);
    create_pre_registered_component!(I32, ECS_I32_T);
    create_pre_registered_component!(I64, ECS_I64_T);
    create_pre_registered_component!(IPtr, ECS_IPTR_T);
    create_pre_registered_component!(F32, ECS_F32_T);
    create_pre_registered_component!(F64, ECS_F64_T);
    create_pre_registered_component!(String, ECS_STRING_T);
    create_pre_registered_component!(Entity, ECS_ENTITY_T);

    // Meta type components
    create_pre_registered_component!(Type, ECS_META_TYPE);
    create_pre_registered_component!(TypeSerialized, ECS_META_TYPE_SERIALIZED);
    create_pre_registered_component!(Primitive, ECS_PRIMITIVE);
    create_pre_registered_component!(Enum, ECS_ENUM);
    create_pre_registered_component!(Bitmask, ECS_BITMASK);
    create_pre_registered_component!(Member, ECS_MEMBER);
    create_pre_registered_component!(StructT, ECS_STRUCT);
    create_pre_registered_component!(Array, ECS_ARRAY);
    create_pre_registered_component!(Vector, ECS_VECTOR);
    create_pre_registered_component!(Opaque, ECS_OPAQUE);
    create_pre_registered_component!(Unit, ECS_UNIT);
    create_pre_registered_component!(UnitPrefix, ECS_UNIT_PREFIX);
    create_pre_registered_component!(Constant, ECS_CONSTANT);
    create_pre_registered_component!(Quantity, ECS_QUANTITY);
}

// Doc module components
pub mod doc {
    use super::*;
    create_pre_registered_component!(Description, ECS_DOC_DESCRIPTION);
    create_pre_registered_component!(Brief, ECS_DOC_BRIEF);
    create_pre_registered_component!(Detail, ECS_DOC_DETAIL);
    create_pre_registered_component!(Link, ECS_DOC_LINK);
    create_pre_registered_component!(Color, ECS_DOC_COLOR);
}

#[cfg(feature = "flecs_rest")]
pub mod rest {
    use super::*;
    // REST module components
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Rest {
        #[doc = "< Port of server (optional, default = 27750)"]
        pub port: u16,
        #[doc = "< Interface address (optional, default = 0.0.0.0)"]
        pub ipaddr: *mut ::std::os::raw::c_char,
        pub impl_: *mut ::std::os::raw::c_void,
    }

    impl Default for Rest {
        fn default() -> Self {
            Self {
                port: Default::default(),
                ipaddr: std::ptr::null_mut::<std::os::raw::c_char>(),
                impl_: std::ptr::null_mut::<std::os::raw::c_void>(),
            }
        }
    }

    crate::impl_component_traits_binding_type_w_id!(Rest, ECS_REST);
}

// default component for event API

impl flecs_ecs::core::EmptyComponent for () {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for () {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo for () {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = true;
    const IMPLS_CLONE: bool = { flecs_ecs::core::utility::types::ImplementsClone::<()>::IMPLS };
    const IMPLS_DEFAULT: bool = { flecs_ecs::core::utility::types::ImplementsDefault::<()>::IMPLS };
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
    type TagType = flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsATag;
}

impl flecs_ecs::core::component_registration::registration_traits::ComponentId for () {
    type UnderlyingType = ();
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
        static ONCE_LOCK: std::sync::OnceLock<flecs_ecs::core::IdComponent> =
            std::sync::OnceLock::new();
        &ONCE_LOCK
    }

    fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<()>(type_hooks);
    }

    fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        const IMPLS_DEFAULT: bool = <() as ComponentInfo>::IMPLS_DEFAULT;

        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,()>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >( type_hooks);
        }
    }

    fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
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
