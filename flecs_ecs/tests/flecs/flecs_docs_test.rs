//! Compile tests for the flecs docs in the core C repo.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::print_stdout)]
#![allow(unused_must_use)]
#![allow(path_statements)]
#![allow(clippy::no_effect)]
#![allow(clippy::if_same_then_else)]
use std::os::raw::c_void;

use flecs_ecs::macros::*;
use flecs_ecs::prelude::*;
use flecs_ecs::sys;

#[derive(Component, Default)]
struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default)]
struct Velocity {
    pub x: f32,
    pub y: f32,
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

#[derive(Component, Default, Clone, Copy, PartialEq)]
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

#[derive(Component, Default)]
struct TimeOfDay {
    pub value: f32,
}

#[derive(Component, Default)]
struct Eats {
    amount: u32,
}

#[derive(Component, Default)]
struct Apples;

#[derive(Component, Default)]
struct MaxSpeed {
    value: u32,
}

#[derive(Component, Default)]
struct Defense {
    value: u32,
}

#[derive(Component, Default)]
struct Serializable;

#[derive(Component, Default)]
struct Gravity {
    x: i32,
    y: i32,
}

#[derive(Component, Default)]
struct Transform;

#[derive(Component, Default)]
struct Mesh;

#[derive(Component, Default)]
struct Health {
    value: u32,
}

#[derive(Component, Default)]
struct Archer;

#[derive(Component, Default)]
struct Node;

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
                .field_mut::<Position>(0)
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
                    .field_mut::<Position>(0)
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
    q.run(|mut it| {
        while it.next() {
            let mut p = it.field_mut::<Position>(0).unwrap();
            let v = it.field::<Velocity>(1).unwrap();
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
            }
        }
    });

    // System iteration (run_iter)
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field_mut::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();
                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
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
        .run(|mut it| {
            while it.next() {
                let mut p = it.field_mut::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();
                for i in it.iter() {
                    p[i].x += v[i].x * it.delta_time();
                    p[i].y += v[i].y * it.delta_time();
                }
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
        .run(|mut it| {
            while it.next() {
                println!("Time: {}", it.field::<Game>(0).unwrap()[9].time);
            }
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

    world.system::<&Position>().write::<Transform>().each(|p| {
        // ...
    });

    world.system::<&Position>().read::<Transform>().each(|p| {
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
        .set_interval(1.0) // Run at 1Hz
        .each(|p| {
            // ...
        });

    world
        .system::<&Position>()
        .set_rate(2) // Run every other frame
        .each(|p| {
            // ...
        });

    // A rate filter can be created with .set_rate(2)
    //let tick_source = world.timer().set_interval(1.0);
    //TODO

    // world
    //     .system::<(&Position, &Velocity)>()
    //     .tick_source_id(tick_source) // Set tick source for system
    //     .each(|(p, v)| {
    //         // ...
    //     });

    //TODO

    //
    // // Pause timer
    // tick_source.stop();

    // // Resume timer
    // tick_source.start();
    //
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
    world.delete_empty_tables(sys::ecs_delete_empty_tables_desc_t {
        id: 0,
        clear_generation: 10,
        delete_generation: 0,
        min_id_count: 0,
        time_budget_seconds: 0.0,
    });

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
            let mut p = it.field_mut::<Position>(0).unwrap();
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
            let p = it.field_mut::<Position>(0).unwrap();
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
            let p = it.field_mut::<Position>(0).unwrap();
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
            let p = it.field_mut::<Position>(0).unwrap();
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
            let mut p = it.field_mut::<Position>(0).unwrap();
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
            let mut st = it.field_mut::<SimTime>(1).unwrap();
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
        .build();

    q.run(|mut it| {
        while it.next() {
            let mut p = it.field_mut::<Position>(0).unwrap();
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
    world.component::<Movement>()
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

    // Create locatedin relationship with transitive property
    #[derive(Component)]
    struct LocatedIn;

    world.component::<LocatedIn>().add::<flecs::Transitive>();

    let new_york = world.entity();
    let manhattan = world.entity().add_first::<LocatedIn>(new_york);
    let central_park = world.entity().add_first::<LocatedIn>(manhattan);
    let bob = world.entity().add_first::<LocatedIn>(central_park);

    // Matches ManHattan, CentralPark, bob
    let q = world
        .query::<()>()
        .with_first::<LocatedIn>(new_york)
        .build();

    // Iterate as usual

    // Matches:
    //  - ManHattan (Place = newyork)
    //  - CentralPark (Place = ManHattan, newyork)
    //  - bob (Place = CentralPark, ManHattan, newyork)
    let q = world
        .query::<()>()
        .with::<LocatedIn>()
        .set_second_name("$Place")
        .build();

    #[derive(Component)]
    struct City;

    // Add City property to newyork
    new_york.add::<City>();

    // Matches:
    //  - ManHattan (Place = newyork)
    //  - CentralPark (Place = newyork)
    //  - bob (Place = newyork)

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

fn flecs_entities_components_docs_compile_test() {
    let world = World::new();

    let my_entity = world.entity();

    my_entity.destruct();

    let e1 = world.entity(); // Returns 500v0
    e1.destruct(); // Recycles 500
    let e2 = world.entity(); // Returns 500v1
                             // Fails, 500v0 is not alive
    e1.add::<Npc>();
    // OK, 500v1 is alive
    e2.add::<Npc>();

    let e1 = world.entity();
    e1.destruct();
    e1.destruct(); // OK: post condition is satisfied

    my_entity.clear();

    let e1 = world.entity();
    let e2 = world.entity();
    e1.destruct();
    e1.is_alive(); // False
    e2.is_alive(); // True

    let e1 = world.entity();
    let e2 = world.entity();
    e1.destruct();
    e1.is_valid(); // False
    world.entity_from_id(0).is_valid(); // False

    let e = world.make_alive(1000);

    //TODO does not exist yet
    //world.set_version(versioned_id);

    world.set_entity_range(5000, 0);

    world.set_entity_range(5000, 10000);

    world.enable_range_check(true);

    let e = world.entity_named("MyEntity");
    if e == world.lookup("MyEntity") {
        // true
    }
    println!("{}", e.name());

    let p = world.entity_named("Parent");
    let e = world.entity_named("Child").child_of_id(p);
    if e == world.lookup("Parent::Child") {
        // true
    }

    let p = world.entity_named("Parent");
    let e = world.entity_named("Child").child_of_id(p);
    if e == p.lookup("Child") {
        // true
    }

    let p = world.entity_named("Parent");
    let e = world.entity_named("Child").child_of_id(p);
    // Returns entity name, does not allocate
    println!("{}", e.name()); // Child
                              // Returns entity path, does allocate
    println!("{}", e.path().unwrap()); // Parent.Child

    let e1 = world.entity_named("Parent::Child");
    let e2 = world.entity_named("Parent::Child");
    if e1 == e2 {
        // true
    }

    let e = world.entity_named("Foo");
    // Change name
    e.set_name("Bar");

    let ten = world.entity_named("10");
    let twenty = world.entity_named("20");

    let e = world.entity();
    // Enable entity
    e.enable_self();
    // Disable entity
    e.disable_self();

    // Three entities to disable
    let e1 = world.entity();
    let e2 = world.entity();
    let e3 = world.entity();
    // Create prefab that has the three entities
    let p = world.prefab();
    p.add_id(e1);
    p.add_id(e2);
    p.add_id(e3);
    // Disable entities
    p.disable_self();
    // Enable entities
    p.enable_self();

    // Three entities to disable
    let e1 = world.entity();
    let e2 = world.entity();
    let e3 = world.entity();

    // Create prefab hierarchy with the three entities
    let p1 = world.prefab().add_id(e1);
    let p2 = world.prefab().is_a_id(p1).add_id(e2).add_id(e3);

    // Disable e1, e2, e3
    p2.disable_self();

    // Enable e1
    p1.enable_self();

    e.add::<flecs::Disabled>();

    // Get the entity for the Position component
    let pos = world.component::<Position>();
    // Component entities have the Component component
    pos.get::<&flecs::Component>(|comp_data| {
        println!(
            "size: {}, alignment: {}",
            comp_data.size, comp_data.alignment
        );
    });

    // Register a sparse component
    world.component::<Position>().add_trait::<flecs::Sparse>();

    fn main() {
        let world = World::new();
        let e1 = world
            .entity()
            .set(Position { x: 10.0, y: 20.0 }) // Position registered here
            .set(Velocity { x: 1.0, y: 2.0 }); // Velocity registered here
        let e2 = world
            .entity()
            .set(Position { x: 10.0, y: 20.0 }) // Position already registered
            .set(Velocity { x: 1.0, y: 2.0 }); // Velocity already registered
    }

    world.component::<Position>();

    use flecs_ecs::prelude::*;
    #[derive(Component)]
    struct Movement;
    impl Module for Movement {
        fn module(world: &World) {
            world.module::<Movement>("Movement");
            // Define components, systems, triggers, ... as usual. They will be
            // letmatically created inside the scope of the module.
        }
    }
    let world = World::new();
    world.import::<Movement>();

    //TODO Rust API does not exist yet
    // let desc = sys::ecs_component_desc_t {
    //     type_: sys::ecs_type_info_t {
    //         size: 8,
    //         alignment: 8,
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // };
    // let comp = sys::ecs_component_init(world, &desc);
    // let e = world.entity();
    // // Add component
    // e.add(comp);
    // // Get component
    // let ptr = e.get(comp);

    //ToDO Rust API does not exist yet
    // let desc = ecs_component_desc_t {
    //     entity: world.entity_named("MyComponent"),
    //     size: 8,
    //     alignment: 8,
    //     ..Default::default()
    // };
    // let comp = ecs_component_init(world, &desc);
    // let e = world.entity();
    // // Add component
    // e.add(comp);
    // // Get component
    // let ptr = e.get(comp);

    let pos = world.component::<Position>();

    // Create entity with Position
    let e = world.entity().add::<Position>();

    // Unregister the component
    pos.destruct();

    // Position is removed from e

    // Set singleton
    world.set(TimeOfDay { value: 0.5 });
    // Get singleton
    world.get::<&TimeOfDay>(|time| println!("{}", time.value));

    // Set singleton
    world.set(TimeOfDay { value: 0.5 });
    // Equivalent to:
    world.component::<TimeOfDay>().set(TimeOfDay { value: 0.5 });

    // Register toggle-able component
    world
        .component::<Position>()
        .add_trait::<flecs::CanToggle>();
    let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    // Disable component
    e.disable::<Position>();
    assert!(!e.is_enabled::<Position>()); // False
                                          // Enable component
    e.enable::<Position>();
    assert!(e.is_enabled::<Position>()); // True
}

fn flecs_docs_relationships_compile_test() {
    let world = World::new();

    let likes = world.entity();
    let bob = world.entity();
    let alice = world.entity();

    // bob likes alice
    bob.add_id((likes, alice));

    // bob likes alice no more
    bob.remove_id((likes, alice));

    let bob = world.entity();
    let eats = world.entity();
    let apples = world.entity();
    let pears = world.entity();
    bob.add_id((eats, apples));
    bob.add_id((eats, pears));
    bob.has_id((eats, apples)); // true
    bob.has_id((eats, pears)); // true

    // Find all entities that eat apples
    let q = world.query::<()>().expr("(Eats, Apples)").build();
    // Find all entities that eat anything
    let q = world.query::<()>().expr("(Eats, *)").build();
    // With the query builder API:
    let q = world.query::<()>().with_id((eats, apples)).build();
    // Or when using pair types, when both relationship & target are compile time types, they can be represented as a tuple:
    let q = world.new_query::<&(Eats, Apples)>();

    bob.has_id((eats, apples));

    bob.has_id((eats, flecs::Wildcard::ID));

    let parent = bob.parent();

    let food = bob.target_id(eats, 0); // first target

    let mut index = 0;
    while bob.target_id(eats, index).is_some() {
        index += 1;
    }

    let parent = bob.target_for::<Position>(flecs::ChildOf::ID);

    bob.each_component(|id| {
        if id.is_pair() {
            let first = id.first_id();
            let second = id.second_id();
        }
    });

    world
        .query::<()>()
        .with_id((eats, apples))
        .build()
        .each_entity(|e, _| {
            // Iterate as usual
        });

    world
        .query::<()>()
        .with_id((eats, flecs::Wildcard::ID))
        .build()
        .each_iter(|it, i, _| {
            let food = it.pair(0).unwrap().second_id(); // Apples, ...
            let e = it.entity(i);
            // Iterate as usual
        });

    let parent = world.entity();

    parent.each_child(|child| {
        // ...
    });

    // Empty types (types without members) are letmatically interpreted as tags
    #[derive(Component)]
    struct Begin;
    #[derive(Component)]
    struct End;
    // Tags
    let likes = world.entity();
    let apples = world.entity();
    let e = world.entity();
    // Both likes and Apples are tags, so (likes, Apples) is a tag
    e.add_id((likes, apples));
    // Eats is a type and Apples is a tag, so (Eats, Apples) has type Eats
    e.set_pair::<Eats, Apples>(Eats { amount: 1 });
    // Begin is a tags and Position is a type, so (Begin, Position) has type Position
    e.set_pair::<Begin, Position>(Position { x: 10.0, y: 20.0 });
    e.set_pair::<End, Position>(Position { x: 100.0, y: 20.0 }); // Same for End
                                                                 // ChildOf has the Tag property, so even though Position is a type, the pair
                                                                 // does not assume the Position type
    e.add_id((flecs::ChildOf::ID, world.component_id::<Position>()));
    e.add::<(flecs::ChildOf, Position)>();

    let e = world.entity();
    let first = world.entity();
    let second = world.entity();
    let third = world.entity();
    // Add component position 3 times, for 3 different objects
    e.set_first::<Position>(Position { x: 1.0, y: 2.0 }, first);
    e.set_first::<Position>(Position { x: 3.0, y: 4.0 }, second);
    e.set_first::<Position>(Position { x: 5.0, y: 6.0 }, third);

    let q = world
        .query::<()>()
        .with_id((likes, flecs::Wildcard::ID))
        .build();
    q.each_iter(|it, i, _| {
        println!(
            "entity {} has relationship {} {}",
            it.entity(i),
            it.pair(0).unwrap().first_id().name(),
            it.pair(0).unwrap().second_id().name()
        );
    });

    let q = world.query::<()>().expr("(likes, *)").build();

    // bob eats apples and pears
    let bob = world.entity();
    let eats = world.entity();
    let apples = world.entity();
    let pears = world.entity();
    bob.add_id((eats, apples));
    bob.add_id((eats, pears));
    // Find all (Eats, *) relationships in bob's type
    bob.each_pair(eats, flecs::Wildcard::ID, |id| {
        println!("bob eats {}", id.second_id().name());
    });
    // For target wildcard pairs, each_target_id() can be used:
    bob.each_target_id(eats, |entity| {
        println!("bob eats {}", entity.name());
    });

    let apple = world.entity();
    let fruit = world.entity();
    apple.add_id((flecs::IsA::ID, fruit));

    apple.is_a_id(fruit);

    let granny_smith = world.entity();
    granny_smith.add_id((flecs::IsA::ID, apple));

    let spaceship = world
        .entity()
        .set(MaxSpeed { value: 100 })
        .set(Defense { value: 50 });
    let frigate = world
        .entity()
        .is_a_id(spaceship) // shorthand for .add(flecs::IsA, Spaceship)
        .set(Defense { value: 75 });

    // Obtain the inherited component from Spaceship
    let is_100 = frigate.get::<&mut MaxSpeed>(|v| {
        v.value == 100 // True
    });

    // Obtain the overridden component from Frigate
    let is_75 = frigate.get::<&mut Defense>(|v| {
        v.value == 75 // True
    });

    let fast_frigate = world.entity().is_a_id(frigate).set(MaxSpeed { value: 200 });
    // Obtain the overridden component from FastFrigate
    let is_200 = fast_frigate.get::<&mut MaxSpeed>(|v| {
        v.value == 200 // True
    });
    // Obtain the inherited component from Frigate
    let is_75 = fast_frigate.get::<&mut Defense>(|v| {
        v.value == 75 // True
    });

    let spaceship = world.entity();
    let cockpit = world.entity();
    cockpit.add_id((flecs::ChildOf::ID, spaceship));

    cockpit.child_of_id(spaceship);

    let parent = world.entity_named("Parent");
    let child = world.entity_named("Child").child_of_id(parent);
    child == world.lookup("Parent::Child"); // true
    child == parent.lookup("Child"); // true

    let parent = world.entity();
    let prev = world.set_scope_id(parent);
    let child_a = world.entity();
    let child_b = world.entity();
    // Restore the previous scope
    world.set_scope_id(prev);
    child_a.has_id((flecs::ChildOf::ID, parent)); // true
    child_b.has_id((flecs::ChildOf::ID, parent)); // true

    let parent = world.entity().run_in_scope(|| {
        let child_a = world.entity();
        let child_b = world.entity();
        child_a.has_id((flecs::ChildOf::ID, parent)); // true
        child_b.has_id((flecs::ChildOf::ID, parent)); // true
    });
}

fn flecs_docs_quick_start_compile_test() {
    let world = World::new();
    let eats = world.entity();
    let apples = world.entity();
    let pears = world.entity();
    let grows = world.entity();

    let world = World::new();

    // Do the ECS stuff

    let e = world.entity();
    e.is_alive(); // true!

    e.destruct();
    e.is_alive(); // false!

    let e = world.entity_named("bob");

    println!("Entity name: {}", e.name());

    let e = world.lookup("bob");

    let e = world.entity();

    // Add a component. This creates the component in the ECS storage, but does not
    // assign it with a value.
    e.add::<Velocity>();

    // Set the value for the Position & Velocity components. A component will be
    // added if the entity doesn't have it yet.
    e.set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    // Get a component
    e.get::<&Position>(|p| {
        println!("Position: ({}, {})", p.x, p.y);
    });

    // Remove component
    e.remove::<Position>();

    //Rust applications can use the `world::entity_from` function.

    let pos_e = world.entity_from::<Position>();
    println!("Name: {}", pos_e.name()); // outputs 'Name: Position'

    // It's possible to add components like you would for any entity
    pos_e.add::<Serializable>();

    let pos_e = world.entity_from::<Position>();

    pos_e.get::<&flecs::Component>(|c| {
        println!("Component size: {}", c.size);
    });

    // Option 1: create Tag as empty struct
    #[derive(Component)]
    struct Enemy;

    // Create entity, add Enemy tag
    let e = world.entity().add::<Enemy>();
    e.has::<Enemy>(); // true!

    e.remove::<Enemy>();
    e.has::<Enemy>(); // false!

    // Option 2: create Tag as entity
    let enemy = world.entity();

    // Create entity, add Enemy tag
    let e = world.entity().add_id(enemy);
    e.has_id(enemy); // true!

    e.remove_id(enemy);
    e.has_id(enemy); // false!

    // Create Likes relationship as empty type (tag)
    #[derive(Component)]
    struct Likes;

    // Create a small graph with two entities that like each other
    let bob = world.entity();
    let alice = world.entity();

    bob.add_first::<Likes>(alice); // bob likes alice
    alice.add_first::<Likes>(bob); // alice likes bob
    bob.has_first::<Likes>(alice); // true!

    bob.remove_first::<Likes>(alice);
    bob.has_first::<Likes>(alice); // false!

    let id = world.id_first::<Likes>(bob);

    let id = world.id_from::<(Likes, Apples)>();
    if id.is_pair() {
        let relationship = id.first_id();
        let target = id.second_id();
    }

    let bob = world.entity();
    bob.add_id((eats, apples));
    bob.add_id((eats, pears));
    bob.add_id((grows, pears));

    bob.has_id((eats, apples)); // true!
    bob.has_id((eats, pears)); // true!
    bob.has_id((grows, pears)); // true!

    let alice = world.entity().add_first::<Likes>(bob);
    let o = alice.target::<Likes>(0); // Returns bob

    let parent = world.entity();
    let child = world.entity().child_of_id(parent);

    // Deleting the parent also deletes its children
    parent.destruct();

    let parent = world.entity_named("parent");
    let child = world.entity_named("child").child_of_id(parent);
    println!("Child path: {}", child.path().unwrap()); // output: 'parent::child'

    world.lookup("parent::child"); // returns child
    parent.lookup("child"); // returns child

    let q = world
        .query::<(&Position, &mut Position)>()
        .term_at(1)
        .parent()
        .cascade()
        .build();

    q.each(|(p, p_parent)| {
        // Do the thing
    });

    let e = world.entity().add::<Position>().add::<Velocity>();

    println!("Components: {}", e.archetype().to_string().unwrap()); // output: 'Position,Velocity'

    e.each_component(|id| {
        if id == world.component_id::<Position>() {
            // Found Position component!
        }
    });

    // Set singleton component
    world.set(Gravity { x: 10, y: 20 });

    // Get singleton component
    world.get::<&Gravity>(|g| {
        println!("Gravity: {}, {}", g.x, g.y);
    });

    let grav_e = world.entity_from::<Gravity>();

    grav_e.set(Gravity { x: 10, y: 20 });

    grav_e.get::<&Gravity>(|g| {
        println!("Gravity: {}, {}", g.x, g.y);
    });

    world
        .query::<(&Velocity, &Gravity)>()
        .term_at(1)
        .singleton()
        .build();

    // For simple queries the world::each function can be used
    world.each::<(&mut Position, &Velocity)>(|(p, v)| {
        // EntityView argument is optional, use each_entity to get it
        p.x += v.x;
        p.y += v.y;
    });

    // More complex queries can first be created, then iterated
    let q = world
        .query::<&Position>()
        .with_id((flecs::ChildOf::ID, parent))
        .build();

    // Option 1: the each() callback iterates over each entity
    q.each_entity(|e, p| {
        println!("{}: ({}, {})", e.name(), p.x, p.y);
    });

    // Option 2: the run() callback offers more control over the iteration
    q.run(|mut it| {
        while it.next() {
            let p = it.field_mut::<Position>(0).unwrap();

            for i in it.iter() {
                println!("{}: ({}, {})", it.entity(i).name(), p[i].x, p[i].y);
            }
        }
    });

    let q = world
        .query::<()>()
        .with::<(flecs::ChildOf, flecs::Wildcard)>()
        .with::<Position>()
        .set_oper(OperKind::Not)
        .build();

    // Iteration code is the same

    // Use each_entity() function that iterates each individual entity
    let move_sys = world
        .system::<(&mut Position, &Velocity)>()
        .each_iter(|it, i, (p, v)| {
            p.x += v.x * it.delta_time();
            p.y += v.y * it.delta_time();
        });

    // Just like with queries, systems have both the run() and
    // each() methods to iterate entities.

    move_sys.run();

    println!("System: {}", move_sys.name());
    move_sys.add::<flecs::pipeline::OnUpdate>();
    move_sys.destruct();

    flecs::pipeline::OnLoad;
    flecs::pipeline::PostLoad;
    flecs::pipeline::PreUpdate;
    flecs::pipeline::OnUpdate;
    flecs::pipeline::OnValidate;
    flecs::pipeline::PostUpdate;
    flecs::pipeline::PreStore;
    flecs::pipeline::OnStore;

    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .kind::<flecs::pipeline::OnUpdate>()
        .each(|(p, v)| {});
    world
        .system_named::<(&mut Position, &Transform)>("Transform")
        .kind::<flecs::pipeline::PostUpdate>()
        .each(|(p, t)| {});
    world
        .system_named::<(&Transform, &mut Mesh)>("Render")
        .kind::<flecs::pipeline::OnStore>()
        .each(|(t, m)| {});

    world.progress();

    move_sys.add::<flecs::pipeline::OnUpdate>();
    move_sys.remove::<flecs::pipeline::PostUpdate>();

    world
        .observer_named::<flecs::OnSet, (&Position, &Velocity)>("OnSetPosition")
        .each(|(p, v)| {}); // Callback code is same as system

    let e = world.entity(); // Doesn't invoke the observer
    e.set(Position { x: 10.0, y: 20.0 }); // Doesn't invoke the observer
    e.set(Velocity { x: 1.0, y: 2.0 }); // Invokes the observer
    e.set(Position { x: 30.0, y: 40.0 }); // Invokes the observer

    #[derive(Component)]
    struct MyModule;

    impl Module for MyModule {
        fn module(world: &World) {
            world.module::<MyModule>("MyModule");
            // Define components, systems, triggers, ... as usual. They will be
            // letmatically created inside the scope of the module.
        }
    }

    // Import code
    world.import::<MyModule>();
}

fn flecs_docs_observers_compile_test() {
    let world = World::new();

    // Create observer that is invoked whenever Position is set
    world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(|e, p| {
            println!("Position set: {{ {}, {} }}", p.x, p.y);
        });

    world.entity().set(Position { x: 10.0, y: 20.0 }); // Invokes observer

    let e = world.entity();

    // OnAdd observer fires
    e.add::<Position>();

    // OnAdd observer doesn't fire, entity already has component
    e.add::<Position>();

    let e = world.entity();

    // OnAdd observer fires first, then OnSet observer fires
    e.set(Position { x: 10.0, y: 20.0 });

    // OnAdd observer doesn't fire, OnSet observer fires
    e.set(Position { x: 10.0, y: 20.0 });

    let p = world.prefab().set(Position { x: 10.0, y: 20.0 });

    // Produces OnSet event for Position
    let i = world.entity().is_a_id(p);

    let p = world.prefab().set(Position { x: 10.0, y: 20.0 });

    // Produces OnSet event for inherited Position component
    let i = world.entity().is_a_id(p);

    // Override component. Produces regular OnSet event.
    i.set(Position { x: 20.0, y: 30.0 });

    // Reexposes inherited component, produces OnSet event
    i.remove::<Position>();

    let p = world.prefab().set(Position { x: 10.0, y: 20.0 });

    // Produces OnSet event for Position
    let i = world.entity().is_a_id(p);

    let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    // OnRemove observer fires
    e.remove::<Position>();

    // OnRemove observer doesn't fire, entity doesn't have the component
    e.remove::<Position>();

    // Observer that listens for both OnAdd and OnRemove events
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .add_event::<flecs::OnRemove>()
        .each_entity(|e, p| {
            // ...
        });

    world
        .observer::<flecs::OnAdd, ()>()
        .add_event::<flecs::OnRemove>()
        .with::<Position>()
        .each_iter(|it, i, p| {
            if it.event() == flecs::OnAdd::ID {
                // ...
            } else if it.event() == flecs::OnRemove::ID {
                // ...
            }
        });

    // Observer that listens for all events for Position
    world
        .observer::<flecs::Wildcard, &Position>()
        .each_entity(|e, p| {
            // ...
        });

    // Observer that listens for entities with both Position and Velocity
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .with::<Velocity>()
        .each_entity(|e, _| {
            // ...
        });

    let e = world.entity();

    // Does not trigger "Position, Velocity" observer
    e.add::<Position>();

    // Entity now matches "Position, Velocity" query, triggers observer
    e.add::<Velocity>();

    // Observer that only triggers on Position, not on Velocity
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .with::<Velocity>()
        .filter()
        .each_entity(|e, p| {
            // ...
        });

    let e = world.entity();

    // Doesn't trigger, entity doesn't have Velocity
    e.set(Position { x: 10.0, y: 20.0 });

    // Doesn't trigger, Velocity is a filter term
    e.set(Velocity { x: 1.0, y: 2.0 });

    // Triggers, entity now matches observer query
    e.set(Position { x: 20.0, y: 30.0 });

    // OnSet observer with both component and tag
    world
        .observer::<flecs::OnSet, &Position>()
        .with::<Npc>() // Tag
        .each_entity(|e, p| {
            // ...
        });

    let e = world.entity();

    // Doesn't trigger, entity doesn't have Npc
    e.set(Position { x: 10.0, y: 20.0 });

    // Produces and OnAdd event & triggers observer
    e.add::<Npc>();

    // Produces an OnSet event & triggers observer
    e.set(Position { x: 20.0, y: 30.0 });

    // Observer with a Not term
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .without::<Velocity>()
        .each_entity(|e, p| {
            // ...
        });

    let e = world.entity();

    // Triggers the observer
    e.set(Position { x: 10.0, y: 20.0 });

    // Doesn't trigger the observer, entity doesn't match the observer query
    e.set(Velocity { x: 1.0, y: 2.0 });

    // Triggers the observer, as the Velocity term was inverted to OnRemove
    e.remove::<Velocity>();

    // Monitor observer
    world
        .observer::<flecs::Monitor, (&Position, &Velocity)>()
        .each_iter(|it, i, (p, v)| {
            if it.event() == flecs::OnAdd::ID {
                // Entity started matching query
            } else if it.event() == flecs::OnRemove::ID {
                // Entity stopped matching query
            }
        });

    let e = world.entity();

    // Doesn't trigger the monitor, entity doesn't match
    e.set(Position { x: 10.0, y: 20.0 });

    // Entity now matches, triggers monitor with OnAdd event
    e.set(Velocity { x: 1.0, y: 2.0 });

    // Entity no longer matches, triggers monitor with OnRemove event
    e.remove::<Position>();

    // Entity created before the observer
    let e1 = world.entity().set(Position { x: 10.0, y: 20.0 });

    // Yield existing observer
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .with::<Velocity>()
        .yield_existing()
        .each_iter(|it, i, _| {
            // ...
        });

    // Observer is invoked for e1

    // Fires observer as usual
    let e2 = world.entity().set(Position { x: 10.0, y: 20.0 });

    // Entity used for fixed source
    let game = world.entity().set(TimeOfDay { value: 0.0 });

    // Observer with fixed source
    world
        .observer::<flecs::OnSet, &TimeOfDay>()
        .term_at(0)
        .set_src_id(game) // Match TimeOfDay on game
        .each_iter(|it, i, time| {
            // ...
        });

    // Triggers observer
    game.set(TimeOfDay { value: 1.0 });

    // Does not trigger observer
    let e = world.entity().set(TimeOfDay { value: 0.0 });

    world.set(TimeOfDay { value: 0.0 });

    // Observer with singleton source
    world
        .observer::<flecs::OnSet, &TimeOfDay>()
        .term_at(0)
        .singleton()
        .each_iter(|it, i, time| {
            // ...
        });

    // Triggers observer
    world.set(TimeOfDay { value: 1.0 });

    // Does not trigger observer
    let e = world.entity().set(TimeOfDay { value: 0.0 });

    // Create an observer that matches OnSet(Position) events on self and a parent
    world
        .observer::<flecs::OnSet, &Position>()
        .term_at(0)
        .self_()
        .up() // .trav(flecs::ChildOf) (default)
        .each_entity(|e, p| {
            // ...
        });

    let parent = world.entity();
    let child = world.entity().child_of_id(parent);

    // Invokes observer twice: once for the parent and once for the child
    parent.set(Position { x: 10.0, y: 20.0 });

    // Create an observer that matches OnAdd(Position) events on a parent
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .term_at(0)
        .up() // .trav(flecs::ChildOf) (default)
        .each_entity(|e, _| {
            // ...
        });

    let parent = world.entity().set(Position { x: 10.0, y: 20.0 });

    // Forwards OnAdd event for Position to child
    let child = world.entity().child_of_id(parent);

    // Create a custom event
    #[derive(Component)]
    struct Synchronized;

    // Alternatively, an plain entity could also be used as event
    // let Synchronized = world.entity();

    // Create an observer that matches a custom event
    world
        .observer::<Synchronized, &Position>()
        .each_entity(|e, p| {
            // ...
        });

    let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    // Emit custom event
    world
        .event()
        .add::<Position>()
        .entity(e)
        .emit(&Synchronized);

    // Create a custom event
    #[derive(Component)]
    struct Clicked;

    // Create entity
    let widget = world.entity_named("widget");

    // Create an entity observer
    widget.observe::<Clicked>(|| {
        // ...
    });

    // Emit entity event
    widget.emit(&Clicked);

    // Create a custom event
    #[derive(Component)]
    struct Resize {
        width: u32,
        height: u32,
    }

    // Create entity
    let widget = world.entity_named("widget");

    // Create an entity observer
    widget.observe_payload::<&Resize>(|r| {
        // ...
    });

    // Emit entity event
    widget.emit(&Resize {
        width: 100,
        height: 200,
    });

    world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(|e, p| {
            // ...
        });

    // Observer is invoked as part of operation
    e.set(Position { x: 10.0, y: 20.0 });

    world.defer_begin();
    e.set(Position { x: 20.0, y: 30.0 });
    // Operation is delayed until here, observer is also invoked here
    world.defer_end();
}

fn flecs_docs_prefabs_compile_test() {
    let world = World::new();

    #[derive(Component)]
    struct Defense {
        value: u32,
    }

    // Create a spaceship prefab with a Defense component.
    let spaceship = world.prefab_named("spaceship").set(Defense { value: 50 });

    // Create two prefab instances
    let inst_1 = world.entity().is_a_id(spaceship);
    let inst_2 = world.entity().is_a_id(spaceship);

    // Get instantiated component
    inst_1.get::<&Defense>(|defense| {
        println!("Defense value: {}", defense.value);
    });

    let myprefab = world.entity().add::<flecs::Prefab>();

    // or the shortcut

    let myprefab = world.prefab();

    // Only match prefab entities
    world.query::<&Position>().with::<flecs::Prefab>().build();

    // Only match prefab entities
    world
        .query::<&Position>()
        .with::<flecs::Prefab>()
        .optional()
        .build();

    // Only match prefab entities
    world
        .query::<&Position>()
        .query_flags(QueryFlags::MatchPrefab)
        .build();

    // Make Defense component inheritable
    world
        .component::<Defense>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    // Create prefab
    let spaceship = world
        .prefab()
        .set(Health { value: 100 })
        .set(Defense { value: 50 });

    // Create prefab instance
    let inst = world.entity().is_a_id(spaceship);

    // Component is retrieved from instance
    inst.get::<&Health>(|health| {
        println!("Health value: {}", health.value);
    });

    // Component is retrieved from prefab
    inst.get::<&Defense>(|defense| {
        println!("Defense value: {}", defense.value);
    });

    if inst.owns::<Defense>() {
        // not inherited
    }

    let inherited_from = inst.target::<Defense>(0);
    if inherited_from.is_none() {
        // not inherited
    }

    // Make Defense component inheritable
    world
        .component::<Defense>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    // Create prefab
    let spaceship = world.prefab().set(Defense { value: 50 });

    // Create prefab instance
    let inst_a = world.entity().is_a_id(spaceship);
    let inst_b = world.entity().is_a_id(spaceship);

    // Override Defense only for inst_a
    inst_a.set(Defense { value: 75 });

    // Make Defense component inheritable
    world
        .component::<Defense>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    // Create prefab
    let spaceship = world.prefab().set(Defense { value: 50 });

    // Create prefab instance
    let inst_a = world.entity().is_a_id(spaceship);
    let inst_b = world.entity().is_a_id(spaceship);

    // Override Defense only for inst_a
    inst_a.add::<Defense>(); // Initialized with value 50

    // Make Defense component inheritable
    world
        .component::<Defense>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    // Create prefab
    let spaceship = world.prefab().set_auto_override(Defense { value: 50 }); // Set & let override Defense

    // Create prefab instance
    let inst = world.entity().is_a_id(spaceship);
    inst.owns::<Defense>(); // true

    // Create prefab
    let spaceship = world
        .prefab_named("spaceship")
        .set(Defense { value: 50 })
        .set(Health { value: 100 });

    // Create prefab variant
    let freighter = world
        .prefab_named("Freighter")
        .is_a_id(spaceship)
        .set(Health { value: 150 }); // Override the Health component of the freighter

    // Create prefab instance
    let inst = world.entity().is_a_id(freighter);
    inst.get::<&Health>(|health| {
        println!("Health value: {}", health.value); // 150
    });
    inst.get::<&Defense>(|defense| {
        println!("Defense value: {}", defense.value); // 50
    });

    let spaceship = world.prefab_named("Spaceship");
    let cockpit = world.prefab_named("Cockpit").child_of_id(spaceship);

    // Instantiate the prefab hierarchy
    let inst = world.entity().is_a_id(spaceship);

    // Lookup instantiated child
    let inst_cockpit = inst.lookup("Cockpit");

    let spaceship = world.prefab_named("Spaceship");
    let cockpit = world.prefab_named("Cockpit").child_of_id(spaceship).slot(); // Defaults to (SlotOf, spaceship)

    // Instantiate the prefab hierarchy
    let inst = world.entity().is_a_id(spaceship);

    // Lookup instantiated child
    let inst_cockpit = inst.target_id(cockpit, 0);

    #[derive(Component)]
    struct Spaceship;

    // Create prefab associated with the spaceship type
    world
        .prefab_type::<Spaceship>()
        .set(Defense { value: 50 })
        .set(Health { value: 100 });

    // Instantiate prefab with type
    let inst = world.entity().is_a::<Spaceship>();

    // Lookup prefab handle
    let prefab = world.lookup("spaceship");
}

fn flecs_docs_component_traits_compile_test() {
    let world = World::new();
    let e = world.entity();
    let parent = world.entity();
    let archer = world.entity();

    #[derive(Component)]
    struct MyComponent {
        e: Entity, // Not covered by cleanup traits
    }

    e.child_of_id(parent); // Covered by cleanup traits

    world.remove_all_id(archer);

    world.remove_all_id(archer);
    world.remove_all_id((archer, flecs::Wildcard::ID));
    world.remove_all_id((flecs::Wildcard::ID, archer));

    // Remove Archer from entities when Archer is deleted
    world
        .component::<Archer>()
        .add_trait::<(flecs::OnDelete, flecs::Remove)>();

    let e = world.entity().add::<Archer>();

    // This will remove Archer from e
    world.component::<Archer>().destruct();

    // Delete entities with Archer when Archer is deleted
    world
        .component::<Archer>()
        .add_trait::<(flecs::OnDelete, flecs::Delete)>();

    let e = world.entity().add::<Archer>();

    // This will delete e
    world.component::<Archer>().destruct();

    // Delete children when deleting parent
    world
        .component::<flecs::ChildOf>()
        .add_trait::<(flecs::OnDeleteTarget, flecs::Delete)>();

    let p = world.entity();
    let e = world.entity().add_first::<flecs::ChildOf>(p);

    // This will delete both p and e
    p.destruct();

    world
        .observer::<flecs::OnRemove, &Node>()
        .each_entity(|e, node| {
            // This observer will be invoked when a Node is removed
        });

    let p = world.entity().add::<Node>();
    let c = world.entity().add::<Node>().child_of_id(p);

    {
        #[derive(Component)]
        struct Serializable;

        world
            .component::<Serializable>()
            .add_trait::<flecs::Trait>();
    }

    {
        #[derive(Component)]
        struct Likes;
        #[derive(Component)]
        struct Apples;

        world
            .component::<Likes>()
            .add_trait::<flecs::Relationship>();

        let e = world
            .entity()
            .add::<Likes>() // Panic, 'Likes' is not used as relationship
            .add::<(Apples, Likes)>() // Panic, 'Likes' is not used as relationship, but as target
            .add::<(Likes, Apples)>(); // OK
    }
    {
        #[derive(Component)]
        struct Likes;
        #[derive(Component)]
        struct Loves;

        world
            .component::<Likes>()
            .add_trait::<flecs::Relationship>();

        // Even though Likes is marked as relationship and used as target here, this
        // won't panic as With is marked as trait.
        world
            .component::<Loves>()
            .add_trait::<(flecs::With, Likes)>();
    }

    #[derive(Component)]
    struct Likes;
    #[derive(Component)]
    struct Apples;

    world.component::<Apples>().add_trait::<flecs::Target>();

    let e = world
        .entity()
        .add::<Apples>() // Panic, 'Apples' is not used as target
        .add::<(Apples, Likes)>() // Panic, 'Apples' is not used as target, but as relationship
        .add::<(Likes, Apples)>(); // OK

    #[derive(Component)]
    struct Serializable; // Tag, contains no data

    impl flecs::FlecsTrait for Serializable {}

    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }

    let e = world
        .entity()
        .set(Position { x: 10.0, y: 20.9 })
        .add_trait::<(Serializable, Position)>(); // Because Serializable is a tag, the pair
                                                  // has a value of type Position

    // Gets value from Position component
    e.get::<&Position>(|pos| {
        println!("Position: ({}, {})", pos.x, pos.y);
    });
    // Gets (unintended) value from (Serializable, Position) pair
    e.get::<&(Serializable, Position)>(|pos| {
        println!("Serializable Position: ({}, {})", pos.x, pos.y);
    });

    // This is currently not supported in Rust due to safety concerns.

    let e = world.entity().add_trait::<flecs::Final>();

    let i = world.entity().is_a_id(e); // not allowed

    // Register component with trait. Optional, since this is the default behavior.
    world
        .component::<Mass>()
        .add_trait::<(flecs::OnInstantiate, flecs::Override)>();

    let base = world.entity().set(Mass { value: 100.0 });
    let inst = world.entity().is_a_id(base); // Mass is copied to inst

    assert!(inst.owns::<Mass>());
    assert!(base.cloned::<&Mass>() != inst.cloned::<&Mass>());

    // Register component with trait
    world
        .component::<Mass>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().set(Mass { value: 100.0 });
    let inst = world.entity().is_a_id(base);

    assert!(inst.has::<Mass>());
    assert!(!inst.owns::<Mass>());
    assert!(base.cloned::<&Mass>() != inst.cloned::<&Mass>());

    // Register component with trait
    world
        .component::<Mass>()
        .add_trait::<(flecs::OnInstantiate, flecs::DontInherit)>();

    let base = world.entity().set(Mass { value: 100.0 });
    let inst = world.entity().is_a_id(base);

    assert!(!inst.has::<Mass>());
    assert!(!inst.owns::<Mass>());
    assert!(inst.try_get::<&Mass>(|mass| {}).is_none());

    let locatedin = world.entity();
    let manhattan = world.entity();
    let newyork = world.entity();
    let usa = world.entity();

    manhattan.add_id((locatedin, newyork));
    newyork.add_id((locatedin, usa));

    locatedin.add_trait::<flecs::Transitive>();

    let parent_a = world.entity();
    let parent_b = world.entity();
    e.child_of_id(parent_a);
    e.child_of_id(parent_b); // replaces (ChildOf, parent_a)

    let married_to = world.entity().add_trait::<flecs::Exclusive>();

    world
        .component::<Position>()
        .add_trait::<flecs::CanToggle>();

    let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    e.disable::<Position>(); // Disable component
    assert!(!e.is_enabled::<Position>());

    e.enable::<Position>(); // Enable component
    assert!(e.is_enabled::<Position>());

    let movement = world.entity().add_trait::<flecs::Union>();
    let walking = world.entity();
    let running = world.entity();

    let e = world.entity().add_id((movement, running));
    e.add_id((movement, walking)); // replaces (Movement, Running)

    world.component::<Position>().add_trait::<flecs::Sparse>();

    let married_to = world.entity().add_trait::<flecs::Symmetric>();
    let bob = world.entity();
    let alice = world.entity();
    bob.add_id((married_to, alice)); // Also adds (MarriedTo, Bob) to Alice

    let responsibility = world.entity();
    let power = world.entity().add_first::<flecs::With>(responsibility);

    // Create new entity that has both Power and Responsibility
    let e = world.entity().add_id(power);

    let likes = world.entity();
    let loves = world.entity().add_trait::<(flecs::With, Likes)>();
    let pears = world.entity();

    // Create new entity with both (Loves, Pears) and (Likes, Pears)
    let e = world.entity().add_id((loves, pears));

    // Enforce that target of relationship is child of Food
    let food = world.entity().add_trait::<flecs::OneOf>();
    let apples = world.entity().child_of_id(food);
    let fork = world.entity();

    // This is ok, Apples is a child of Food
    let a = world.entity().add_id((food, apples));

    // This is not ok, Fork is not a child of Food
    let b = world.entity().add_id((food, fork));

    // Enforce that target of relationship is child of Food
    let food = world.entity();
    let eats = world.entity().add_first::<flecs::OneOf>(food);
    let apples = world.entity().child_of_id(food);
    let fork = world.entity();

    // This is ok, Apples is a child of Food
    let a = world.entity().add_id((eats, apples));

    // This is not ok, Fork is not a child of Food
    let b = world.entity().add_id((eats, fork));
}

// app.world.set(flecs::rest::Rest::default());
// // enable stats for flecs (system, pipeline, etc)
// app.world.import::<stats::Stats>();
fn flecs_docs_remote_api_compile_test() {
    let world = World::new();

    // Optional, gather statistics for explorer
    world.import::<stats::Stats>();
    // Creates REST server on default port (27750)
    world.set(flecs::rest::Rest::default());
    // Runs the system serving up REST requests
    while world.progress() {}

    world
        .app()
        // Optional, gather statistics for explorer
        .enable_stats(true)
        .enable_rest(0)
        .run();

    // Optional, gather statistics for explorer
    world.import::<stats::Stats>();
    // Creates REST server on default port (27750)
    world.set(flecs::rest::Rest::default());
    // Runs the system serving up REST requests
    while world.progress() {}
}
