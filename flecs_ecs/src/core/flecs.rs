//! Contains flecs traits and pre-registered components.
use core::ops::Deref;

use crate::core::*;
use crate::sys;

//TODO: a lot of these need to be feature gated

pub trait FlecsTrait {}

macro_rules! create_pre_registered_component {
    ($struct_name:ident, $const_name:ident) => {
        create_pre_registered_component!($struct_name, $const_name, "");
    };
    ($struct_name:ident, $const_name:ident, $doc:tt) => {
        #[derive(Debug, Default, Clone)]
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
            const IMPLS_CLONE: bool = true;
            const IMPLS_DEFAULT: bool = true;
            const IMPLS_PARTIAL_EQ: bool = false;
            const IMPLS_PARTIAL_ORD: bool = false;
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

            fn entity_id<'a>(_world: impl WorldProvider<'a>) -> sys::ecs_id_t {
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
            fn internal_on_component_registration(_world: WorldRef, _component_id: super::Entity) {}
        }

        impl OnComponentRegistration for $struct_name {
            fn on_component_registration(_world: WorldRef, _component_id: super::Entity) {}
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
    create_pre_registered_component!(IsCacheable, IS_CACHEABLE);
    create_pre_registered_component!(IsScope, IS_SCOPE);
    create_pre_registered_component!(IsMember, IS_MEMBER);
    create_pre_registered_component!(IsToggle, IS_TOGGLE);
    create_pre_registered_component!(KeepAlive, KEEP_ALIVE);
    create_pre_registered_component!(IsSparse, IS_SPARSE);
    create_pre_registered_component!(IsOr, IS_OR);
    create_pre_registered_component!(IsDontFragment, IS_DONT_FRAGMENT);
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
create_pre_registered_component!(
    OrderedChildren,
    ECS_ORDERED_CHILDREN,
    "Tag that when added to a parent ensures stable order of `ecs_children` result."
);
create_pre_registered_component!(Flag, ECS_FLAG);
create_pre_registered_component!(Monitor, ECS_MONITOR);
create_pre_registered_component!(Empty, ECS_EMPTY);
create_pre_registered_component!(Constant, ECS_CONSTANT);

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
create_pre_registered_component!(
    Inheritable,
    ECS_INHERITABLE,
    "Component trait. Mark component as inheritable.
    This is the opposite of Final. This trait can be used to enforce that queries
    take into account component inheritance before inheritance (`IsA`) 
    relationships are added with the component as target."
);

create_pre_registered_component!(
    PairIsTag,
    ECS_PAIR_IS_TAG,
    "Component trait. A relationship can be marked with `PairIsTag` in which case
     a pair with the relationship will never contain data."
);
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
create_pre_registered_component!(
    Traversable,
    ECS_TRAVERSABLE,
    "Component trait. This relationship can be traversed automatically by queries, e.g. using [`Up`]."
);
create_pre_registered_component!(
    With,
    ECS_WITH,
    "Component trait. Indicates that this relationship must always come together with another component."
);
create_pre_registered_component!(
    OneOf,
    ECS_ONE_OF,
    "Component trait. Enforces that the target of the relationship is a child of a specified entity."
);
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
    DontFragment,
    ECS_DONT_FRAGMENT,
    "Component trait. Mark component as non-fragmenting"
);

// Builtin predicate for comparing entity ids
create_pre_registered_component!(PredEq, ECS_PRED_EQ);
create_pre_registered_component!(PredMatch, ECS_PRED_MATCH);
create_pre_registered_component!(PredLookup, ECS_PRED_LOOKUP);

// builtin marker entities for query scopes
create_pre_registered_component!(ScopeOpen, ECS_SCOPE_OPEN);
create_pre_registered_component!(ScopeClose, ECS_SCOPE_CLOSE);

// Pipeline
#[cfg(feature = "flecs_pipeline")]
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

#[cfg(all(
    not(feature = "flecs_meta"),
    not(feature = "flecs_rust_no_enum_reflection")
))]
pub mod meta {
    use super::*;

