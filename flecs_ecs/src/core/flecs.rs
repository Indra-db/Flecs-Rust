//! Contains flecs traits and pre-registered components.
use std::ops::Deref;

use crate::core::*;
use crate::sys;

//TODO: a lot of these need to be feature gated

pub trait FlecsTrait {}

macro_rules! create_pre_registered_component {
    ($struct_name:ident, $const_name:ident) => {
        create_pre_registered_component!($struct_name, $const_name, "");
    };
    ($struct_name:ident, $const_name:ident, $doc:tt) => {
        #[derive(Debug, Default)]
        #[allow(clippy::empty_docs)]
        #[doc = $doc]
        pub struct $struct_name;

        impl FlecsConstantId for $struct_name {
            const ID: u64 = $const_name;
        }

        impl FlecsTrait for $struct_name {}

        impl From<$struct_name> for flecs_ecs::core::Entity {
            #[inline]
            fn from(_view: $struct_name) -> Self {
                flecs_ecs::core::Entity($struct_name::ID)
            }
        }

        impl Deref for $struct_name {
            type Target = u64;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &Self::ID
            }
        }

        impl PartialEq<u64> for $struct_name {
            #[inline]
            fn eq(&self, other: &u64) -> bool {
                Self::ID == *other
            }
        }

        impl PartialEq<$struct_name> for u64 {
            #[inline]
            fn eq(&self, _other: &$struct_name) -> bool {
                *self == $struct_name::ID
            }
        }

        impl ComponentInfo for $struct_name {
            const IS_GENERIC: bool = false;
            const IS_ENUM: bool = false;
            const IS_TAG: bool = true;
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = false;
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

            fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl WorldProvider<'a>,
            ) -> sys::ecs_entity_t {
                $const_name
            }

            fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl WorldProvider<'a>,
                _name: &str,
            ) -> sys::ecs_entity_t {
                $const_name
            }

            fn is_registered_with_world<'a>(_: impl WorldProvider<'a>) -> bool {
                true
            }

            fn id<'a>(_world: impl WorldProvider<'a>) -> sys::ecs_id_t {
                $const_name
            }

            #[inline(always)]
            fn index() -> u32 {
                static INDEX: std::sync::atomic::AtomicU32 =
                    std::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }
        }
    };
}

// Term id flags
create_pre_registered_component!(Self_, ECS_SELF, "Match on self");
create_pre_registered_component!(Up, ECS_UP, "Match by traversing upwards");
create_pre_registered_component!(
    Trav,
    ECS_TRAV,
    "Match by traversing downwards (derived, cannot be set)"
);
create_pre_registered_component!(
    Cascade,
    ECS_CASCADE,
    "Match by traversing upwards, but iterate in breadth-first order"
);
create_pre_registered_component!(
    Desc,
    ECS_DESC,
    "Combine with Cascade to iterate hierarchy bottom to top"
);
create_pre_registered_component!(IsVariable, ECS_IS_VARIABLE, "Term id is a variable");
create_pre_registered_component!(IsEntity, ECS_IS_ENTITY, "Term id is an entity");
create_pre_registered_component!(
    IsName,
    ECS_IS_NAME,
    "Term id is a name (don't attempt to lookup as entity)"
);
create_pre_registered_component!(
    TraverseFlags,
    ECS_TRAVERSE_FLAGS,
    "all term traversal flags"
);
create_pre_registered_component!(
    TermRefFlags,
    ECS_TERM_REF_FLAGS,
    "all term reference kind flags"
);

/// Term flags discovered & set during query creation.
/// Mostly used internally to store information relevant to queries.
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

/// Query flags discovered & set during query creation.
pub mod query_flags {
    use super::*;
    create_pre_registered_component!(
        MatchPrefab,
        ECS_QUERY_MATCH_PREFAB,
        "Query must match prefabs."
    );
    create_pre_registered_component!(
        MatchDisabled,
        ECS_QUERY_MATCH_DISABLED,
        "Query must match disabled entities."
    );
    create_pre_registered_component!(
        MatchEmptyTables,
        ECS_QUERY_MATCH_EMPTY_TABLES,
        "Query must match empty tables."
    );

    create_pre_registered_component!(
        AllowUnresolvedByName,
        ECS_QUERY_ALLOW_UNRESOLVED_BY_NAME,
        "Query may have unresolved entity identifiers."
    );
    create_pre_registered_component!(
        TableOnly,
        ECS_QUERY_TABLE_ONLY,
        "Query only returns whole tables (ignores toggle/member fields)."
    );
}

