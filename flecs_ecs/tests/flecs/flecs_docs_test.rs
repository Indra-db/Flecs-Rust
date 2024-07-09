//! Compile tests for the flecs docs in the core C repo.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::print_stdout)]
use std::os::raw::c_void;

use flecs_ecs::macros::*;
use flecs_ecs::prelude::*;
use flecs_ecs::sys;

#[derive(Component, Default)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Default)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component, Default)]
struct Game {
    pub time: f32,
}

#[derive(Component, Default)]
struct Foo;

#[derive(Component, Default)]
struct Plate;

#[derive(Component, Default)]
struct Npc;

#[derive(Component, Default)]
struct Likes;

#[derive(Component, Default)]
struct Tag;

#[derive(Component, Default)]
struct Speed;

#[derive(Component, Default)]
struct Mass {
    value: f32,
}

#[derive(Component, Default)]
struct Bar;

#[derive(Component, Default)]
struct SimTime {
    value: f32,
}

#[derive(Component, Default)]
struct SimConfig {
    sim_speed: f32,
}

#[derive(Component, Default)]
struct Player;

#[derive(Component, Default)]
struct Input;

#[derive(Component, Default)]
struct SpaceShip;

#[derive(Component, Default)]
struct Planet;

#[derive(Component, Default)]
struct DockedTo;

#[derive(Component, Default)]
struct Depth {
    value: i32,
}

fn flecs_system_docs_compile_test() {
    let world = World::new();

    let sys = world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });
    sys.run();

    world.progress();

    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .kind_id(0)
        .each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    let q = world.new_query::<(&mut Position, &Velocity)>();

    // Query iteration (each)
    q.each(|(p, v)| { /* ... */ });

    // System iteration (each)
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .each(|(p, v)| { /* ... */ });

    // Query iteration (run)
    q.run(|mut it| {
        while it.next() {
            let mut p = it
                .field::<Position>(0)
                .expect("query term changed and not at the same index anymore");
            let v = it
                .field::<Velocity>(1)
                .expect("query term changed and not at the same index anymore");
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
            }
        }
    });

    // System iteration (run)
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .run(|mut it| {
            while it.next() {
                let mut p = it
                    .field::<Position>(0)
                    .expect("query term changed and not at the same index anymore");
                let v = it
                    .field::<Velocity>(1)
                    .expect("query term changed and not at the same index anymore");
                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
            }
        });

    // Query iteration (run_iter)
    q.run_iter(|it, (p, v)| {
        for i in it.iter() {
            p[i].x += v[i].x;
            p[i].y += v[i].y;
        }
    });

    // System iteration (run_iter)
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .run_iter(|it, (p, v)| {
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
            }
        });

    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .each_iter(|it, i, (p, v)| {
            p.x += v.x * it.delta_time();
            p.y += v.y * it.delta_time();
        });

    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .run_iter(|it, (p, v)| {
            for i in it.iter() {
                p[i].x += v[i].x * it.delta_time();
                p[i].y += v[i].y * it.delta_time();
            }
        });

    let delta_time = 1.0;
    world.progress_time(delta_time);

    world.system_named::<()>("PrintTime").run(|mut it| {
        while it.next() {
            println!("Time: {}", it.delta_time());
        }
    });

    world
        .system_named::<&Game>("PrintTime")
        .term_at(0)
        .singleton()
        .kind::<flecs::pipeline::OnUpdate>()
        .run_iter(|it, game| {
            println!("Time: {}", game[0].time);
        });

    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .kind::<flecs::pipeline::OnUpdate>()
        .each(|(p, v)| {
            // ...
        });

    // Phases must have the EcsPhase tag
    #[derive(Component, Default)]
    struct Physics;

    // a component to represent the phase
    let physics = world.component::<Physics>().add::<flecs::pipeline::Phase>();
    // a (dynamic) entity to represent the phase
    let collisions = world.entity().add::<flecs::pipeline::Phase>();

    // Phases can (but don't have to) depend on other phases which forces ordering
    physics.add_trait::<(flecs::DependsOn, flecs::pipeline::OnUpdate)>();
    collisions.add_trait::<(flecs::DependsOn, Physics)>();

    // Custom phases can be used just like regular phases
    world
        .system_named::<(&Position, &Velocity)>("Collide")
        .kind_id(collisions) // .kind::<Physics>()
        .each(|(p, v)| {
            // ...
        });

    world
        .pipeline()
        .with::<flecs::system::System>()
        .with::<flecs::pipeline::Phase>()
        .cascade_type::<flecs::DependsOn>()
        .without::<flecs::Disabled>()
        .up_type::<flecs::DependsOn>()
        .without::<flecs::Disabled>()
        .up_type::<flecs::ChildOf>()
        .build();

    // Create custom pipeline
    let pipeline = world
        .pipeline()
        .with::<flecs::system::System>()
        .with::<Foo>() // or `.with_id(foo) if an id`
        .build();

    // Configure the world to use the custom pipeline
    world.set_pipeline_id(pipeline);

    // Create system
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .kind::<Foo>() // or `.kind_id(foo) if an id`
        .each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    // Runs the pipeline & system
    world.progress();

    sys.disable_self();
    sys.enable_self();
    sys.add::<flecs::Disabled>();

    world.system::<&Position>().write::<Position>().each(|p| {
        // ...
    });

    world.system::<&Position>().read::<Position>().each(|p| {
        // ...
    });

    world
        .system_named::<&Plate>("AssignPlate")
        .immediate(true)
        .run(|mut it| {
            while it.next() {
                // ...
            }
        });

    world
        .system_named::<&Plate>("AssignPlate")
        .immediate(true)
        .run(|mut it| {
            while it.next() {
                // ECS operations ran here are visible after running the system
                it.world().defer_suspend();
                // ECS operations ran here are immediately visible
                it.world().defer_resume();
                // ECS operations ran here are visible after running the system
            }
        });

    world.set_threads(4);

    world.system::<&Position>().multi_threaded().each(|p| {
        // ...
    });

    world.set_task_threads(4);

    world
        .system::<&Position>()
        .interval(1.0) // Run at 1Hz
        .each(|p| {
            // ...
        });

    world
        .system::<&Position>()
        .rate(2) // Run every other frame
        .each(|p| {
            // ...
        });

    // A rate filter can be created with .rate(2)
    //let tick_source = world.timer().interval(1.0);
    //TODO

    // world
    //     .system::<(&Position, &Velocity)>()
    //     .tick_source_id(tick_source) // Set tick source for system
    //     .each(|(p, v)| {
    //         // ...
    //     });
}

