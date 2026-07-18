//! Contains core pre-registered components used by various core features.

use super::*;

// Builtin component ids
pub type Component = crate::sys::EcsComponent;
pub type Identifier = crate::sys::EcsIdentifier;
pub type Poly = crate::sys::EcsPoly;
pub type DefaultChildComponent = crate::sys::EcsDefaultChildComponent;
pub type Parent = crate::sys::EcsParent;
pub type TreeSpawner = crate::sys::EcsTreeSpawner;

impl_component_traits_binding_type_w_id!(Component, ECS_COMPONENT);
impl_component_traits_binding_type_w_id!(Identifier, ECS_IDENTIFIER);
impl_component_traits_binding_type_w_id!(Poly, ECS_POLY);
impl_component_traits_binding_type_w_id!(DefaultChildComponent, ECS_DEFAULT_CHILD_COMPONENT);
impl_component_traits_binding_type_w_id!(Parent, ECS_PARENT);
impl_component_traits_binding_type_w_id!(TreeSpawner, ECS_TREE_SPAWNER);

create_pre_registered_component!(
    ParentDepth,
    ECS_PARENT_DEPTH,
    "Exclusive relationship encoding the depth of a non-fragmenting hierarchy (used as value pair)"
);

create_pre_registered_component!(Wildcard, ECS_WILDCARD, "Match all entities");
create_pre_registered_component!(Any, ECS_ANY, "Match at most one entity");
create_pre_registered_component!(This_, ECS_THIS);
create_pre_registered_component!(Variable, ECS_VARIABLE);

// Poly target components
create_pre_registered_component!(Query, ECS_QUERY);
create_pre_registered_component!(Observer, ECS_OBSERVER);

// Core scopes & entities
create_pre_registered_component!(EcsWorld, ECS_WORLD);
create_pre_registered_component!(Flecs, ECS_FLECS);
create_pre_registered_component!(FlecsCore, ECS_FLECS_CORE);
create_pre_registered_component!(Module, ECS_MODULE);
create_component_trait!(Prefab, ECS_PREFAB);
create_component_trait!(Disabled, ECS_DISABLED);
create_component_trait!(NotQueryable, ECS_NOT_QUERYABLE);
create_component_trait!(SlotOf, ECS_SLOT_OF);

create_pre_registered_component!(Flag, ECS_FLAG);
create_pre_registered_component!(Monitor, ECS_MONITOR);
create_pre_registered_component!(Empty, ECS_EMPTY);
create_pre_registered_component!(Constant, ECS_CONSTANT);

// Identifier tags
create_pre_registered_component!(Name, ECS_NAME);
create_pre_registered_component!(Symbol, ECS_SYMBOL);
create_pre_registered_component!(Alias, ECS_ALIAS);

// Builtin predicate for comparing entity ids
create_pre_registered_component!(PredEq, ECS_PRED_EQ);
create_pre_registered_component!(PredMatch, ECS_PRED_MATCH);
create_pre_registered_component!(PredLookup, ECS_PRED_LOOKUP);

// builtin marker entities for query scopes
create_pre_registered_component!(ScopeOpen, ECS_SCOPE_OPEN);
create_pre_registered_component!(ScopeClose, ECS_SCOPE_CLOSE);

// Builtin relationships
create_component_trait!(
    ChildOf,
    ECS_CHILD_OF,
    "Builtin relationship. Allows for the creation of entity hierarchies."
);
create_component_trait!(
    IsA,
    ECS_IS_A,
    "Builtin relationship. Used to express that one entity is equivalent to another."
);
