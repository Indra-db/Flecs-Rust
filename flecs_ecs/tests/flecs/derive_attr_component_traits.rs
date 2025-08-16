#![allow(dead_code)]
use crate::common_test::*;

mod component_traits {
    use super::*;

    #[derive(Component)]
    struct _Y;

    #[derive(Component)]
    struct Group;

    // Single-trait components
    #[derive(Component)]
    #[flecs(Transitive)]
    struct TTransitive;

    #[derive(Component)]
    #[flecs(Reflexive)]
    struct TReflexive;

    #[derive(Component)]
    #[flecs(Symmetric)]
    struct TSymmetric;

    #[derive(Component)]
    #[flecs(Final)]
    struct TFinal;

    #[derive(Component)]
    #[flecs(Inheritable)]
    struct TInheritable;

    #[derive(Component)]
    #[flecs(PairIsTag)]
    struct TPairIsTag;

    #[derive(Component)]
    #[flecs(Exclusive)]
    struct TExclusive;

    #[derive(Component)]
    #[flecs(Acyclic)]
    struct TAcyclic;

    #[derive(Component)]
    #[flecs(Traversable)]
    struct TTraversable;

    #[derive(Component)]
    #[flecs(CanToggle)]
    struct TCanToggle;

    #[derive(Component)]
    #[flecs(Trait)]
    struct TTrait;

    #[derive(Component)]
    #[flecs(Relationship)]
    struct TRelationship;

    #[derive(Component)]
    #[flecs(Target)]
    struct TTarget;

    #[derive(Component)]
    #[flecs(Sparse)]
    struct TSparse;

    #[derive(Component)]
    #[flecs(DontFragment)]
    struct TDontFragment;

    // Pair-trait components (relationship, target)
    #[derive(Component)]
    #[flecs((With, _Y))]
    struct TWithY;

    #[derive(Component)]
    #[flecs((OneOf, Group))]
    struct TOneOfGroup;

    #[derive(Component)]
    #[flecs((OnInstantiate, Override))]
    struct TOnInstOverride;

    #[derive(Component)]
    #[flecs((OnInstantiate, Inherit))]
    struct TOnInstInherit;

    #[derive(Component)]
    #[flecs((OnInstantiate, DontInherit))]
    struct TOnInstDontInherit;

    #[derive(Component)]
    #[flecs(Acyclic,(OnInstantiate,Inherit),Inheritable,flecs::Sparse)]
    struct MultipleTraits;

    #[test]
    fn component_traits_flags_present() {
        let world = World::new();

        // Singles
        let c = world.component::<TTransitive>();
        assert!(c.has(flecs::Transitive));
        let c = world.component::<TReflexive>();
        assert!(c.has(flecs::Reflexive));
        let c = world.component::<TSymmetric>();
        assert!(c.has(flecs::Symmetric));
        let c = world.component::<TFinal>();
        assert!(c.has(flecs::Final));
        let c = world.component::<TInheritable>();
        assert!(c.has(flecs::Inheritable));
        let c = world.component::<TPairIsTag>();
        assert!(c.has(flecs::PairIsTag));
        let c = world.component::<TExclusive>();
        assert!(c.has(flecs::Exclusive));
        let c = world.component::<TAcyclic>();
        assert!(c.has(flecs::Acyclic));
        let c = world.component::<TTraversable>();
        assert!(c.has(flecs::Traversable));
        let c = world.component::<TCanToggle>();
        assert!(c.has(flecs::CanToggle));
        let c = world.component::<TTrait>();
        assert!(c.has(flecs::Trait));
        let c = world.component::<TRelationship>();
        assert!(c.has(flecs::Relationship));
        let c = world.component::<TTarget>();
        assert!(c.has(flecs::Target));
        let c = world.component::<TSparse>();
        assert!(c.has(flecs::Sparse));
        let c = world.component::<TDontFragment>();
        assert!(c.has(flecs::DontFragment));

        // Pairs
        let c = world.component::<TWithY>();
        assert!(c.has((flecs::With, _Y)));

        let c = world.component::<TOneOfGroup>();
        assert!(c.has((flecs::OneOf, self::Group)));

        let c = world.component::<TOnInstOverride>();
        assert!(c.has((flecs::OnInstantiate, flecs::Override)));
        let c = world.component::<TOnInstInherit>();
        assert!(c.has((flecs::OnInstantiate, flecs::Inherit)));
        let c = world.component::<TOnInstDontInherit>();
        assert!(c.has((flecs::OnInstantiate, flecs::DontInherit)));

        // Multiple traits
        let c = world.component::<MultipleTraits>();
        assert!(c.has(flecs::Acyclic));
        assert!(c.has((flecs::OnInstantiate, flecs::Inherit)));
        assert!(c.has(flecs::Inheritable));
        assert!(c.has(flecs::Sparse));
    }
}

mod child_of_isa {
    use super::*;
    #[derive(Component)]
    struct IsATarget;

    #[derive(Component)]
    struct ChildOfTarget;

    #[derive(Component)]
    #[flecs((IsA, IsATarget))]
    struct TIsA;

    #[derive(Component)]
    #[flecs((ChildOf, ChildOfTarget))]
    struct TChildOf;

    #[test]
    fn child_of_isa_traits_flags_present() {
        let world = World::new();

        let c = world.component::<TIsA>();
        assert!(c.has((flecs::IsA, IsATarget)));
        let c = world.component::<TChildOf>();
        assert!(c.has((flecs::ChildOf, ChildOfTarget)));
    }
}

mod name_attribute {
    use super::*;

    #[derive(Component)]
    #[flecs(meta, name = "AName")]
    struct CompileTestOrdering;

    #[derive(Component)]
    #[flecs(name = "AName", meta)]
    struct CompileTestOrdering2;

    #[derive(Component)]
    #[flecs(name = "AName", meta, DontFragment, flecs::Sparse)]
    struct CompileTestMultipleFlecsAttributes;

    #[derive(Component)]
    #[flecs(name = "AName")]
    struct CompileTestNameAttribute;

    #[test]
    fn der_attr_name_setting() {
        let world = World::new();

        let c = world.component::<CompileTestNameAttribute>();

        assert_eq!(c.name(), "AName");
    }
}
