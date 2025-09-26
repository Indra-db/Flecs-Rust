//! Tests from systems.md

#![allow(unused_imports, unused_variables, dead_code, non_snake_case, path_statements, unreachable_code, unused_mut,clippy::print_stdout)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::common_test::*;

#[test]
fn systems_01() {
    let world = World::new();
    // System declaration
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });
}

#[test]
fn systems_02() {
    let world = World::new();
    let sys = world
    .system_named::<(&mut Position, &Velocity)>("Move")
    .each(|(p, v)| {
    p.x += v.x;
    p.y += v.y;
    });
    /*
    let sys = ...;
    */
    sys.run();
}

#[test]
fn systems_03() {
    let world = World::new();
    let world = World::new();
    world.progress();
}

#[test]
fn systems_04() {
    let world = World::new();
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .kind(0)
        .each(|(p, v)| { /* ... */ });
}

#[test]
fn systems_system_iteration_05() {
    let world = World::new();
    // `.each_entity` if you need the associated entity.

    let q = world.query::<(&Position, &Velocity)>().build();

    // Query iteration (each)
    q.each(|(p, v)| { /* ... */ });

    // System iteration (each)
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .each(|(p, v)| { /* ... */ });
}

#[test]
fn systems_system_iteration_06() {
    let world = World::new();
    let q = world.query::<(&Position, &Velocity)>().build();
    // Query iteration (run)
    q.run(|mut it| {
        while it.next() {
            let mut p = it.field_mut::<Position>(0);
            let v = it.field::<Velocity>(1);

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
                let mut p = it.field_mut::<Position>(0);
                let v = it.field::<Velocity>(1);
                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
            }
        });
}

#[test]
fn systems_system_iteration_07() {
    let world = World::new();

    let q = world.query::<(&mut Position, &Velocity)>().build();
    // Query iteration (run_each_iter)
        q.run_each_iter(|mut it| {
            while it.next() {
                it.each();
            }
        }, |it,i,(p, v)| {
                p.x += v.x;
                p.y += v.y;
        });

    // System iteration (run_each_iter)
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .run_each_iter(|mut it| {
            while it.next() {
                it.each();
            }
        }, |it,i,(p, v)| {
                p.x += v.x;
                p.y += v.y;
        });
}

#[test]
fn systems_using_delta_time_08() {
    let world = World::new();
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .each_iter(|it, i, (p, v)| {
            p.x += v.x * it.delta_time();
            p.y += v.y * it.delta_time();
        });
}

#[test]
fn systems_using_delta_time_09() {
    let world = World::new();
    let delta_time = 0.016;
    world.progress_time(delta_time);
}

#[test]
fn systems_using_delta_time_10() {
    let world = World::new();
    world.progress();
}

#[test]
fn systems_tasks_11() {
    let world = World::new();
    world.system_named::<()>("PrintTime").run(|mut it| {
        while it.next() {
            println!("Time: {}", it.delta_time());
        }
    });

    // Runs PrintTime
    world.progress();
}

#[test]
fn systems_tasks_12() {
    let world = World::new();

    world.component::<Game>().add_trait::<flecs::Singleton>();

    world
        .system_named::<&Game>("PrintTime")
        .kind(flecs::pipeline::OnUpdate::id())
        .each(|game| {
            println!("Time: {}", game.time);
        });
}

#[test]
fn systems_pipelines_builtin_pipeline_13() {
    let world = World::new();
     // System is created with (DependsOn, OnUpdate)
     world
         .system_named::<(&mut Position, &Velocity)>("Move")
         .kind(flecs::pipeline::OnUpdate::id())
         .each(|(p, v)| {
             // ...
         });
}

#[test]
fn systems_pipelines_builtin_pipeline_builtin_pipeline_query_14() {
    let world = World::new();
    world
        .pipeline()
        .with(flecs::system::System::id())
        .with(flecs::pipeline::Phase::id())
        .cascade_id(flecs::DependsOn::id())
        .without(flecs::Disabled::id())
        .up_id(flecs::DependsOn::id())
        .without(flecs::Disabled::id())
        .up_id(flecs::ChildOf::id())
        .build();
}

#[test]
fn systems_pipelines_custom_pipeline_15() {
    let world = World::new();
    // Create custom pipeline
    let pipeline = world
        .pipeline()
        .with(flecs::system::System::id())
        .with(Foo::id()) // or `.with(foo) if an id`
        .build();

    // Configure the world to use the custom pipeline
    world.set_pipeline(pipeline);

    // Create system
    world
        .system_named::<(&mut Position, &Velocity)>("Move")
        .kind(Foo::id())
        .each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });
    // Runs the pipeline & system
    world.progress();
}

