use flecs_ecs::core::{IntoWorld, QueryBuilderImpl, ReactorAPI, TermBuilderImpl, World};
use flecs_ecs_derive::{system, Component};

#[derive(Debug, Component)]
struct Message {
    content: String,
}

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct Singleton {
    value: i32,
}

#[test]
fn emit_fails_with_singleton() {
    let world = World::new();

    for _ in 0..1_000 {
        world.entity().add::<Player>();
    }
    
    world.set(Singleton { value: 0 });

    world
        .observer::<Message, (&Singleton, &Player)>()
        .term_at(0)
        .singleton()
        .each_iter(|it, index, (_, msg)| {
            let elem = it.param();
            println!("{elem:?}");
        });

    world
        .system::<&Player>()
        .multi_threaded()
        .each_iter(|it, idx, msg| {
            let x = it.world();
            let e = it.entity(idx);

            x.event().target(e).add::<Player>().emit(&Message {
                content: String::from("Hello World"),
            });

            // e.emit(&Message {
            //     content: String::from("Hello World"),
            // });
        });

    world.set_threads(8);

    world.progress();
}

#[test]
fn emit_passes_without_singleton() {
    let world = World::new();

    for _ in 0..1_000 {
        world.entity().add::<Player>();
    }

    world.set(Singleton { value: 0 });

    world
        .observer::<Message, &Player>()
        .each_iter(|it, index, (msg)| {
            let elem = it.param();
            println!("{elem:?}");
        });

    world
        .system::<&Player>()
        .multi_threaded()
        .each_iter(|it, idx, msg| {
            let x = it.world();
            let e = it.entity(idx);

            x.event().target(e).add::<Player>().emit(&Message {
                content: String::from("Hello World"),
            });

            // e.emit(&Message {
            //     content: String::from("Hello World"),
            // });
        });

    world.set_threads(8);

    world.progress();
}