fn flecs_query_docs_compile_test() {
    let world = World::new();

    // Create Position, Velocity query that matches empty archetypes.
    let q = world
        .query::<(&mut Position, &Velocity)>()
        .set_cached()
        .query_flags(QueryFlags::MatchEmptyTables)
        .build();

    // Delete empty archetypes that have been empty for 10 calls to this function.
    world.delete_empty_tables(0, 0, 10, 0, 0.0);

    let q = world.new_query::<(&mut Position, &Velocity)>();

    q.each(|(p, v)| {
        p.x += v.x;
        p.y += v.y;
    });

    let add_npc = true;

    let mut q = world.query::<(&mut Position, &Velocity)>();
    q.with::<&Velocity>();

    if add_npc {
        q.with::<&Foo>(); // Conditionally add
    }

    q.build(); // Create query

    let q = world.new_query::<(&mut Position, &Velocity)>();

    q.each_entity(|e, (p, v)| {
        println!("Entity: {}", e.name());
        p.x += v.x;
        p.y += v.y;
    });

    let q = world
        .query::<&Position>()
        .with::<(&Likes, &flecs::Wildcard)>()
        .build();

    q.each_iter(|it, index, p| {
        println!("Entity: {}: {}", it.entity(index).name(), it.id(1).to_str());
    });

    #[derive(Component, Default)]
    struct Tag;

    world.new_query::<&Tag>().each_entity(|e, tag| { /* */ });

    world
        .query::<()>()
        .with::<&Tag>()
        .build()
        .each_entity(|e, _| { /* */ });

    let q = world.new_query::<(&Position, &Velocity)>();

    q.run(|mut it| {
        while it.next() {
            let mut p = it.field::<Position>(0).unwrap();
            let v = it.field::<Velocity>(1).unwrap();
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
                println!("Entity: {}", it.entity(i).name());
            }
        }
    });

    let q = world.new_query::<&Position>();

    q.each_entity(|e, p| {
        e.add::<Velocity>(); // OK
    });

    let q = world.new_query::<&Position>();

    world.defer(|| {
        q.each_entity(|e, p| {
            e.add::<Velocity>(); // OK
        });
    }); // operations are executed here

    let q = world.new_query::<&Position>();

    world.defer_begin();

    q.each_entity(|e, p| {
        e.add::<Velocity>(); // OK
    });

    world.defer_end(); // operations are executed here

    let q = world.new_query::<(&mut Position, &Velocity)>();

    q.each(|(p, v)| { /* */ });

    let q = world.query::<&mut Position>().with::<&Velocity>().build();

    let npc = world.entity();
    let platoon_01 = world.entity();

    let q = world
        .query::<(&mut Position, &Velocity)>()
        .with_id(npc)
        .with_id(platoon_01)
        .build();

    // Register component type so we can look it up by name
    world.component::<Position>();

    // Create entity with name so we can look it up
    let npc = world.entity_named("npc");

    let q = world
        .query::<(&Position, &Npc)>()
        .with_name("npc")
        .with_name("Position")
        .build();

    let e = world.entity().add::<Position>().add::<Velocity>();

    let q = world.query::<()>().with::<flecs::Wildcard>().build();

    let e = world.entity().add::<Position>().add::<Velocity>();

    let q = world.query::<()>().with::<flecs::Any>().build();

    #[derive(Component, Default)]
    struct Eats {
        value: f32,
    }

    #[derive(Component, Default)]
    struct Apples;

    let q = world.new_query::<&mut (Eats, Apples)>();

    q.each(|eats| {
        eats.value += 1.0;
    });

    let eats = world.component::<Eats>();
    let apples = world.component::<Apples>();

    let q1 = world.query::<()>().with::<(Eats, Apples)>().build();

    let q2 = world.query::<()>().with_first::<Eats>(apples).build();

    let q3 = world.query::<()>().with_id((eats, apples)).build();

    let q = world
        .query::<()>()
        .term()
        .set_first::<Eats>()
        .set_second_id(apples)
        .build();

    let q = world
        .query::<()>()
        .term()
        .set_first_name("Eats")
        .set_second_name("Apples")
        .build();

    let q = world
        .query::<()>()
        .with::<(Eats, flecs::Wildcard)>()
        .build();

    q.each_iter(|it, index, _| {
        let pair = it.pair(0).unwrap();
        let second = pair.second_id();
        let e = it.entity(index);

        println!("Entity {} likes {}", e.name(), second.name());
    });

    // The following two queries are the same:
    let q = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .set_inout_kind(InOutKind::In)
        .build();

    let q = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .set_in() // shorthand for .set_inout_kind(InOutKind::In)
        .build();

    // Velocity term will be added with InOutKind::In modifier due to `&`
    let q = world.new_query::<(&mut Position, &Velocity)>();

    let q = world
        .query::<()>()
        .with::<&mut Position>()
        .with::<&Velocity>() // uses InOutKind::In modifier
        .build();

    let q = world
        .query::<()>()
        .with::<&mut Position>()
        .with::<&Velocity>()
        .build();

    q.run(|mut it| {
        while it.next() {
            let p = it.field::<Position>(0).unwrap();
            let v = it.field::<Velocity>(1).unwrap();
        }
    });

    let q = world
        .query::<()>()
        .with::<Position>()
        .set_inout()
        .with::<Velocity>()
        .set_in()
        .build();

    let q = world
        .query::<()>()
        .with::<Position>()
        .and()
        .with::<Velocity>()
        .and()
        .build();

    let q = world.new_query::<(&mut Position, &Velocity)>();

    let q2 = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .build();

    let q3 = world
        .query::<()>()
        .with::<Position>()
        .set_oper(OperKind::And)
        .with::<Velocity>()
        .set_oper(OperKind::And)
        .build();

    // Position, Velocity || Speed, Mass
    let q = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .set_oper(OperKind::Or)
        .with::<Speed>()
        .with::<Mass>()
        .build();

    q.run(|mut it| {
        while it.next() {
            let p = it.field::<Position>(0).unwrap();
            let v = it.field::<Mass>(2).unwrap(); // not 4, because of the Or expression

            let vs_id = it.id(1);
            if vs_id == world.component_id::<Velocity>() {
                // We can only use ecs_field if the field type is the same for all results,
                // but we can use table_range() to get the table column directly.
                let v = it.range().unwrap().get_mut::<Velocity>();
                // iterate as usual
            } else if vs_id == world.component_id::<Speed>() {
                let s = it.range().unwrap().get_mut::<Speed>();
                // iterate as usual
            }
        }
    });

    let q = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .or()
        .with::<Speed>()
        .with::<Mass>()
        .build();

    let q = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .set_oper(OperKind::Not)
        .build();

    let q = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .not()
        .build();

    let q = world
        .query::<()>()
        .with::<Position>()
        .without::<Velocity>()
        .build();

    let q = world.new_query::<(&Position, Option<&Velocity>)>();

    q.each(|(p, v)| {
        if let Some(v) = v {
            // ...
        }
    });

    let q = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .set_oper(OperKind::Optional)
        .build();

    q.run(|mut it| {
        while it.next() {
            let p = it.field::<Position>(0).unwrap();
            if let Some(v) = it.field::<Velocity>(1) {
                // iterate as usual
            }
        }
    });

    let q = world
        .query::<()>()
        .with::<Position>()
        .with::<Velocity>()
        .optional()
        .build();

    world
        .query::<()>()
        // $this == Foo
        .with::<(flecs::PredEq, Foo)>()
        // $this != Foo
        .without::<(flecs::PredEq, Bar)>()
        // $this == "Foo"
        .with::<flecs::PredEq>()
        .set_second_name("Foo")
        .flags(sys::EcsIsName)
        // $this ~= "Fo"
        .with::<flecs::PredMatch>()
        .set_second_name("Fo")
        .flags(sys::EcsIsName)
        .build();

    let type_list = world.prefab().add::<Position>().add::<Velocity>();

    let q = world
        .query::<()>()
        .with_id(type_list)
        .set_oper(OperKind::AndFrom) // match Position, Velocity
        .with_id(type_list)
        .set_oper(OperKind::OrFrom) // match Position || Velocity
        .with_id(type_list)
        .set_oper(OperKind::NotFrom) // match !Position, !Velocity
        .build();

    let q = world
        .query::<()>()
        .with_id(type_list)
        .and_from()
        .with_id(type_list)
        .or_from()
        .with_id(type_list)
        .not_from()
        .build();

    world
        .query::<()>()
        // Position, !{ Velocity || Speed }
        .with::<Position>()
        .scope_open()
        .not()
        .with::<Velocity>()
        .or()
        .with::<Speed>()
        .scope_close()
        .build();

    let game = world.entity().add::<SimTime>();

    let q = world
        .query::<()>()
        .with::<Position>() // normal term, uses $this source
        .with::<Velocity>() // normal term, uses $this source
        .with::<SimTime>()
        .set_src_id(game) // fixed source, match SimTime on Game
        .build();

    q.run(|mut it| {
        while it.next() {
            let mut p = it.field::<Position>(0).unwrap();
            let v = it.field::<Velocity>(1).unwrap();
            let st = it.field::<SimTime>(2).unwrap();

            for i in it.iter() {
                p[i].x += v[i].x * st[0].value;
                p[i].y += v[i].y * st[0].value;
            }
        }
    });

    let q = world
        .query::<(&mut Position, &Velocity, &SimTime)>()
        .term_at(2)
        .set_src_id(game) // fixed source for 3rd template argument (SimTime)
        .build();

    // Because all components are now part of the query type, we can use each
    q.each_entity(|e, (p, v, st)| {
        p.x += v.x * st.value;
        p.y += v.y * st.value;
    });

    let cfg = world.entity().add::<SimConfig>();

    let q = world
        .query::<(&SimConfig, &mut SimTime)>()
        .term_at(0)
        .set_src_id(cfg)
        .term_at(1)
        .set_src_id(game)
        .build();

    // Ok (note that it.count() will be 0)
    q.run(|mut it| {
        while it.next() {
            let sc = it.field::<SimConfig>(0).unwrap();
            let mut st = it.field::<SimTime>(1).unwrap();
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

    // Not ok: there is no entity to pass to first argument
    q.each_entity(|e, (sc, st)| {
        st.value += sc.sim_speed;
    });

    let q = world
        .query::<(&SimConfig, &SimTime)>()
        .term_at(0)
        .set_src_name("cfg")
        .term_at(1)
        .set_src_name("game")
        .build();

    let q = world
        .query::<(&Player, &Position)>()
        .with::<Input>()
        .set_src::<Input>() // match Input on itself
        .build();

    let q = world
        .query::<(&Player, &Position)>()
        .with::<Input>()
        .singleton() // match Input on itself
        .build();

    let q = world
        .query::<(&Player, &Position, &Input)>()
        .term_at(2)
        .singleton() // match Input on itself
        .build();

    // These three queries are the same:
    let q1 = world
        .query::<()>()
        .with::<Mass>()
        .up_type::<flecs::ChildOf>()
        .build();

    let q2 = world
        .query::<()>()
        .with::<Mass>()
        .up() // defaults to .up(flecs::ChildOf)
        .build();

    let q3 = world
        .query::<()>()
        .with::<Mass>()
        .parent() // shortcut for .up(flecs::ChildOf)
        .build();

    // Register an inheritable component 'Mass'
    world
        .component::<Mass>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    // These two queries are the same:
    let q1 = world
        .query::<()>()
        .with::<Mass>()
        .self_()
        .up_type::<flecs::IsA>()
        .build();

    let q2 = world
        .query::<()>()
        .with::<Mass>() // defaults to .self().up(flecs::IsA)
        .build();

    // Register an inheritable component 'Mass'
    world
        .component::<Mass>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().add::<Mass>();

    let parent = world.entity().is_a_id(base); // inherits Mass

    let child = world.entity().child_of_id(parent);

    // Matches 'child', because parent inherits Mass from prefab
    let q = world
        .query::<()>()
        .with::<Mass>()
        .up() // traverses ChildOf upwards
        .build();

    // Register inheritable 'Position' component
    world
        .component::<Position>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().add::<Position>();

    let inst = world.entity().is_a_id(base); // short for .add(flecs::IsA, base);

    // The following two queries are the same:
    let q1 = world.new_query::<&Position>();

    let q2 = world
        .query::<&Position>()
        .term_at(0)
        .self_()
        .up_id(flecs::IsA::ID)
        .build();

    let parent = world.entity().add::<Position>();

    let child = world.entity().child_of_id(parent); // short for .add_id((flecs::ChildOf::ID, base));

    let q = world.query::<&Position>().term_at(0).up().build();

    // Create a new traversable relationship
    let contained_by = world.entity().add::<flecs::Traversable>();

    let parent = world.entity().add::<Position>();

    let child = world.entity().add_id((contained_by, parent));

    let q = world
        .query::<&Position>()
        .term_at(0)
        .up_id(contained_by)
        .build();

    let q = world
        .query::<(&Position, &Mass)>()
        .term_at(0)
        // Never inherit Position
        .self_()
        // Instancing is a property of the iterator, but by setting it on the query
        // all iterators created for the query will be instanced.
        .instanced()
        .build();

    q.run(|mut it| {
        while it.next() {
            let mut p = it.field::<Position>(0).unwrap();
            let m = it.field::<Mass>(1).unwrap();

            if it.is_self(1) {
                // Mass is matched on self, access as array
                for i in it.iter() {
                    p[i].x += 1.0 / m[i].value;
                    p[i].y += 1.0 / m[i].value;
                }
            } else {
                // Mass is matched on other entity, access as single value
                for i in it.iter() {
                    p[i].x += 1.0 / m[0].value; // [0] because it is a single value
                    p[i].y += 1.0 / m[0].value;
                }
            }
        }
    });

    let q = world
        .query::<()>()
        .with::<SpaceShip>()
        .with::<DockedTo>()
        .set_second_name("$Location")
        .with::<Planet>()
        .set_src_name("$Location")
        .build();

    let q = world
        .query::<()>()
        .with::<SpaceShip>()
        .with::<DockedTo>()
        .second()
        .set_var("$Location")
        .with::<Planet>()
        .src()
        .set_var("$Location")
        .build();

    let earth = world.entity();

    let location_var = q.find_var("$Location").unwrap();

    q.iterable().set_var(location_var, earth).each(|it| {
        // iterate as usual
    });

    let earth = world.entity();

    q.iterable().set_var_expr("$Location", earth).each(|it| {
        // iterate as usual
    });

    #[derive(Component, Default)]
    struct Movement {
        value: Entity,
    }

    //TODO META
    /*
    struct Movement {
      flecs::entity_t value;
    };

    // Register 'Movement' component and reflection data
    world.component<Movement>()
      .member("value", &Movement::value);

    // Create two entities for the direction
    flecs::entity Left = world.entity();
    flecs::entity Right = world.entity();

    // Create two entities with different directions
    flecs::entity e1 = world.entity().set(Movement{ Left });
    flecs::entity e2 = world.entity().set(Movement{ Right });

    // Create query that only matches e1
    flecs::query<> q = world.query()
      .with("Movement.value", Left)
      .build();
        */

    // Query used for change detection. Note that change detection is not enabled on
    // the query itself, but by calling change detection functions for the query.
    let q_read = world.new_query::<&Position>();

    // Query used to create changes
    let q_write = world.new_query::<&mut Position>(); // defaults to inout

    // Test if changes have occurred for anything matching the query. If this is the
    // first call to the function, it will enable change detection for the query.
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

    // Use readonly term for component used for sorting
    let q = world
        .query::<(&Depth, &Position)>()
        .order_by::<Depth>(|e1, d1: &Depth, e2, d2: &Depth| {
            (d1.value > d2.value) as i32 - (d1.value < d2.value) as i32
        })
        .build();

    let depth_id = world.component::<Depth>();

    let q = world
        .query::<&Position>()
        .with_id(depth_id)
        .set_in()
        .order_by_id(depth_id, |e1, d1: *const c_void, e2, d2: *const c_void| {
            let d1 = unsafe { &*(d1 as *const Depth) };
            let d2 = unsafe { &*(d2 as *const Depth) };
            (d1.value > d2.value) as i32 - (d1.value < d2.value) as i32
        })
        .build();

    let q = world
        .query::<&Position>()
        .order_by_id(0, |e1, _d1: *const c_void, e2, _d2: *const c_void| {
            (e1 > e2) as i32 - (e1 < e2) as i32
        })
        .build();

    //TODO group by section

    #[derive(Component)]
    struct Unit;

    let unit = world.component::<Unit>();
    let melee_unit = world.entity().is_a::<Unit>();
    let ranged_unit = world.entity().is_a::<Unit>();

    let unit_01 = world.entity().add_id(melee_unit);
    let unit_02 = world.entity().add_id(ranged_unit);

    // Matches entities with Unit, MeleeUnit and RangedUnit
    let q = world.query::<&Unit>();

    // Iterate as usual

    // Create LocatedIn relationship with transitive property
    #[derive(Component)]
    struct LocatedIn;

    world.component::<LocatedIn>().add::<flecs::Transitive>();

    let new_york = world.entity();
    let manhattan = world.entity().add_first::<LocatedIn>(new_york);
    let central_park = world.entity().add_first::<LocatedIn>(manhattan);
    let bob = world.entity().add_first::<LocatedIn>(central_park);

    // Matches ManHattan, CentralPark, Bob
    let q = world
        .query::<()>()
        .with_first::<LocatedIn>(new_york)
        .build();

    // Iterate as usual

    // Matches:
    //  - ManHattan (Place = NewYork)
    //  - CentralPark (Place = ManHattan, NewYork)
    //  - Bob (Place = CentralPark, ManHattan, NewYork)
    let q = world
        .query::<()>()
        .with::<LocatedIn>()
        .set_second_name("$Place")
        .build();

    #[derive(Component)]
    struct City;

    // Add City property to NewYork
    new_york.add::<City>();

    // Matches:
    //  - ManHattan (Place = NewYork)
    //  - CentralPark (Place = NewYork)
    //  - Bob (Place = NewYork)

    let q = world
        .query::<()>()
        .with::<LocatedIn>()
        .set_second_name("$Place")
        .with::<City>()
        .set_src_name("$Place")
        .build();

    let tree = world.entity();
    let oak = world.entity().is_a_id(tree);

    // Matches Tree, Oak
    let q = world.query::<()>().with_first::<flecs::IsA>(tree).build();

    // Iterate as usual
}
