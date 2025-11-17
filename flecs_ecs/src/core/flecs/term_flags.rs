//! Term flags discovered & set during query creation.
//! Mostly used internally to store information relevant to queries.

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
create_pre_registered_component!(IsSparse, IS_SPARSE);
create_pre_registered_component!(IsOr, IS_OR);
create_pre_registered_component!(IsDontFragment, IS_DONT_FRAGMENT);

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