    create_pre_registered_component!(I8, ECS_I8_T);
    create_pre_registered_component!(I16, ECS_I16_T);
    create_pre_registered_component!(I32, ECS_I32_T);
    create_pre_registered_component!(I64, ECS_I64_T);
    create_pre_registered_component!(U8, ECS_U8_T);
    create_pre_registered_component!(U16, ECS_U16_T);
    create_pre_registered_component!(U32, ECS_U32_T);
    create_pre_registered_component!(U64, ECS_U64_T);
}

#[cfg(feature = "flecs_meta")]
pub mod meta {
    use super::*;
    // Meta primitive components (don't use low ids to save id space)
    create_pre_registered_component!(Bool, ECS_BOOL_T);
    create_pre_registered_component!(Char, ECS_CHAR_T);
    create_pre_registered_component!(Byte, ECS_BYTE_T);
    create_pre_registered_component!(UPtr, ECS_UPTR_T);
    create_pre_registered_component!(IPtr, ECS_IPTR_T);
    create_pre_registered_component!(I8, ECS_I8_T);
    create_pre_registered_component!(I16, ECS_I16_T);
    create_pre_registered_component!(I32, ECS_I32_T);
    create_pre_registered_component!(I64, ECS_I64_T);
    create_pre_registered_component!(U8, ECS_U8_T);
    create_pre_registered_component!(U16, ECS_U16_T);
    create_pre_registered_component!(U32, ECS_U32_T);
    create_pre_registered_component!(U64, ECS_U64_T);
    create_pre_registered_component!(F32, ECS_F32_T);
    create_pre_registered_component!(F64, ECS_F64_T);
    create_pre_registered_component!(String, ECS_STRING_T);
    create_pre_registered_component!(Entity, ECS_ENTITY_T);
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
#[cfg(feature = "flecs_doc")]
pub mod doc {
    use super::*;
    create_pre_registered_component!(Description, ECS_DOC_DESCRIPTION);
    create_pre_registered_component!(Brief, ECS_DOC_BRIEF);
    create_pre_registered_component!(Detail, ECS_DOC_DETAIL);
    create_pre_registered_component!(Link, ECS_DOC_LINK);
    create_pre_registered_component!(Color, ECS_DOC_COLOR);
    create_pre_registered_component!(UUID, ECS_DOC_UUID);
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
        pub ipaddr: *mut ::core::ffi::c_char,
        pub impl_: *mut ::core::ffi::c_void,
    }

