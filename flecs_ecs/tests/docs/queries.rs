//! Tests from queries.md

#![allow(unused_imports, unused_variables, dead_code, non_snake_case, path_statements, unreachable_code, unused_mut,clippy::print_stdout)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::common_test::*;

#[test]
fn queries_performance_and_caching_performance_tips_tricks_empty_archetype_optimization_01() {
    let world = World::new();
    // Create Position, Velocity query that matches empty archetypes.
    let q = world
        .query::<(&mut Position, &Velocity)>()
        .set_cached()
        .query_flags(QueryFlags::MatchEmptyTables)
        .build();

    let mut desc : flecs_ecs_sys::ecs_delete_empty_tables_desc_t = Default::default();
    desc.time_budget_seconds = 60.0;
    world.delete_empty_tables(desc);
}

#[test]
fn queries_creating_queries_02() {
    let world = World::new();
    // new_query is a convenience function that creates a query with the default builder
    let q = world.new_query::<(&mut Position, &Velocity)>();
}

#[test]
fn queries_creating_queries_03() {
    let world = World::new();
    let q = world.new_query::<(&mut Position, &Velocity)>();
    q.each(|(p, v)| {
        p.x += v.x;
        p.y += v.y;
    });
}

#[test]
fn queries_creating_queries_04() {
    let world = World::new();
    let add_npc = true;
    let mut q = world.query::<(&mut Position, &Velocity)>();
    q.with(Velocity::id());

    if add_npc {
        q.with(Foo::id()); // Conditionally add
    }

    q.build(); // Create query
}

#[test]
fn queries_iteration_05() {
    let world = World::new();
    let q = world.new_query::<(&mut Position, &Velocity)>();
    q.each(|(p, v)| {
        p.x += v.x;
        p.y += v.y;
    });
}

#[test]
fn queries_iteration_06() {
    let world = World::new();
    let q = world.new_query::<(&mut Position, &Velocity)>();
    q.each_entity(|e, (p, v)| {
        println!("Entity: {}", e.name());
        p.x += v.x;
        p.y += v.y;
    });
}

#[test]
fn queries_iteration_07() {
    let world = World::new();
    let q = world
        .query::<&Position>()
        .with((Likes, flecs::Wildcard))
        .build();

    q.each_iter(|it, index, p| {
        println!("Entity: {}: {}", it.entity(index).name(), it.id(1).to_str());
    });
}

#[test]
fn queries_iteration_08() {
    let world = World::new();
    #[derive(Component)]
    struct Tag;

    world.query::<()>().with(Tag).build().each_entity(|e, _| {
        /* */
    });
}

#[test]
fn queries_iteration_09() {
    let world = World::new();
    #[derive(Component)]
    struct Tag;

    world
        .query::<()>()
        .with(Tag::id())
        .build()
        .each_entity(|e, _| { /* */ });
}

#[test]
fn queries_iteration_10() {
    let world = World::new();
    let q = world.new_query::<(&mut Position, &Velocity)>();

    q.run(|mut it| {
        while it.next() {
            let mut p = it.field_mut::<Position>(0);
            let v = it.field::<Velocity>(1);
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
                println!("Entity: {}", it.entity(i).name());
            }
        }
    });
}

#[test]
fn queries_iteration_11() {
    let world = World::new();
    let q = world.new_query::<&Position>();

    q.each_entity(|e, p| {
        e.add(Velocity::id()); // throws locked table assert
    });
}

#[test]
fn queries_iteration_12() {
    let world = World::new();
    let q = world.new_query::<&Position>();

    world.defer(|| {
        q.each_entity(|e, p| {
            e.add(Velocity::id()); // OK
        });
    }); // operations are executed here
}

#[test]
fn queries_iteration_13() {
    let world = World::new();
    let q = world.new_query::<&Position>();

    world.defer_begin();

    q.each_entity(|e, p| {
        e.add(Velocity::id()); // OK
    });

    world.defer_end(); // operations are executed here
}

#[test]
fn queries_reference_components_14() {
    let world = World::new();
    let q = world.new_query::<(&mut Position, &Velocity)>();
}

