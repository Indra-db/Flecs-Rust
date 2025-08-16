#![allow(dead_code)]
use crate::common_test::*;

mod component_traits_attributes {
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

mod child_of_isa_attributes {
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

mod add_set_attributes {
    use super::*;

    #[derive(Component)]
    struct TAdd;

    #[derive(Component, Default)]
    struct CAdd;

    #[derive(Component, Default)]
    struct CSetWDefault {
        value: u32,
    }

    #[derive(Component, Default)]
    struct CSet1F {
        value: u32,
    }

    #[derive(Component, Default)]
    struct CSet2F {
        value: u32,
        other: u32,
    }

    #[derive(Component)]
    #[flecs(
        Prefab, //flecs trait to say it's a prefab
        add(TAdd, CAdd,
            (TAdd, CAdd) //pair
            ),
        set(CSet1F { value: 1 }, CSet2F { value: 2, other: 3 }, // inline construction
            CSetWDefault::default(),  //constructed from fn
            (CSet1F { value: 4 }, CSet2F), //pair
            (TAdd, CSet1F { value: 5 }) //pair
        )
    )]
    struct TestAddSet;

    #[test]
    fn add_set_attr() {
        let world = World::new();

        let c = world.component::<TestAddSet>();
        assert!(c.has(flecs::Prefab));
        assert!(c.has(TAdd::id()));
        assert!(c.has(CAdd::id()));
        assert!(c.has((TAdd::id(), CAdd::id())));

        c.get::<(
            &CSet1F,
            &CSet2F,
            &CSetWDefault,
            &(CSet1F, CSet2F),
            &(TAdd, CSet1F),
        )>(|(set1, set2, setdef, setpair, setpair2)| {
            assert_eq!(set1.value, 1);
            assert_eq!(set2.value, 2);
            assert_eq!(set2.other, 3);
            assert_eq!(setdef.value, 0);
            assert_eq!(setpair.value, 4);
            assert_eq!(setpair2.value, 5);
        });

        //internally it does
        // world
        //     .component::<TestAddSet>()
        //     .add((TAdd::id(), CAdd::id()));
        //     .add(TAdd::id())
        //     .add(CAdd::id())
        //     .set(CSet1F { value: 1 })
        //     .set(CSet2F { value: 2, other: 3 })
        //     .set_first(CSet1F { value: 4 }, CSet2F)
        //     .set_second(CTAdd, CSet1F { value: 5 });
    }
}

mod component_hooks_attributes {

    use super::*;

    #[derive(Default, Component)]
    #[flecs(on_add(on_add_hook))]
    struct OnAddHookFn(i32);

    #[derive(Default, Component)]
    #[flecs(on_add(|e, _c| {
        e.world().get::<&mut OnAddHookCounter>(|counter| {
            counter.count += 1;
        });
    }))]
    struct OnAddHookInline(i32);

    #[derive(Default, Component)]
    #[flecs(on_remove(on_remove_hook))]
    struct OnRemoveHookFn(i32);

    #[derive(Default, Component)]
    #[flecs(on_remove(|e, _c| {
        e.world().get::<&mut OnRemoveHookCounter>(|counter| {
            counter.count += 1;
        });
    }))]
    struct OnRemoveHookInline(i32);

    #[derive(Default, Component)]
    #[flecs(on_set(on_set_hook))]
    struct OnSetHookFn(i32);

    #[derive(Default, Component)]
    #[flecs(on_set(|e, _c| {
        e.world().get::<&mut OnSetHookCounter>(|counter| {
            counter.count += 1;
        });
    }))]
    struct OnSetHookInline(i32);

    #[derive(Default, Component)]
    #[flecs(on_replace(on_replace_hook))]
    struct OnReplaceHookFn(i32);

    #[derive(Default, Component)]
    #[flecs(on_replace(|e, _c| {
        e.world().get::<&mut OnReplaceHookCounter>(|counter| {
            counter.count += 1;
        });
    }))]
    struct OnReplaceHookInline(i32);

    fn on_add_hook(e: EntityView<'_>, _c: &mut OnAddHookFn) {
        e.world().get::<&mut OnAddHookCounter>(|counter| {
            counter.count += 1;
        });
    }

    fn on_set_hook(e: EntityView<'_>, _c: &mut OnSetHookFn) {
        e.world().get::<&mut OnSetHookCounter>(|counter| {
            counter.count += 1;
        });
    }

    fn on_remove_hook(e: EntityView<'_>, _c: &mut OnRemoveHookFn) {
        e.world().get::<&mut OnRemoveHookCounter>(|counter| {
            counter.count += 1;
        });
    }

    fn on_replace_hook(e: EntityView<'_>, _c: &mut OnReplaceHookFn) {
        e.world().get::<&mut OnReplaceHookCounter>(|counter| {
            counter.count += 1;
        });
    }

    #[derive(Component, Clone, Default)]
    struct OnAddHookCounter {
        count: u32,
    }

    #[derive(Component, Clone, Default)]
    struct OnSetHookCounter {
        count: u32,
    }

    #[derive(Component, Clone, Default)]
    struct OnRemoveHookCounter {
        count: u32,
    }

    #[derive(Component, Clone, Default)]
    struct OnReplaceHookCounter {
        count: u32,
    }

    #[test]
    fn component_hooks_attr() {
        let world = World::new();

        world.add(OnAddHookCounter::id());
        world.add(OnSetHookCounter::id());
        world.add(OnRemoveHookCounter::id());
        world.add(OnReplaceHookCounter::id());

        world
            .entity()
            .add(OnAddHookFn::id())
            .add(OnAddHookInline::id())
            .add(OnRemoveHookFn::id())
            .add(OnRemoveHookInline::id())
            .set(OnSetHookFn::default())
            .set(OnSetHookInline::default())
            .add(OnReplaceHookFn::id())
            .add(OnReplaceHookInline::id())
            .remove(OnRemoveHookFn::id())
            .remove(OnRemoveHookInline::id());

        let c_add = world.cloned::<&OnAddHookCounter>();
        assert_eq!(c_add.count, 2, "Expected 2 OnAddHook calls");
        let c_set = world.cloned::<&OnSetHookCounter>();
        assert_eq!(c_set.count, 2, "Expected 2 OnSetHook calls");
        let c_remove = world.cloned::<&OnRemoveHookCounter>();
        assert_eq!(c_remove.count, 2, "Expected 2 OnRemoveHook calls");
        let _c_replace = world.cloned::<&OnReplaceHookCounter>();
        //TODO feature flecs version
        //assert_eq!(c_replace.count, 2, "Expected 2 OnReplaceHook calls");
    }
}
