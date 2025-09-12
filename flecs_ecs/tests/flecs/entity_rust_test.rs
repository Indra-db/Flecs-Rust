#![allow(dead_code)]

use crate::common_test::*;
use rstest::rstest;
use rstest_reuse::*;

mod misc_tests {
    use super::*;

    #[test]
    fn count_target_ids() {
        let world = World::new();

        let e = world.entity();
        let r = world.entity();
        let o = world.entity();

        e.add((r, o));
        e.add((r, o));

        assert_eq!(e.target_id_count(r).unwrap(), 1);

        let e2 = world.entity();
        e2.add((r, o));

        assert_eq!(e.target_id_count(r).unwrap(), 1);
        assert_eq!(e2.target_id_count(r).unwrap(), 1);

        let o2 = world.entity();

        e.add((r, o2));

        assert_eq!(e.target_id_count(r).unwrap(), 2);
        assert_eq!(e2.target_id_count(r).unwrap(), 1);
    }

    #[test]
    fn entity_id_reuse() {
        let world = World::new();

        let a = world.entity_named("a");
        let b = world.entity().child_of(a);
        let first_archetype = b.archetype().to_string();
        a.destruct();

        let a = world.entity_named("a");
        let b = world.entity().child_of(a);
        assert!(
            b.id() > u32::MAX as u64,
            "this test is not valid if the id was not reused"
        );
        assert_eq!(b.archetype().to_string(), first_archetype);
    }
}

#[derive(Clone, Copy, Debug)]
enum ComponentType {
    Fragment,
    Sparse,
    DontFragment,
}

fn set_component_type<T: ComponentId>(world: &World, component_type: ComponentType) {
    match component_type {
        ComponentType::Fragment => {
            world.component::<T>();
        }
        ComponentType::Sparse => {
            world.component::<T>().add_trait::<flecs::Sparse>();
        }
        ComponentType::DontFragment => {
            world.component::<T>().add_trait::<flecs::DontFragment>();
        }
    }
}