pub mod id_flags {
    use super::*;
    create_pre_registered_component!(Pair, ECS_PAIR, "Indicates that the id is a pair.");
    create_pre_registered_component!(
        AutoOverride,
        ECS_AUTO_OVERRIDE,
        "Automatically override component when it is inherited"
    );
    create_pre_registered_component!(
        Toggle,
        ECS_TOGGLE,
        "Adds bitset to storage which allows component to be enabled/disabled"
    );
    create_pre_registered_component!(
        And,
        ECS_AND,
        "Include all components from entity to which AND is applied"
    );
}

// Builtin component ids
pub type Component = crate::sys::EcsComponent;
pub type Identifier = crate::sys::EcsIdentifier;
pub type Poly = crate::sys::EcsPoly;
pub type DefaultChildComponent = crate::sys::EcsDefaultChildComponent;

impl_component_traits_binding_type_w_id!(Component, ECS_COMPONENT);
impl_component_traits_binding_type_w_id!(Identifier, ECS_IDENTIFIER);
impl_component_traits_binding_type_w_id!(Poly, ECS_POLY);
impl_component_traits_binding_type_w_id!(DefaultChildComponent, ECS_DEFAULT_CHILD_COMPONENT);

// Poly target components
create_pre_registered_component!(Query, ECS_QUERY);
create_pre_registered_component!(Observer, ECS_OBSERVER);

// Core scopes & entities
create_pre_registered_component!(EcsWorld, ECS_WORLD);
create_pre_registered_component!(Flecs, ECS_FLECS);
create_pre_registered_component!(FlecsCore, ECS_FLECS_CORE);
create_pre_registered_component!(FlecsInternals, ECS_FLECS_INTERNALS);
create_pre_registered_component!(Module, ECS_MODULE);
create_pre_registered_component!(Private, ECS_PRIVATE);
create_pre_registered_component!(Prefab, ECS_PREFAB);
create_pre_registered_component!(Disabled, ECS_DISABLED);
create_pre_registered_component!(NotQueryable, ECS_NOT_QUERYABLE);
create_pre_registered_component!(SlotOf, ECS_SLOT_OF);
create_pre_registered_component!(Flag, ECS_FLAG);
create_pre_registered_component!(Monitor, ECS_MONITOR);
create_pre_registered_component!(Empty, ECS_EMPTY);

// Component traits
create_pre_registered_component!(Wildcard, ECS_WILDCARD, "Match all entities");
create_pre_registered_component!(Any, ECS_ANY, "Match at most one entity");
create_pre_registered_component!(This_, ECS_THIS);
create_pre_registered_component!(Variable, ECS_VARIABLE);
// Shortcut as EcsVariable is typically used as source for singleton terms
create_pre_registered_component!(Singleton, ECS_VARIABLE);
create_pre_registered_component!(
    Transitive,
    ECS_TRANSITIVE,
    "Component trait. Relationship is marked as transitive."
);
create_pre_registered_component!(
    Reflexive,
    ECS_REFLEXIVE,
    "Component trait. Relationship is marked as reflexive."
);
create_pre_registered_component!(
    Symmetric,
    ECS_SYMMETRIC,
    "Component trait. Relationship is marked as symmetric."
);
create_pre_registered_component!(
    Final,
    ECS_FINAL,
    "Component trait. This component cannot be used in an [`IsA`] relationship."
);
//create_pre_registered_component!(PairIsTag, ECS_PAIR_IS_TAG); //not supported in Flecs Rust
create_pre_registered_component!(
    Exclusive,
    ECS_EXCLUSIVE,
    "Component trait. Enforces that an entity can only have a single instance of a relationship."
);
create_pre_registered_component!(
    Acyclic,
    ECS_ACYCLIC,
    "Component trait. Indicates that the relationship cannot contain cycles."
);
create_pre_registered_component!(Traversable, ECS_TRAVERSABLE, "Component trait. This relationship can be traversed automatically by queries, e.g. using [`Up`].");
create_pre_registered_component!(With, ECS_WITH, "Component trait. Indicates that this relationship must always come together with another component.");
create_pre_registered_component!(OneOf, ECS_ONE_OF, "Component trait. Enforces that the target of the relationship is a child of a specified entity.");
create_pre_registered_component!(
    CanToggle,
    ECS_CAN_TOGGLE,
    "Component trait. Allows a component to be toggled."
);
create_pre_registered_component!(
    Trait,
    ECS_TRAIT,
    "Component trait. Marks an entity as a trait."
);
create_pre_registered_component!(
    Relationship,
    ECS_RELATIONSHIP,
    "Component trait. Enforces that an entity can only be used as a relationship."
);
create_pre_registered_component!(
    Target,
    ECS_TARGET,
    "Component trait. Enforces that an entity can only be used as the target of a relationship."
);

