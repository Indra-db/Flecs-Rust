//! Compile tests for the flecs docs in the core C repo.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::print_stdout)]
use flecs_ecs::core::*;
use flecs_ecs::macros::*;

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

fn system_01() {
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

fn flecs_query_docs() {
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
}

/*
struct Eats { float value; };
struct Apples { };

flecs::entity eats = world.component<Eats>();
flecs::entity apples = world.component<Apples>();

flecs::query<> q1 = world.query_builder()
    .with<Eats, Apples>()
    .build();

flecs::query<> q2 = world.query_builder()
    .with<Eats>(apples)
    .build();

flecs::query<> q_3 = world.query_builder()
    .with(eats, apples)
    .build();
    */