    impl Default for Rest {
        fn default() -> Self {
            Self {
                port: Default::default(),
                ipaddr: core::ptr::null_mut::<core::ffi::c_char>(),
                impl_: core::ptr::null_mut::<core::ffi::c_void>(),
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
    const IMPLS_PARTIAL_EQ: bool =
        { flecs_ecs::core::utility::types::ImplementsPartialEq::<()>::IMPLS };
    const IMPLS_PARTIAL_ORD: bool =
        { flecs_ecs::core::utility::types::ImplementsPartialOrd::<()>::IMPLS };
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
    type TagType = flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsATag;
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_vs_rust_ids() {
        let world = flecs_ecs::core::World::new();

        unsafe {
            assert_eq!(Self_, sys::EcsSelf as u64, "EcsSelf (C) != Self_ (Rust)");
            assert_eq!(Up, sys::EcsUp, "EcsUp (C) != Up (Rust)");
            assert_eq!(Trav, sys::EcsTrav, "EcsTrav (C) != Trav (Rust)");
            assert_eq!(Cascade, sys::EcsCascade, "EcsCascade (C) != Cascade (Rust)");
            assert_eq!(Desc, sys::EcsDesc, "EcsDesc (C) != Desc (Rust)");
            assert_eq!(
                IsVariable,
                sys::EcsIsVariable,
                "EcsIsVariable (C) != IsVariable (Rust)"
            );
            assert_eq!(
                IsEntity,
                sys::EcsIsEntity,
                "EcsIsEntity (C) != IsEntity (Rust)"
            );
            assert_eq!(IsName, sys::EcsIsName, "EcsIsName (C) != IsName (Rust)");
            assert_eq!(
                TraverseFlags,
                sys::EcsTraverseFlags as u64,
                "EcsTraverseFlags (C) != TraverseFlags (Rust)"
            );
            assert_eq!(
                TermRefFlags,
                sys::EcsTermRefFlags as u64,
                "EcsTermRefFlags (C) != TermRefFlags (Rust)"
            );

            // Term flags
            assert_eq!(
                term_flags::MatchAny,
                sys::EcsTermMatchAny as u64,
                "EcsTermMatchAny (C) != MatchAny (Rust)"
            );
            assert_eq!(
                term_flags::MatchAnySrc,
                sys::EcsTermMatchAnySrc as u64,
                "EcsTermMatchAnySrc (C) != MatchAnySrc (Rust)"
            );
            assert_eq!(
                term_flags::Transitive,
                sys::EcsTermTransitive as u64,
                "EcsTermTransitive (C) != Transitive (Rust)"
            );
            assert_eq!(
                term_flags::Reflexive,
                sys::EcsTermReflexive as u64,
                "EcsTermReflexive (C) != Reflexive (Rust)"
            );
            assert_eq!(
                term_flags::IdInherited,
                sys::EcsTermIdInherited as u64,
                "EcsTermIdInherited (C) != IdInherited (Rust)"
            );
            assert_eq!(
                term_flags::IsTrivial,
                sys::EcsTermIsTrivial as u64,
                "EcsTermIsTrivial (C) != IsTrivial (Rust)"
            );
            assert_eq!(
                term_flags::IsCacheable,
                sys::EcsTermIsCacheable as u64,
                "EcsTermIsCacheable (C) != IsCacheable (Rust)"
            );
            assert_eq!(
                term_flags::IsScope,
                sys::EcsTermIsScope as u64,
                "EcsTermIsScope (C) != IsScope (Rust)"
            );
            assert_eq!(
                term_flags::IsMember,
                sys::EcsTermIsMember as u64,
                "EcsTermIsMember (C) != IsMember (Rust)"
            );
            assert_eq!(
                term_flags::IsToggle,
                sys::EcsTermIsToggle as u64,
                "EcsTermIsToggle (C) != IsToggle (Rust)"
            );
            assert_eq!(
                term_flags::KeepAlive,
                sys::EcsTermKeepAlive as u64,
                "EcsTermKeepAlive (C) != KeepAlive (Rust)"
            );
            assert_eq!(
                term_flags::IsSparse,
                sys::EcsTermIsSparse as u64,
                "EcsTermIsSparse (C) != IsSparse (Rust)"
            );
            assert_eq!(
                term_flags::IsOr,
                sys::EcsTermIsOr as u64,
                "EcsTermIsOr (C) != IsOr (Rust)"
            );
            assert_eq!(
                term_flags::IsDontFragment,
                sys::EcsTermDontFragment as u64,
                "EcsTermDontFragment (C) != IsDontFragment (Rust)"
            );

            // Query flags
            assert_eq!(
                query_flags::MatchPrefab,
                sys::EcsQueryMatchPrefab as u64,
                "EcsQueryMatchPrefab (C) != MatchPrefab (Rust)"
            );
            assert_eq!(
                query_flags::MatchDisabled,
                sys::EcsQueryMatchDisabled as u64,
                "EcsQueryMatchDisabled (C) != MatchDisabled (Rust)"
            );
            assert_eq!(
                query_flags::MatchEmptyTables,
                sys::EcsQueryMatchEmptyTables as u64,
                "EcsQueryMatchEmptyTables (C) != MatchEmptyTables (Rust)"
            );
            assert_eq!(
                query_flags::AllowUnresolvedByName,
                sys::EcsQueryAllowUnresolvedByName as u64,
                "EcsQueryAllowUnresolvedByName (C) != AllowUnresolvedByName (Rust)"
            );
            assert_eq!(
                query_flags::TableOnly,
                sys::EcsQueryTableOnly as u64,
                "EcsQueryTableOnly (C) != TableOnly (Rust)"
            );

            assert_eq!(flecs::Component::ID, sys::FLECS_IDEcsComponentID_);
            assert_eq!(flecs::Identifier::ID, sys::FLECS_IDEcsIdentifierID_);
            assert_eq!(flecs::Poly::ID, sys::FLECS_IDEcsPolyID_);
            assert_eq!(
                flecs::DefaultChildComponent::ID,
                sys::FLECS_IDEcsDefaultChildComponentID_
            );

            // Poly target components
            assert_eq!(flecs::Query, sys::EcsQuery);
            assert_eq!(flecs::Observer, sys::EcsObserver);

            // Core scopes & entities
            assert_eq!(flecs::EcsWorld, sys::EcsWorld);
            assert_eq!(flecs::Flecs, sys::EcsFlecs);
            assert_eq!(flecs::FlecsCore, sys::EcsFlecsCore);
            //assert_eq!(flecs::FlecsInternals, sys::EcsFlecsInternals);
            assert_eq!(flecs::Module, sys::EcsModule);
            assert_eq!(flecs::Private, sys::EcsPrivate);
            assert_eq!(flecs::Prefab, sys::EcsPrefab);
            assert_eq!(flecs::Disabled, sys::EcsDisabled);
            assert_eq!(flecs::NotQueryable, sys::EcsNotQueryable);
            assert_eq!(flecs::SlotOf, sys::EcsSlotOf);
            assert_eq!(flecs::OrderedChildren, sys::EcsOrderedChildren);
            //assert_eq!(flecs::Flag, sys::EcsFlag);
            assert_eq!(flecs::Monitor, sys::EcsMonitor);
            assert_eq!(flecs::Empty, sys::EcsEmpty);
            assert_eq!(flecs::Constant, sys::EcsConstant);

            // Component traits
            assert_eq!(flecs::Wildcard, sys::EcsWildcard);
            assert_eq!(flecs::Any, sys::EcsAny);
            assert_eq!(flecs::This_, sys::EcsThis);
            assert_eq!(flecs::Variable, sys::EcsVariable);
            assert_eq!(flecs::Singleton, sys::EcsVariable);
            assert_eq!(flecs::Transitive, sys::EcsTransitive);
            assert_eq!(flecs::Reflexive, sys::EcsReflexive);
            assert_eq!(flecs::Symmetric, sys::EcsSymmetric);
            assert_eq!(flecs::Final, sys::EcsFinal);
            assert_eq!(flecs::Inheritable, sys::EcsInheritable);
            assert_eq!(flecs::PairIsTag, sys::EcsPairIsTag);
            assert_eq!(flecs::Exclusive, sys::EcsExclusive);
            assert_eq!(flecs::Acyclic, sys::EcsAcyclic);
            assert_eq!(flecs::Traversable, sys::EcsTraversable);
            assert_eq!(flecs::With, sys::EcsWith);
            assert_eq!(flecs::OneOf, sys::EcsOneOf);
            assert_eq!(flecs::CanToggle, sys::EcsCanToggle);
            assert_eq!(flecs::Trait, sys::EcsTrait);
            assert_eq!(flecs::Relationship, sys::EcsRelationship);
            assert_eq!(flecs::Target, sys::EcsTarget);

            // OnInstantiate traits
            assert_eq!(flecs::OnInstantiate, sys::EcsOnInstantiate);
            assert_eq!(flecs::Override, sys::EcsOverride);
            assert_eq!(flecs::Inherit, sys::EcsInherit);
            assert_eq!(flecs::DontInherit, sys::EcsDontInherit);

            // OnDelete/OnDeleteTarget traits
            assert_eq!(flecs::OnDelete, sys::EcsOnDelete);
            assert_eq!(flecs::OnDeleteTarget, sys::EcsOnDeleteTarget);
            assert_eq!(flecs::Remove, sys::EcsRemove);
            assert_eq!(flecs::Delete, sys::EcsDelete);
            assert_eq!(flecs::Panic, sys::EcsPanic);

            // Builtin relationships
            assert_eq!(flecs::ChildOf, sys::EcsChildOf);
            assert_eq!(flecs::IsA, sys::EcsIsA);
            assert_eq!(flecs::DependsOn, sys::EcsDependsOn);

            // Identifier tags
            assert_eq!(flecs::Name, sys::EcsName);
            assert_eq!(flecs::Symbol, sys::EcsSymbol);
            assert_eq!(flecs::Alias, sys::EcsAlias);

            // Events
            assert_eq!(flecs::OnAdd, sys::EcsOnAdd);
            assert_eq!(flecs::OnRemove, sys::EcsOnRemove);
            assert_eq!(flecs::OnSet, sys::EcsOnSet);
            assert_eq!(flecs::OnTableCreate, sys::EcsOnTableCreate);
            assert_eq!(flecs::OnTableDelete, sys::EcsOnTableDelete);

            // System
            #[cfg(feature = "flecs_system")]
            {
                assert_eq!(flecs::system::TickSource::ID, sys::FLECS_IDEcsTickSourceID_);
                assert_eq!(flecs::system::System, sys::EcsSystem);
            }

            // Timer
            #[cfg(feature = "flecs_timer")]
            {
                assert_eq!(flecs::timer::Timer::ID, sys::FLECS_IDEcsTimerID_);
                assert_eq!(flecs::timer::RateFilter::ID, sys::FLECS_IDEcsRateFilterID_);
            }

            // Script
            #[allow(static_mut_refs)]
            #[cfg(feature = "flecs_script")]
            {
                assert_eq!(
                    flecs::script::Script::__register_or_get_id::<false>(&world),
                    sys::FLECS_IDEcsScriptID_
                );
            }

            assert_eq!(
                flecs::Sparse,
                sys::EcsSparse,
                "EcsSparse (C) != Sparse (Rust)",
            );
            assert_eq!(
                flecs::DontFragment,
                sys::EcsDontFragment,
                "EcsDontFragment (C) != DontFragment (Rust)",
            );

            // Builtin predicate for comparing entity ids
            assert_eq!(flecs::PredEq, sys::EcsPredEq);
            assert_eq!(flecs::PredMatch, sys::EcsPredMatch);
            assert_eq!(flecs::PredLookup, sys::EcsPredLookup);

            // builtin marker entities for query scopes
            assert_eq!(flecs::ScopeOpen, sys::EcsScopeOpen);
            assert_eq!(flecs::ScopeClose, sys::EcsScopeClose);

            // Pipeline
            #[cfg(feature = "flecs_pipeline")]
            {
                assert_eq!(flecs::pipeline::Pipeline, sys::FLECS_IDEcsPipelineID_);
                assert_eq!(flecs::pipeline::OnStart, sys::EcsOnStart);
                assert_eq!(flecs::pipeline::OnLoad, sys::EcsOnLoad);
                assert_eq!(flecs::pipeline::PostLoad, sys::EcsPostLoad);
                assert_eq!(flecs::pipeline::PreUpdate, sys::EcsPreUpdate);
                assert_eq!(flecs::pipeline::OnUpdate, sys::EcsOnUpdate);
                assert_eq!(flecs::pipeline::OnValidate, sys::EcsOnValidate);
                assert_eq!(flecs::pipeline::PostUpdate, sys::EcsPostUpdate);
                assert_eq!(flecs::pipeline::PreStore, sys::EcsPreStore);
                assert_eq!(flecs::pipeline::OnStore, sys::EcsOnStore);
                assert_eq!(flecs::pipeline::Phase, sys::EcsPhase);
            }

            // Meta
            #[cfg(feature = "flecs_meta")]
            {
                assert_eq!(flecs::meta::Bool, sys::FLECS_IDecs_bool_tID_);
                assert_eq!(flecs::meta::Char, sys::FLECS_IDecs_char_tID_);
                assert_eq!(flecs::meta::Byte, sys::FLECS_IDecs_byte_tID_);
                assert_eq!(flecs::meta::U8, sys::FLECS_IDecs_u8_tID_);
                assert_eq!(flecs::meta::U16, sys::FLECS_IDecs_u16_tID_);
                assert_eq!(flecs::meta::U32, sys::FLECS_IDecs_u32_tID_);
                assert_eq!(flecs::meta::U64, sys::FLECS_IDecs_u64_tID_);
                assert_eq!(flecs::meta::UPtr, sys::FLECS_IDecs_uptr_tID_);
                assert_eq!(flecs::meta::I8, sys::FLECS_IDecs_i8_tID_);
                assert_eq!(flecs::meta::I16, sys::FLECS_IDecs_i16_tID_);
                assert_eq!(flecs::meta::I32, sys::FLECS_IDecs_i32_tID_);
                assert_eq!(flecs::meta::I64, sys::FLECS_IDecs_i64_tID_);
                assert_eq!(flecs::meta::IPtr, sys::FLECS_IDecs_iptr_tID_);
                assert_eq!(flecs::meta::F32, sys::FLECS_IDecs_f32_tID_);
                assert_eq!(flecs::meta::F64, sys::FLECS_IDecs_f64_tID_);
                assert_eq!(flecs::meta::String, sys::FLECS_IDecs_string_tID_);
                assert_eq!(flecs::meta::Entity, sys::FLECS_IDecs_entity_tID_);
                assert_eq!(flecs::meta::Quantity, sys::EcsQuantity);
                assert_eq!(flecs::meta::EcsOpaque, sys::FLECS_IDEcsOpaqueID_);

                assert_eq!(flecs::meta::Type::ID, sys::FLECS_IDEcsTypeID_);
                assert_eq!(
                    flecs::meta::TypeSerializer::ID,
                    sys::FLECS_IDEcsTypeSerializerID_
                );
                assert_eq!(flecs::meta::Primitive::ID, sys::FLECS_IDEcsPrimitiveID_);
                assert_eq!(flecs::meta::EcsEnum::ID, sys::FLECS_IDEcsEnumID_);
                assert_eq!(flecs::meta::Bitmask::ID, sys::FLECS_IDEcsBitmaskID_);
                assert_eq!(flecs::meta::Member::ID, sys::FLECS_IDEcsMemberID_);
                assert_eq!(
                    flecs::meta::MemberRanges::ID,
                    sys::FLECS_IDEcsMemberRangesID_
                );
                assert_eq!(flecs::meta::EcsStruct::ID, sys::FLECS_IDEcsStructID_);
                assert_eq!(flecs::meta::Array::ID, sys::FLECS_IDEcsArrayID_);
                assert_eq!(flecs::meta::Vector::ID, sys::FLECS_IDEcsVectorID_);
                assert_eq!(flecs::meta::Unit::ID, sys::FLECS_IDEcsUnitID_);
                assert_eq!(flecs::meta::UnitPrefix::ID, sys::FLECS_IDEcsUnitPrefixID_);
            }

            // Doc
            #[cfg(feature = "flecs_doc")]
            {
                assert_eq!(flecs::doc::Description, sys::FLECS_IDEcsDocDescriptionID_);
                assert_eq!(flecs::doc::Brief, sys::EcsDocBrief);
                assert_eq!(flecs::doc::Detail, sys::EcsDocDetail);
                assert_eq!(flecs::doc::Link, sys::EcsDocLink);
                assert_eq!(flecs::doc::Color, sys::EcsDocColor);
                assert_eq!(flecs::doc::UUID, sys::EcsDocUuid);
            }

            // Rest
            #[cfg(feature = "flecs_rest")]
            {
                assert_eq!(flecs::rest::Rest::ID, sys::FLECS_IDEcsRestID_);
            }
        }
    }
}
