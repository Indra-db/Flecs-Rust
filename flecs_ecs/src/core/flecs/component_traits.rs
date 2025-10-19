use super::*;

pub trait FlecsComponentTrait {}

// Component traits

create_component_trait!(
    Transitive,
    ECS_TRANSITIVE,
    "Component trait. Relationship is marked as transitive."
);
create_component_trait!(
    Reflexive,
    ECS_REFLEXIVE,
    "Component trait. Relationship is marked as reflexive."
);
create_component_trait!(
    Symmetric,
    ECS_SYMMETRIC,
    "Component trait. Relationship is marked as symmetric."
);
create_component_trait!(
    Final,
    ECS_FINAL,
    "Component trait. This component cannot be used in an [`IsA`] relationship."
);
create_component_trait!(
    Inheritable,
    ECS_INHERITABLE,
    "Component trait. Mark component as inheritable.
    This is the opposite of Final. This trait can be used to enforce that queries
    take into account component inheritance before inheritance (`IsA`) 
    relationships are added with the component as target."
);

create_component_trait!(
    PairIsTag,
    ECS_PAIR_IS_TAG,
    "Component trait. A relationship can be marked with `PairIsTag` in which case
     a pair with the relationship will never contain data."
);
create_component_trait!(
    Exclusive,
    ECS_EXCLUSIVE,
    "Component trait. Enforces that an entity can only have a single instance of a relationship."
);
create_component_trait!(
    Acyclic,
    ECS_ACYCLIC,
    "Component trait. Indicates that the relationship cannot contain cycles."
);
create_component_trait!(
    Traversable,
    ECS_TRAVERSABLE,
    "Component trait. This relationship can be traversed automatically by queries, e.g. using [`Up`]."
);
create_component_trait!(
    With,
    ECS_WITH,
    "Component trait. Indicates that this relationship must always come together with another component."
);
create_component_trait!(
    OneOf,
    ECS_ONE_OF,
    "Component trait. Enforces that the target of the relationship is a child of a specified entity."
);
create_component_trait!(
    CanToggle,
    ECS_CAN_TOGGLE,
    "Component trait. Allows a component to be toggled."
);
create_component_trait!(
    Trait,
    ECS_TRAIT,
    "Component trait. Marks an entity as a trait."
);
create_component_trait!(
    Relationship,
    ECS_RELATIONSHIP,
    "Component trait. Enforces that an entity can only be used as a relationship."
);
create_component_trait!(
    Target,
    ECS_TARGET,
    "Component trait. Enforces that an entity can only be used as the target of a relationship."
);

create_component_trait!(
    Singleton,
    ECS_SINGLETON,
    "Component trait. Configures the components to be used as a singleton only. This automatically tells queries that it's a singleton"
);

// OnInstantiate traits
create_component_trait!(
    OnInstantiate,
    ECS_ON_INSTANTIATE,
    "Component trait. Configures behavior of components when an entity is instantiated from another entity. \
    Used as a pair with one of [`Override`], [`Inherit`], or [`DontInherit`]."
);
create_component_trait!(
    Override,
    ECS_OVERRIDE,
    "The default behavior. Inherited components are copied to the instance."
);
create_component_trait!(
    Inherit,
    ECS_INHERIT,
    "Inherited components are not copied to the instance. \
    Operations such as `get` and `has`, and queries will automatically lookup inheritable components \
    by following the [`IsA`] relationship."
);
create_component_trait!(
    DontInherit,
    ECS_DONT_INHERIT,
    "Components with the [`DontInherit`] trait are not inherited from a base entity \
    (the [`IsA`] target) on instantiation."
);

// OnDelete/OnDeleteTarget traits
create_component_trait!(OnDelete, ECS_ON_DELETE);
create_component_trait!(OnDeleteTarget, ECS_ON_DELETE_TARGET);
create_component_trait!(Remove, ECS_REMOVE);
create_component_trait!(Delete, ECS_DELETE);
create_component_trait!(Panic, ECS_PANIC);

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
create_component_trait!(
    DependsOn,
    ECS_DEPENDS_ON,
    "Builtin relationship. Used to determine the execution order of systems."
);
create_component_trait!(
    OrderedChildren,
    ECS_ORDERED_CHILDREN,
    "Tag that when added to a parent ensures stable order of `ecs_children` result."
);

create_component_trait!(
    Sparse,
    ECS_SPARSE,
    "Component trait. Configures a component to use sparse storage."
);
create_component_trait!(
    DontFragment,
    ECS_DONT_FRAGMENT,
    "Component trait. Mark component as non-fragmenting"
);