#[template]
#[rstest(
    case::fragment(ComponentType::Fragment),
    case::sparse(ComponentType::Sparse),
    case::dont_fragment(ComponentType::DontFragment)
)]
fn component_types(#[case] ty: ComponentType) {}

mod cloned_tests {
    use super::*;

    #[apply(component_types)]
    fn cloned_single_present(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);

        let e = world.entity().set(Value { value: 42 });
        let v = e.cloned::<&Value>();
        assert_eq!(v.value, 42);
    }

    #[apply(component_types)]
    fn cloned_tuple_all_present(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        set_component_type::<Value3>(&world, ty);
        let e = world
            .entity()
            .set(Value { value: 42 })
            .set(Value2 { value: 84 })
            .set(Value3 { value: 168 });
        let v = e.cloned::<(&Value, &Value2, &Value3)>();
        assert_eq!(v.0.value, 42);
        assert_eq!(v.1.value, 84);
        assert_eq!(v.2.value, 168);
    }

    #[should_panic]
    #[apply(component_types)]
    fn cloned_single_missing_panics(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        let e = world.entity();
        let _ = e.cloned::<&Value>();
    }

    #[should_panic]
    #[apply(component_types)]
    fn cloned_tuple_all_missing_panics(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        set_component_type::<Value3>(&world, ty);
        let e = world.entity();
        let _ = e.cloned::<(&Value, &Value2, &Value3)>();
    }

    #[should_panic]
    #[apply(component_types)]
    fn cloned_tuple_some_missing_panics(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        set_component_type::<Value3>(&world, ty);
        let e = world.entity().set(Value { value: 42 });
        let _ = e.cloned::<(&Value, &Value2, &Value3)>();
    }

    #[apply(component_types)]
    fn cloned_option_single_present(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        let e = world.entity().set(Value { value: 42 });
        let v = e.cloned::<Option<&Value>>();
        assert!(v.is_some());
        assert_eq!(v.unwrap().value, 42);
    }

    #[apply(component_types)]
    fn cloned_option_single_absent(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        let e = world.entity();
        let v = e.cloned::<Option<&Value>>();
        assert!(v.is_none());
    }

    #[apply(component_types)]
    fn cloned_option_tuple_all_present(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        set_component_type::<Value3>(&world, ty);
        let e = world
            .entity()
            .set(Value { value: 42 })
            .set(Value2 { value: 84 })
            .set(Value3 { value: 168 });
        let v = e.cloned::<(Option<&Value>, Option<&Value2>, Option<&Value3>)>();
        assert_eq!(v.0.unwrap().value, 42);
        assert_eq!(v.1.unwrap().value, 84);
        assert_eq!(v.2.unwrap().value, 168);
    }

    #[apply(component_types)]
    fn cloned_option_tuple_all_absent(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        set_component_type::<Value3>(&world, ty);
        let e = world.entity();
        let v = e.cloned::<(Option<&Value>, Option<&Value2>, Option<&Value3>)>();
        assert!(v.0.is_none());
        assert!(v.1.is_none());
        assert!(v.2.is_none());
    }

    #[apply(component_types)]
    fn cloned_option_tuple_partial_present(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        set_component_type::<Value3>(&world, ty);
        let e = world.entity().set(Value { value: 42 });
        let v = e.cloned::<(Option<&Value>, Option<&Value2>, Option<&Value3>)>();
        assert_eq!(v.0.unwrap().value, 42);
        assert!(v.1.is_none());
        assert!(v.2.is_none());
    }

    #[apply(component_types)]
    fn cloned_mixed_required_and_optional_present(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        let e = world
            .entity()
            .set(Value { value: 11 })
            .set(Value2 { value: 22 });
        let v = e.cloned::<(&Value, Option<&Value2>)>();
        assert_eq!(v.0.value, 11);
        assert_eq!(v.1.unwrap().value, 22);
    }

    #[apply(component_types)]
    fn cloned_mixed_required_and_optional_absent(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        let e = world.entity().set(Value { value: 33 });
        let v = e.cloned::<(&Value, Option<&Value2>)>();
        assert_eq!(v.0.value, 33);
        assert!(v.1.is_none());
    }

    #[should_panic]
    #[apply(component_types)]
    fn cloned_mixed_required_and_optional_required_missing_panics(ty: ComponentType) {
        let world = World::new();
        set_component_type::<Value>(&world, ty);
        set_component_type::<Value2>(&world, ty);
        let e = world.entity();
        let _ = e.cloned::<(&Value, Option<&Value2>)>();
    }

    mod inheritance {

        use super::*;

        #[apply(component_types)]
        fn cloned_inherited_from_prefab_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            world
                .component::<Value>()
                .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
            let base = world.prefab().set(Value { value: 77 });
            let inst = world.entity().is_a(base);
            let v = inst.cloned::<&Value>();
            assert_eq!(v.value, 77);
        }

        #[apply(component_types)]
        fn cloned_inherited_overridden_takes_precedence(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            world
                .component::<Value>()
                .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
            let base = world.prefab().set(Value { value: 100 });
            let inst = world.entity().is_a(base).set(Value { value: 5 });
            let v = inst.cloned::<&Value>();
            assert_eq!(v.value, 5);
        }

        #[apply(component_types)]
        fn cloned_inherited_pair_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            world
                .component::<Value>()
                .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
            let base = world.prefab().set_first(Value { value: 88 }, Tag::id());
            let inst = world.entity().is_a(base);
            let v = inst.cloned::<&(Value, Tag)>();
            assert_eq!(v.value, 88);
        }
    }

    mod pairs {
        use super::*;

        mod basic {
            use super::*;

            #[apply(component_types)]
            fn pair_single_present(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let e = world.entity().set_first(Value { value: 42 }, Tag::id());
                let v = e.cloned::<&(Value, Tag)>();
                assert_eq!(v.value, 42);
            }

            #[should_panic]
            #[apply(component_types)]
            fn pair_single_missing_panics(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let e = world.entity();
                let _ = e.cloned::<&(Value, Tag)>();
            }

            #[apply(component_types)]
            fn pair_option_single_present(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let e = world.entity().set_first(Value { value: 42 }, Tag::id());
                let v = e.cloned::<Option<&(Value, Tag)>>().unwrap();
                assert_eq!(v.value, 42);
            }

            #[apply(component_types)]
            fn pair_tuple_all_present(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                set_component_type::<Value2>(&world, ty);
                let e = world
                    .entity()
                    .set_first(Value { value: 42 }, Tag::id())
                    .set_first(Value2 { value: 84 }, Tag::id());
                let v = e.cloned::<(&(Value, Tag), &(Value2, Tag))>();
                assert_eq!(v.0.value, 42);
                assert_eq!(v.1.value, 84);
            }

            #[should_panic]
            #[apply(component_types)]
            fn pair_tuple_all_missing_panics(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                set_component_type::<Value2>(&world, ty);
                let e = world.entity();
                let _ = e.cloned::<(&(Value, Tag), &(Value2, Tag))>();
            }

            #[apply(component_types)]
            fn pair_option_tuple_all_present(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                set_component_type::<Value2>(&world, ty);
                let e = world
                    .entity()
                    .set_first(Value { value: 42 }, Tag::id())
                    .set_first(Value2 { value: 84 }, Tag::id());
                let v = e.cloned::<(Option<&(Value, Tag)>, Option<&(Value2, Tag)>)>();
                assert_eq!(v.0.unwrap().value, 42);
                assert_eq!(v.1.unwrap().value, 84);
            }

            #[apply(component_types)]
            fn pair_option_tuple_partial_present(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                set_component_type::<Value2>(&world, ty);
                let e = world.entity().set_first(Value { value: 42 }, Tag::id());
                let v = e.cloned::<(Option<&(Value, Tag)>, Option<&(Value2, Tag)>)>();
                assert_eq!(v.0.unwrap().value, 42);
                assert!(v.1.is_none());
            }

            #[apply(component_types)]
            fn pair_option_tuple_all_absent(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                set_component_type::<Value2>(&world, ty);
                let e = world.entity();
                let v = e.cloned::<(Option<&(Value, Tag)>, Option<&(Value2, Tag)>)>();
                assert!(v.0.is_none());
                assert!(v.1.is_none());
            }
        }

        mod wildcard_any {
            use super::*;

            #[ignore = "wildcard pair w/ sparse broken"]
            #[apply(component_types)]
            fn pair_wildcard_single_present(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let e = world.entity().set_first(Value { value: 42 }, Tag::id());
                let v = e.cloned::<&(Value, flecs::Wildcard)>();
                assert_eq!(v.value, 42);
            }

            #[ignore = "wildcard pair w/ sparse broken"]
            #[apply(component_types)]
            fn pair_any_single_present(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let e = world.entity().set_first(Value { value: 42 }, Tag::id());
                let v = e.cloned::<&(Value, flecs::Any)>();
                assert_eq!(v.value, 42);
            }

            #[ignore = "wildcard pair w/ sparse broken"]
            #[apply(component_types)]
            fn pair_wildcard_multiple_objects_picks_first(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let o1 = world.entity();
                let o2 = world.entity();
                let e = world
                    .entity()
                    .set_first(Value { value: 1 }, o1)
                    .set_first(Value { value: 2 }, o2);
                let v = e.cloned::<&(Value, flecs::Wildcard)>();
                assert_eq!(v.value, 1);
            }

            #[ignore = "wildcard pair w/ sparse broken"]
            #[apply(component_types)]
            fn pair_any_multiple_objects_picks_first(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let o1 = world.entity();
                let o2 = world.entity();
                let e = world
                    .entity()
                    .set_first(Value { value: 10 }, o1)
                    .set_first(Value { value: 20 }, o2);
                let v = e.cloned::<&(Value, flecs::Any)>();
                assert_eq!(v.value, 10);
            }

            #[ignore = "wildcard pair w/ sparse broken"]
            #[apply(component_types)]
            fn pair_option_wildcard_single_absent(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let e = world.entity();
                let v = e.cloned::<Option<&(Value, flecs::Wildcard)>>();
                assert!(v.is_none());
            }

            #[ignore = "wildcard pair w/ sparse broken"]
            #[apply(component_types)]
            fn pair_option_wildcard_single_present(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let o = world.entity();
                let e = world.entity().set_first(Value { value: 88 }, o);
                let v = e.cloned::<Option<&(Value, flecs::Wildcard)>>();
                assert_eq!(v.unwrap().value, 88);
            }
        }

        mod set_second {
            use super::*;

            #[apply(component_types)]
            fn pair_set_second_and_read(ty: ComponentType) {
                let world = World::new();
                set_component_type::<Value>(&world, ty);
                let e = world.entity().set_second(Tag::id(), Value { value: 64 });
                let v = e.cloned::<&(Tag, Value)>();
                assert_eq!(v.value, 64);
            }
        }
    }

    mod mixed {
        use super::*;

        #[apply(component_types)]
        fn mixed_pair_and_nonpair_all_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set_first(Value { value: 7 }, Tag::id())
                .set(Value2 { value: 14 });
            let v = e.cloned::<(&(Value, Tag), &Value2)>();
            assert_eq!(v.0.value, 7);
            assert_eq!(v.1.value, 14);
        }

        #[should_panic]
        #[apply(component_types)]
        fn mixed_pair_missing_panics(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            let e = world.entity().set(Value2 { value: 9 });
            let _ = e.cloned::<(&(Value, Tag), &Value2)>();
        }

        #[should_panic]
        #[apply(component_types)]
        fn mixed_nonpair_missing_panics(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            let e = world.entity().set_first(Value { value: 5 }, Tag::id());
            let _ = e.cloned::<(&(Value, Tag), &Value2)>();
        }

        #[apply(component_types)]
        fn mixed_pair_required_and_optional_component_absent_ok(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world.entity().set_first(Value { value: 21 }, Tag::id());
            let v = e.cloned::<(&(Value, Tag), Option<&Value2>)>();
            assert_eq!(v.0.value, 21);
            assert!(v.1.is_none());
        }

        #[apply(component_types)]
        fn mixed_pair_required_and_optional_component_present_ok(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set_first(Value { value: 30 }, Tag::id())
                .set(Value2 { value: 60 });
            let v = e.cloned::<(&(Value, Tag), Option<&Value2>)>();
            assert_eq!(v.0.value, 30);
            assert_eq!(v.1.unwrap().value, 60);
        }

        #[apply(component_types)]
        fn mixed_optional_pair_absent_required_component_present_ok(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world.entity().set(Value2 { value: 101 });
            let v = e.cloned::<(Option<&(Value, Tag)>, &Value2)>();
            assert!(v.0.is_none());
            assert_eq!(v.1.value, 101);
        }

        #[apply(component_types)]
        fn mixed_optional_pair_present_required_component_present_ok(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set_first(Value { value: 3 }, Tag::id())
                .set(Value2 { value: 6 });
            let v = e.cloned::<(Option<&(Value, Tag)>, &Value2)>();
            assert_eq!(v.0.unwrap().value, 3);
            assert_eq!(v.1.value, 6);
        }

        #[apply(component_types)]
        fn mixed_two_pairs_and_nonpair_all_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            set_component_type::<Value3>(&world, ty);
            let e = world
                .entity()
                .set_first(Value { value: 1 }, Tag::id())
                .set(Value2 { value: 2 })
                .set_first(Value3 { value: 3 }, Tag::id());
            let v = e.cloned::<(&(Value, Tag), &Value2, &(Value3, Tag))>();
            assert_eq!(v.0.value, 1);
            assert_eq!(v.1.value, 2);
            assert_eq!(v.2.value, 3);
        }

        #[apply(component_types)]
        fn mixed_set_second_pair_and_nonpair_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set_second(Tag::id(), Value { value: 8 })
                .set(Value2 { value: 16 });
            let v = e.cloned::<(&(Tag, Value), &Value2)>();
            assert_eq!(v.0.value, 8);
            assert_eq!(v.1.value, 16);
        }

        #[ignore = "wildcard pair w/ sparse broken"]
        #[apply(component_types)]
        fn mixed_wildcard_pair_and_nonpair_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let o1 = world.entity();
            let o2 = world.entity();
            let e = world
                .entity()
                .set_first(Value { value: 5 }, o1)
                .set_first(Value { value: 6 }, o2)
                .set(Value2 { value: 7 });
            let v = e.cloned::<(&(Value, flecs::Wildcard), &Value2)>();
            assert_eq!(v.0.value, 5);
            assert_eq!(v.1.value, 7);
        }

        #[apply(component_types)]
        fn mixed_nonpair_then_pair_all_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set(Value2 { value: 14 })
                .set_first(Value { value: 7 }, Tag::id());
            let v = e.cloned::<(&Value2, &(Value, Tag))>();
            assert_eq!(v.0.value, 14);
            assert_eq!(v.1.value, 7);
        }

        #[apply(component_types)]
        fn mixed_option_both_absent(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world.entity();
            let v = e.cloned::<(Option<&(Value, Tag)>, Option<&Value2>)>();
            assert!(v.0.is_none());
            assert!(v.1.is_none());
        }

        #[apply(component_types)]
        fn mixed_required_component_optional_pair_absent_ok(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world.entity().set(Value2 { value: 1 });
            let v = e.cloned::<(&Value2, Option<&(Value, Tag)>)>();
            assert_eq!(v.0.value, 1);
            assert!(v.1.is_none());
        }

        #[apply(component_types)]
        fn mixed_required_component_optional_pair_present_ok(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set(Value2 { value: 2 })
                .set_first(Value { value: 3 }, Tag::id());
            let v = e.cloned::<(&Value2, Option<&(Value, Tag)>)>();
            assert_eq!(v.0.value, 2);
            assert_eq!(v.1.unwrap().value, 3);
        }

        #[apply(component_types)]
        fn mixed_three_terms_required_pair_required_component_optional_pair_absent_ok(
            ty: ComponentType,
        ) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set_first(Value { value: 1 }, Tag::id())
                .set(Value2 { value: 2 });
            let v = e.cloned::<(&(Value, Tag), &Value2, Option<&(Value3, Tag)>)>();
            assert_eq!(v.0.value, 1);
            assert_eq!(v.1.value, 2);
            assert!(v.2.is_none());
        }

        #[should_panic]
        #[apply(component_types)]
        fn mixed_three_terms_required_component_missing_panics(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world.entity().set_first(Value { value: 1 }, Tag::id());
            let _ = e.cloned::<(&(Value, Tag), &Value2, Option<&(Value3, Tag)>)>();
        }

        #[apply(component_types)]
        fn mixed_two_pairs_and_optional_nonpair_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set_first(Value { value: 1 }, Tag::id())
                .set_first(Value3 { value: 3 }, Tag::id())
                .set(Value2 { value: 2 });
            let v = e.cloned::<(&(Value, Tag), &(Value3, Tag), Option<&Value2>)>();
            assert_eq!(v.0.value, 1);
            assert_eq!(v.1.value, 3);
            assert_eq!(v.2.unwrap().value, 2);
        }

        #[apply(component_types)]
        fn mixed_two_pairs_and_optional_nonpair_absent(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let e = world
                .entity()
                .set_first(Value { value: 1 }, Tag::id())
                .set_first(Value3 { value: 3 }, Tag::id());
            let v = e.cloned::<(&(Value, Tag), &(Value3, Tag), Option<&Value2>)>();
            assert_eq!(v.0.value, 1);
            assert_eq!(v.1.value, 3);
            assert!(v.2.is_none());
        }

        #[ignore = "wildcard pair w/ sparse broken"]
        #[apply(component_types)]
        fn mixed_wildcard_pair_optional_nonpair_present(ty: ComponentType) {
            let world = World::new();
            set_component_type::<Value>(&world, ty);
            set_component_type::<Value2>(&world, ty);
            let target = world.entity();
            let e = world
                .entity()
                .set_first(Value { value: 9 }, target)
                .set(Value2 { value: 18 });
            let v = e.cloned::<(&(Value, flecs::Wildcard), Option<&Value2>)>();
            assert_eq!(v.0.value, 9);
            assert_eq!(v.1.unwrap().value, 18);
        }
    }
}