#[test]
fn queries_reference_components_15() {
    let world = World::new();
    let q = world.new_query::<(&mut Position, &Velocity)>();
    q.each(|(p, v)| { /* */ });
}

#[test]
fn queries_reference_components_16() {
    let world = World::new();
    let q = world.query::<&mut Position>().with(&Velocity::id()).build();
}

#[test]
fn queries_reference_components_17() {
    let world = World::new();
    let npc = world.entity();
    let platoon_01 = world.entity();

    let q = world
        .query::<(&mut Position, &Velocity)>()
        .with(npc)
        .with(platoon_01)
        .build();
}

#[test]
fn queries_reference_components_18() {
    let world = World::new();
    world.component_named::<Position>("Position");
    // Create entity with name so we can look it up
    let npc = world.entity_named("npc");

    let q = world
        .query::<(&Position, &Npc)>()
        .with("npc")
        .with("Position")
        .build();
}

#[test]
fn queries_reference_wildcards_19() {
    let world = World::new();
    let e = world
        .entity()
        .add(Position::id())
        .add(Velocity::id());

    let q = world
        .query::<()>()
        .with(flecs::Wildcard::id())
        .build();
}

#[test]
fn queries_reference_wildcards_20() {
    let world = World::new();
    let e = world
        .entity()
        .add(Position::id()).add(Velocity::id());

    let q = world
        .query::<()>()
        .with(flecs::Any::id())
        .build();
}

#[test]
fn queries_reference_pairs_21() {
    let world = World::new();
     #[derive(Component)]
     struct Eats {
         value: f32,
     }

     #[derive(Component)]
     struct Apples;

     let q = world.new_query::<&mut (Eats, Apples)>();

     q.each(|eats| {
         eats.value += 1.0;
     });
}

#[test]
fn queries_reference_pairs_22() {
    let world = World::new();
    #[derive(Component)]
    struct Eats {
        value: f32,
    }

    #[derive(Component)]
    struct Apples;

    let eats = world.component::<Eats>();
    let apples = world.component::<Apples>();
    let q1 = world.query::<()>().with((Eats::id(), Apples::id())).build(); // tuple types
    let q2 = world.query::<()>().with((Eats::id(), apples)).build();
    let q3 = world.query::<()>().with((eats, apples)).build(); // tuple ids
}

#[test]
fn queries_reference_pairs_23() {
    let world = World::new();
    let apples = world.entity();
    let q = world
        .query::<()>()
        .term()
        .set_first(Eats)
        .set_second(apples)
        .build();
}

#[test]
fn queries_reference_pairs_24() {
    let world = World::new();
    world.entity_named("Eats");
    world.entity_named("Apples");
    let q = world
        .query::<()>()
        .term()
        .set_first("Eats")
        .set_second("Apples")
        .build();
}

#[test]
fn queries_reference_pairs_25() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with((Eats::id(), flecs::Wildcard::id()))
        .build();

    q.each_iter(|it, index, _| {
        let pair = it.pair(0);
        let second = pair.second_id();
        let e = it.entity(index);
        println!("Entity {} likes {}", e.name(), second.name());
    });
}

#[test]
fn queries_reference_access_modifiers_26() {
    let world = World::new();
    // The following two queries are the same:
    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .set_inout_kind(InOutKind::In)
        .build();

    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .set_in() // shorthand for .set_inout_kind(InOutKind::In)
        .build();
}

#[test]
fn queries_reference_access_modifiers_27() {
    let world = World::new();
    // Velocity term will be added with InOutKind::In modifier due to `&`
    let q = world.new_query::<(&mut Position, &Velocity)>();
}

#[test]
fn queries_reference_access_modifiers_28() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(&mut Position::id())
        .with(&Velocity::id()) // uses InOutKind::In modifier
        .build();
}

#[test]
fn queries_reference_access_modifiers_29() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(&mut Position::id())
        .with(&Velocity::id())
        .build();

    q.run(|mut it| {
        while it.next() {
            let mut p = it.field_mut::<Position>(0);
            let v = it.field::<Velocity>(1);
        }
    });
}

#[test]
fn queries_reference_access_modifiers_30() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(Position::id()).set_inout()
        .with(Velocity::id()).set_in()
        .build();
}