// OnInstantiate traits
create_pre_registered_component!(
    OnInstantiate,
    ECS_ON_INSTANTIATE,
    "Component trait. Configures behavior of components when an entity is instantiated from another entity. \
    Used as a pair with one of [`Override`], [`Inherit`], or [`DontInherit`]."
);
create_pre_registered_component!(
    Override,
    ECS_OVERRIDE,
    "The default behavior. Inherited components are copied to the instance."
);
create_pre_registered_component!(
    Inherit,
    ECS_INHERIT,
    "Inherited components are not copied to the instance. \
    Operations such as `get` and `has`, and queries will automatically lookup inheritable components \
    by following the [`IsA`] relationship."
);
create_pre_registered_component!(
    DontInherit,
    ECS_DONT_INHERIT,
    "Components with the [`DontInherit`] trait are not inherited from a base entity \
    (the [`IsA`] target) on instantiation."
);

// OnDelete/OnDeleteTarget traits
create_pre_registered_component!(OnDelete, ECS_ON_DELETE);
create_pre_registered_component!(OnDeleteTarget, ECS_ON_DELETE_TARGET);
create_pre_registered_component!(Remove, ECS_REMOVE);
create_pre_registered_component!(Delete, ECS_DELETE);
create_pre_registered_component!(Panic, ECS_PANIC);

// Builtin relationships
create_pre_registered_component!(
    ChildOf,
    ECS_CHILD_OF,
    "Builtin relationship. Allows for the creation of entity hierarchies."
);
create_pre_registered_component!(
    IsA,
    ECS_IS_A,
    "Builtin relationship. Used to express that one entity is equivalent to another."
);
create_pre_registered_component!(
    DependsOn,
    ECS_DEPENDS_ON,
    "Builtin relationship. Used to determine the execution order of systems."
);

// Identifier tags
create_pre_registered_component!(Name, ECS_NAME);
create_pre_registered_component!(Symbol, ECS_SYMBOL);
create_pre_registered_component!(Alias, ECS_ALIAS);

// Events
create_pre_registered_component!(
    OnAdd,
    ECS_ON_ADD,
    "Event. Invoked whenever a component, tag or pair is added to an entity."
);
create_pre_registered_component!(
    OnRemove,
    ECS_ON_REMOVE,
    "Event. Invoked whenever a component, tag or pair is removed from an entity."
);
create_pre_registered_component!(
    OnSet,
    ECS_ON_SET,
    "Event. Invoked whenever a component is assigned a new value."
);
create_pre_registered_component!(OnTableCreate, ECS_ON_TABLE_CREATE);
create_pre_registered_component!(OnTableDelete, ECS_ON_TABLE_DELETE);
create_pre_registered_component!(OnTableEmpty, ECS_ON_TABLE_EMPTY);
create_pre_registered_component!(OnTableFill, ECS_ON_TABLE_FILL);

// System
#[cfg(feature = "flecs_system")]
pub mod system {
    use super::*;
    pub type TickSource = crate::sys::EcsTickSource;
    impl_component_traits_binding_type_w_id!(TickSource, ECS_TICK_SOURCE);

    create_pre_registered_component!(System, ECS_SYSTEM);
}

#[cfg(feature = "flecs_timer")]
pub mod timer {
    use super::*;

    pub type Timer = crate::sys::EcsTimer;
    impl_component_traits_binding_type_w_id!(Timer, ECS_TIMER);

    pub type RateFilter = crate::sys::EcsRateFilter;
    impl_component_traits_binding_type_w_id!(RateFilter, ECS_RATE_FILTER);
}

#[cfg(feature = "flecs_script")]
pub mod script {
    use super::*;
    use crate::sys::FLECS_IDEcsScriptID_;
    pub type Script = crate::sys::EcsScript;
    impl_component_traits_binding_type_w_static_id!(Script, FLECS_IDEcsScriptID_);
}
#[cfg(feature = "flecs_script")]
pub use script::Script;