#[test]
fn systems_pipelines_custom_pipeline_16() {
    let world = World::new();
    let move_sys = world
    .system_named::<(&mut Position, &Velocity)>("Move").run(|mut it| {
    while it.next() {
    }
    });
    move_sys.add(Foo::id());
}

#[test]
fn systems_pipelines_disabling_systems_17() {
    let world = World::new();
    let s = world.system_named::<(&mut Position, &Velocity)>("Move").each(|(p, v)| {
    p.x += v.x;
    p.y += v.y;
    });
    // Disable system
    s.disable_self();
    // Enable system
    s.enable_self();
}

#[test]
fn systems_pipelines_disabling_systems_18() {
    let world = World::new();
    let sys = world
    .system_named::<(&mut Position, &Velocity)>("Move")
    .each(|(p, v)| {
    p.x += v.x;
    p.y += v.y;
    });
    sys.add(flecs::Disabled::id());
}

#[test]
fn systems_staging_sync_points_19() {
    let world = World::new();
    // In the Rust API, use the write method to indicate commands could be inserted.
    world.system::<&Position>().write(Transform::id()).each(|p| {
        // ...
    });
}

#[test]
fn systems_staging_sync_points_20() {
    let world = World::new();
    // In the Rust API, use the read method to indicate a component is read using .get
    world.system::<&Position>().read(Transform::id()).each(|p| {
        // ...
    });
}

#[test]
fn systems_staging_immediate_systems_21() {
    let world = World::new();
    world
        .system_named::<&Plate>("AssignPlate")
        .immediate(true) // disable readonly mode for this system
        .run(|mut it| {
            while it.next() {
                // ...
            }
        });
}

#[test]
fn systems_staging_immediate_systems_22() {
    let world = World::new();
    world
    .system_named::<&Plate>("AssignPlate")
    .immediate(true) // disable readonly mode for this system
    .run(|mut it| {
        while it.next() {
            // ECS operations ran here are visible after running the system
            it.world().defer_suspend();
            // ECS operations ran here are immediately visible
            it.world().defer_resume();
            // ECS operations ran here are visible after running the system
        }
    });
}

#[test]
fn systems_staging_threading_23() {
    let world = World::new();
    world.set_threads(4);
}

#[test]
fn systems_staging_threading_24() {
    let world = World::new();
    world.system::<&Position>().par_each(|p| {
        // ...
    });
}

#[test]
fn systems_staging_threading_with_async_tasks_25() {
    let world = World::new();
    world.set_task_threads(4);
}

#[test]
fn systems_timers_interval_26() {
    let world = World::new();
    world.system::<&Position>()
        .set_interval(1.0) // Run at 1Hz
        .each(|p| {
        // ...
    });
}

#[test]
fn systems_timers_rate_27() {
    let world = World::new();
    world
        .system::<&Position>()
        .set_rate(2) // Run every other frame
        .each(|p| {
            // ...
        });
}

#[test]
fn systems_timers_tick_source_28() {
    let world = World::new();

    let tick_source = world.timer().set_interval(1.0);

    world.system::<(&mut Position, &Velocity)>()
    .set_tick_source(tick_source)
    .each(|(p,v)| { /* ... */});
}

#[test]
fn systems_timers_tick_source_29() {
    let world = World::new();
    let tick_source = world.timer().set_interval(1.0);

    // Pause timer
    tick_source.stop();

    // Resume timer
    tick_source.start();
}

#[test]
fn systems_timers_nested_tick_sources_30() {
    let world = World::new();

    // tick at 1Hz
    let each_second = world.timer().set_interval(1.0);

    // tick each minute
    let each_minute = world.timer().set_rate_w_tick_source(60, each_second);

    // tick each hour
    let each_hour = world.timer().set_rate_w_tick_source(60, each_minute);
}

#[test]
fn systems_timers_nested_tick_sources_31() {
    let world = World::new();
    let each_second = world.system_named::<()>("EachSecond")
        .set_interval(1.0)
        .run(|mut it| {
            /* ... */
        });
    let each_minute = world.system_named::<()>("EachMinute")
        .set_tick_source(each_second)
        .set_rate(60)
        .run(|mut it| {
            /* ... */
        });
    let each_hour = world.system_named::<()>("EachHour")
        .set_tick_source(each_minute)
        .set_rate(60)
        .run(|mut it| {
            /* ... */
        });
}