#[test]
fn queries_reference_and_operator_31() {
    let world = World::new();
    let q = world.new_query::<(&mut Position, &Velocity)>();

    let q2 = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .build();

    let q3 = world
        .query::<()>()
        .with(Position::id())
        .set_oper(OperKind::And)
        .with(Velocity::id())
        .set_oper(OperKind::And)
        .build();
}

#[test]
fn queries_reference_and_operator_32() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(Position::id())
        .and()
        .with(Velocity::id())
        .and()
        .build();
}

#[test]
fn queries_reference_or_operator_33() {
    let world = World::new();
    // Position, Velocity || Speed, Mass
    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .set_oper(OperKind::Or)
        .with(Speed::id())
        .with(Mass::id())
        .build();

    q.run(|mut it| {
        while it.next() {
            let p = it.field::<Position>(0);
            let v = it.field::<Mass>(2); // not 4, because of the Or expression
            let vs_id = it.id(1);
            if vs_id == world.component_id::<Velocity>() {
                // We can only use ecs_field if the field type is the same for all results,
                // but we can use range() to get the table column directly.
                let v = it.range().unwrap().get_mut::<Velocity>();
                // iterate as usual
            } else if vs_id == world.component_id::<Speed>() {
                let s = it.range().unwrap().get_mut::<Speed>();
                // iterate as usual
            }
        }
    });
}

#[test]
fn queries_reference_or_operator_34() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .or()
        .with(Speed::id())
        .with(Mass::id())
        .build();
}

#[test]
fn queries_reference_not_operator_35() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .set_oper(OperKind::Not)
        .build();
}

#[test]
fn queries_reference_not_operator_36() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .not()
        .build();
}

#[test]
fn queries_reference_not_operator_37() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(Position::id())
        .without(Velocity::id())
        .build();
}

#[test]
fn queries_reference_optional_operator_38() {
    let world = World::new();
    let q = world.new_query::<(&Position, Option<&Velocity>)>();

    q.each(|(p, v)| {
        if let Some(v) = v {
            // ...
        }
    });
}

#[test]
fn queries_reference_optional_operator_39() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .set_oper(OperKind::Optional)
        .build();

    q.run(|mut it| {
        while it.next() {
            let p = it.field::<Position>(0);
            if let Some(v) = it.get_field::<Velocity>(1) {
                // iterate as usual
            }
        }
    });
}

#[test]
fn queries_reference_optional_operator_40() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(Position::id())
        .with(Velocity::id())
        .optional()
        .build();
}

#[test]
fn queries_reference_equality_operators_41() {
    let world = World::new();
    world.component_named::<Foo>("Foo");
    return; // TODO bug fix in master
    world
        .query::<()>()
        // $this == Foo
        .with((flecs::PredEq, Foo))
        // $this != Foo
        .without((flecs::PredEq, Bar))
        // $this == "Foo"
        .with(flecs::PredEq)
        .set_second("Foo")
        .flags(sys::EcsIsName)
        // $this ~= "Fo"
        .with(flecs::PredMatch)
        .set_second("Fo")
        .flags(sys::EcsIsName)
        .build();
}

#[test]
fn queries_reference_andfrom_orfrom_notfrom_operators_42() {
    let world = World::new();
    let type_list = world.prefab()
      .add(Position::id())
      .add(Velocity::id());

    let q = world
        .query::<()>()
        .with(type_list)
        .set_oper(OperKind::AndFrom) // match Position, Velocity
        .with(type_list)
        .set_oper(OperKind::OrFrom) // match Position || Velocity
        .with(type_list)
        .set_oper(OperKind::NotFrom) // match !Position, !Velocity
        .build();
}

#[test]
fn queries_reference_andfrom_orfrom_notfrom_operators_43() {
    let world = World::new();
    let type_list = world.prefab()
    .add(Position::id())
    .add(Velocity::id());

    let q = world
        .query::<()>()
        .with(type_list)
        .and_from()
        .with(type_list)
        .or_from()
        .with(type_list)
        .not_from()
        .build();
}