create_pre_registered_component!(
    Sparse,
    ECS_SPARSE,
    "Component trait. Configures a component to use sparse storage."
);
create_pre_registered_component!(
    Union,
    ECS_UNION,
    "Component trait. Similar to [`Exclusive`] but combines \
    different relationship targets in a single table."
);

// Builtin predicate for comparing entity ids
create_pre_registered_component!(PredEq, ECS_PRED_EQ);
create_pre_registered_component!(PredMatch, ECS_PRED_MATCH);
create_pre_registered_component!(PredLookup, ECS_PRED_LOOKUP);

// builtin marker entities for query scopes
create_pre_registered_component!(ScopeOpen, ECS_SCOPE_OPEN);
create_pre_registered_component!(ScopeClose, ECS_SCOPE_CLOSE);

// Systems
#[cfg(feature = "flecs_system")]
pub mod pipeline {
    use super::*;
    create_pre_registered_component!(Pipeline, ECS_PIPELINE);
    create_pre_registered_component!(OnStart, ECS_ON_START);
    //create_pre_registered_component!(PreFrame, ECS_PRE_FRAME); //not meant to be exposed, internal only
    create_pre_registered_component!(OnLoad, ECS_ON_LOAD);
    create_pre_registered_component!(PostLoad, ECS_POST_LOAD);
    create_pre_registered_component!(PreUpdate, ECS_PRE_UPDATE);
    create_pre_registered_component!(OnUpdate, ECS_ON_UPDATE);
    create_pre_registered_component!(OnValidate, ECS_ON_VALIDATE);
    create_pre_registered_component!(PostUpdate, ECS_POST_UPDATE);
    create_pre_registered_component!(PreStore, ECS_PRE_STORE);
    create_pre_registered_component!(OnStore, ECS_ON_STORE);
    //create_pre_registered_component!(PostFrame, ECS_POST_FRAME); //not meant to be exposed, internal only
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
    create_pre_registered_component!(U16, ECS_U16_T);
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
    create_pre_registered_component!(Constant, ECS_CONSTANT);
    create_pre_registered_component!(Quantity, ECS_QUANTITY);
    create_pre_registered_component!(EcsOpaque, ECS_OPAQUE);

    // Meta type components
    pub type Type = sys::EcsType;
    pub type TypeSerializer = sys::EcsTypeSerializer;
    pub type Primitive = sys::EcsPrimitive;
    pub type EcsEnum = sys::EcsEnum;
    pub type Bitmask = sys::EcsBitmask;
    pub type Member = sys::EcsMember;
    pub type MemberRanges = sys::EcsMemberRanges;
    pub type EcsStruct = sys::EcsStruct;
    pub type Array = sys::EcsArray;
    pub type Vector = sys::EcsVector;
    pub type Unit = sys::EcsUnit;
    pub type UnitPrefix = sys::EcsUnitPrefix;

    super::impl_component_traits_binding_type_w_id!(Type, ECS_META_TYPE);
    super::impl_component_traits_binding_type_w_id!(TypeSerializer, ECS_META_TYPE_SERIALIZER);
    super::impl_component_traits_binding_type_w_id!(Primitive, ECS_PRIMITIVE);
    super::impl_component_traits_binding_type_w_id!(EcsEnum, ECS_ENUM);
    super::impl_component_traits_binding_type_w_id!(Bitmask, ECS_BITMASK);
    super::impl_component_traits_binding_type_w_id!(Member, ECS_MEMBER);
    super::impl_component_traits_binding_type_w_id!(MemberRanges, ECS_MEMBER_RANGES);
    super::impl_component_traits_binding_type_w_id!(EcsStruct, ECS_STRUCT);
    super::impl_component_traits_binding_type_w_id!(Array, ECS_ARRAY);
    super::impl_component_traits_binding_type_w_id!(Vector, ECS_VECTOR);
    super::impl_component_traits_binding_type_w_id!(Unit, ECS_UNIT);
    super::impl_component_traits_binding_type_w_id!(UnitPrefix, ECS_UNIT_PREFIX);
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

    impl_component_traits_binding_type_w_id!(Rest, ECS_REST);
    unsafe impl Send for Rest {}
    unsafe impl Sync for Rest {}
}

// default component for event API

impl flecs_ecs::core::TagComponent for () {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for () {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo for () {
    const IS_GENERIC: bool = false;
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

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
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