#[test]
fn queries_reference_query_scopes_44() {
    let world = World::new();
    world
        .query::<()>()
        // Position, !{ Velocity || Speed }
        .with(Position::id())
        .scope_open()
        .not()
        .with(Velocity::id())
        .or()
        .with(Speed::id())
        .scope_close()
        .build();
}

#[test]
fn queries_reference_source_45() {
    let world = World::new();
    let game = world.entity().add(SimTime::id());

    let q = world
        .query::<()>()
        .with(&mut Position::id()) // normal term, uses $this source
        .with(Velocity::id()) // normal term, uses $this source
        .with(SimTime::id())
        .set_src(game) // fixed source, match SimTime on Game
        .build();

    q.run(|mut it| {
        while it.next() {
            let mut p = it.field_mut::<Position>(0);
            let v = it.field::<Velocity>(1);
            let st = it.field::<SimTime>(2);
            for i in it.iter() {
                p[i].x += v[i].x * st[0].value; // 0 because it's a single source element
                p[i].y += v[i].y * st[0].value;
            }
        }
    });
}

#[test]
fn queries_reference_source_46() {
    let world = World::new();
    let game = world.entity().add(SimTime::id());
    let q = world
        .query::<(&mut Position, &Velocity, &SimTime)>()
        .term_at(2)
        .set_src(game) // fixed source for 3rd template argument (SimTime)
        .build();

    // Because all components are now part of the query type, we can use each
    q.each_entity(|e, (p, v, st)| {
        p.x += v.x * st.value;
        p.y += v.y * st.value;
    });
}

#[test]
fn queries_reference_source_47() {
    let world = World::new();
    let config = world.entity().add(SimConfig::id());
    let game = world.entity().add(SimTime::id());
    let q = world
        .query::<(&SimConfig, &mut SimTime)>()
        .term_at(0)
        .set_src(config)
        .term_at(1)
        .set_src(game)
        .build();

    // Ok (note that it.count() will be 0)
    q.run(|mut it| {
        while it.next() {
            let sc = it.field::<SimConfig>(0);
            let mut st = it.field_mut::<SimTime>(1);
            st[0].value += sc[0].sim_speed; // 0 because it's a single source element
        }
    });

    // Ok
    q.each(|(sc, st)| {
        st.value += sc.sim_speed;
    });

    // Ok
    q.each_iter(|it, index, (sc, st)| {
        st.value += sc.sim_speed;
    });

    /*
    // Not ok: there is no entity to pass to first argument
    q.each_entity(|e, (sc, st)| {
        st.value += sc.sim_speed;
    });
    */
}

#[test]
fn queries_reference_source_48() {
    let world = World::new();
    let config = world.entity_named("Config").add(SimConfig::id());
    let game = world.entity_named("Game").add(SimTime::id());
    let q = world
        .query::<(&SimConfig, &SimTime)>()
        .term_at(0)
        .set_src("Config")
        .term_at(1)
        .set_src("Game")
        .build();
}

#[test]
fn queries_reference_relationship_traversal_traversal_flags_49() {
    let world = World::new();
    // These three queries are the same:
    let q1 = world
        .query::<()>()
        .with(Mass::id())
        .up_id(flecs::ChildOf::id())
        .build();

    let q2 = world
        .query::<()>()
        .with(Mass::id())
        .up() // defaults to .up(flecs::ChildOf)
        .build();

    let q3 = world
        .query::<()>()
        .with(Mass::id())
        .parent() // shortcut for .up(flecs::ChildOf)
        .build();
}

#[test]
fn queries_reference_relationship_traversal_traversal_flags_50() {
    let world = World::new();
     // Register an inheritable component 'Mass'
     world
         .component::<Mass>()
         .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

     // These two queries are the same:
     let q1 = world
         .query::<()>()
         .with(Mass::id())
         .self_()
         .up_id(flecs::IsA::id())
         .build();

     let q2 = world
         .query::<()>()
         .with(Mass::id()) // defaults to .self().up(flecs::IsA)
         .build();
}

#[test]
fn queries_reference_relationship_traversal_traversal_flags_51() {
    let world = World::new();
    // Register an inheritable component 'Mass'
    world
        .component::<Mass>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().add(Mass::id());

    let parent = world.entity().is_a(base); // inherits Mass

    let child = world.entity().child_of(parent);

    // Matches 'child', because parent inherits Mass from prefab
    let q = world
        .query::<()>()
        .with(Mass::id())
        .up() // traverses ChildOf upwards
        .build();
}

#[test]
fn queries_reference_relationship_traversal_limitations_52() {
    let world = World::new();
     // Register inheritable 'Position' component
     world
         .component::<Position>()
         .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

     let base = world.entity().add(Position::id());
     let inst = world.entity().is_a(base); // short for .add((flecs::IsA::ID, base));

     // The following two queries are the same:
     let q1 = world.new_query::<&Position>();

     let q2 = world
         .query::<&Position>()
         .term_at(0)
         .self_()
         .up_id(flecs::IsA::ID)
         .build();
}

#[test]
fn queries_reference_relationship_traversal_limitations_53() {
    let world = World::new();
    let parent = world.entity().add(Position::id());
    let child = world.entity().child_of(parent); // short for .add((flecs::ChildOf::ID, base));

    let q = world
    .query::<&Position>()
    .term_at(0).up()
    .build();
}

#[test]
fn queries_reference_relationship_traversal_limitations_54() {
    let world = World::new();
    // Create a new traversable relationship
    let contained_by = world.entity().add(flecs::Traversable::id());

    let parent = world.entity().add(Position::id());

    let child = world.entity().add((contained_by, parent));

    let q = world
        .query::<&Position>()
        .term_at(0)
        .up_id(contained_by)
        .build();
}

#[test]
fn queries_reference_variables_setting_variables_55() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(SpaceShip::id())
        .with(DockedTo::id())
        .set_second("$Location")
        .with(Planet::id())
        .set_src("$Location")
        .build();
}

#[test]
fn queries_reference_variables_setting_variables_56() {
    let world = World::new();
    let q = world
        .query::<()>()
        .with(SpaceShip::id())
        .with(DockedTo::id())
        .second()
        .set_var("$Location")
        .with(Planet::id())
        .src()
        .set_var("$Location")
        .build();
}

#[test]
fn queries_reference_variables_setting_variables_57() {
    let world = World::new();
    let earth = world.entity();
    let q = world
    .query::<()>()
    .with(SpaceShip::id())
    .with(DockedTo::id())
    .second()
    .set_var("$Location")
    .with(Planet::id())
    .src()
    .set_var("$Location")
    .build();
    let location_var = q.find_var("Location").unwrap();

    q.iterable().set_var(location_var, earth).each(|it| {
        // iterate as usual
    });
}

#[test]
fn queries_reference_variables_setting_variables_58() {
    let world = World::new();
    let earth = world.entity();
    let q = world
    .query::<()>()
    .with(SpaceShip::id())
    .with(DockedTo::id())
    .second()
    .set_var("$Location")
    .with(Planet::id())
    .src()
    .set_var("$Location")
    .build();

    q.iterable().set_var_expr("Location", earth).each(|it| {
        // iterate as usual
    });
}

#[test]
fn queries_reference_member_value_queries_59() {
    let world = World::new();
    // Rust API does not support member value queries until reflection is implemented. This is the Meta addon.
}

#[test]
fn queries_reference_change_detection_60() {
    let world = World::new();
    // Query used for change detection.
    let q_read = world.query::<&Position>()
      .detect_changes()
      .build();

    // Query used to create changes
    let q_write = world.new_query::<&mut Position>(); // defaults to inout

    // Test if changes have occurred for anything matching the query.
    let changed = q_read.is_changed();

    // Setting a component will update the changed state
    let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    q_write.run(|mut it| {
        while it.next() {
            if !changed {
                // If no changes are made to the iterated table, the skip function can be
                // called to prevent marking the matched components as dirty.
                it.skip();
            } else {
                // Iterate as usual. It does not matter whether the code actually writes the
                // components or not: when a table is not skipped, components matched with
                // inout or out terms will be marked dirty by the iterator.
            }
        }
    });

    q_read.run(|mut it| {
        while it.next() {
            if it.is_changed() {
                // Check if the current table has changed. The change state will be reset
                // after the table is iterated, so code can respond to changes in individual
                // tables.
            }
        }
    });
}

#[test]
fn queries_reference_sorting_sorting_algorithm_61() {
    let world = World::new();
    // Use readonly term for component used for sorting
    let q = world
        .query::<(&Depth, &Position)>()
        .order_by::<Depth>(|e1, d1: &Depth, e2, d2: &Depth| {
            (d1.value > d2.value) as i32 - (d1.value < d2.value) as i32
        })
        .build();
}

#[test]
fn queries_reference_sorting_sorting_algorithm_62() {
    let world = World::new();
    let depth_id = world.component::<Depth>();

    let q = world
        .query::<&Position>()
        .with(depth_id)
        .set_in()
        .order_by_id(depth_id, |e1, d1: *const c_void, e2, d2: *const c_void| {
            let d1 = unsafe { &*(d1 as *const Depth) };
            let d2 = unsafe { &*(d2 as *const Depth) };
            (d1.value > d2.value) as i32 - (d1.value < d2.value) as i32
        })
        .build();
}

#[test]
fn queries_reference_sorting_sorting_algorithm_63() {
    let world = World::new();
    let q = world
        .query::<&Position>()
        .order_by_id(0, |e1, _d1: *const c_void, e2, _d2: *const c_void| {
            (e1 > e2) as i32 - (e1 < e2) as i32
        })
        .build();
}

#[test]
fn queries_reference_grouping_group_iterators_64() {
    let world = World::new();
    // see example in examples folder under query/group_by
}

#[test]
fn queries_reference_grouping_group_iterators_65() {
    let world = World::new();
    // see example in examples folder under query/group_by
}

#[test]
fn queries_reference_grouping_group_iterators_66() {
    let world = World::new();
    // see example in examples folder under query/group_by
}

#[test]
fn queries_reference_component_inheritance_67() {
    let world = World::new();
    #[derive(Component)]
    struct Unit;

    let unit = world.component::<Unit>();

    let melee_unit = world.entity().is_a(Unit::id());
    let ranged_unit = world.entity().is_a(Unit::id());

    let unit_01 = world.entity().add(melee_unit);
    let unit_02 = world.entity().add(ranged_unit);

    // Matches entities with Unit, MeleeUnit and RangedUnit
    let q = world.query::<&Unit>();

    // Iterate as usual
}

#[test]
fn queries_reference_transitive_relationships_68() {
    let world = World::new();
    // Create LocatedIn relationship with transitive property
    #[derive(Component)]
    struct LocatedIn;

    world.component::<LocatedIn>().add(flecs::Transitive::id());

    let new_york = world.entity();
    let manhattan = world.entity().add((LocatedIn::id(), new_york));
    let central_park = world.entity().add((LocatedIn::id(), manhattan));
    let bob = world.entity().add((LocatedIn::id(), central_park));

    // Matches ManHattan, CentralPark, Bob
    let q = world
        .query::<()>()
        .with((LocatedIn::id(), new_york))
        .build();

    // Iterate as usual
}

#[test]
fn queries_reference_transitive_relationships_69() {
    let world = World::new();
    // Matches:
    //  - ManHattan (Place = NewYork)
    //  - CentralPark (Place = ManHattan, NewYork)
    //  - Bob (Place = CentralPark, ManHattan, NewYork)
    let q = world
        .query::<()>()
        .with(LocatedIn::id())
        .set_second("$Place")
        .build();
}

#[test]
fn queries_reference_transitive_relationships_70() {
    let world = World::new();
    #[derive(Component)]
    struct City;

    let new_york = world.entity();

    // Add City property to NewYork
    new_york.add(City::id());

    // Matches:
    //  - ManHattan (Place = NewYork)
    //  - CentralPark (Place = NewYork)
    //  - Bob (Place = NewYork)

    let q = world
        .query::<()>()
        .with(LocatedIn::id())
        .set_second("$Place")
        .with(City::id())
        .set_src("$Place")
        .build();
}

#[test]
fn queries_reference_reflexive_relationships_71() {
    let world = World::new();
    let tree = world.entity();
    let oak = world.entity().is_a(tree);

    // Matches Tree, Oak
    let q = world
    .query::<()>()
    .with((flecs::IsA::id(), tree))
    .build();

    // Iterate as usual
}