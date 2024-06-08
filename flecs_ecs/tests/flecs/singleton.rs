use flecs_ecs::core::internals::QueryConfig;
use flecs_ecs::core::{flecs, Builder, QueryBuilderImpl, ReactorAPI, TermBuilderImpl, World, ComponentId};
use flecs_ecs::macros::Component;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Component)]
struct Player {
    name: String,
}

#[derive(Component)]
struct Yeehaw {
    text: String,
}

#[derive(Component)]
struct SomeSingleton {
    pub some_value: i32,
}

#[test]
fn something() {
    let world = World::new();

    let mut has_player = Arc::new(AtomicBool::new(false));

    send_yeehaws(&world);
    get_yeehaws(&world, has_player.clone());

    world.entity_named("old town road").set(Player {
        name: "The Rizzler".to_string(),
    });

    let id = SomeSingleton::id(&world);
    println!("id of singleton is: {:?}", id);

    world.set(SomeSingleton { some_value: 42 });

    world.progress();

    let has_player = has_player.load(std::sync::atomic::Ordering::Relaxed);
    println!("has_player: {}", has_player);

    assert!(has_player, "has_player should be true");
}

fn send_yeehaws(world: &World) {
    world.system::<&Player>().each_entity(|entity, player| {
        println!("sending to '{}'", player.name);

        entity.enqueue(Yeehaw {
            text: "Hello World!".to_string(),
        });
    });
}

fn get_yeehaws(world: &World, has_player: Arc<AtomicBool>) {
    let mut observer = world.observer::<Yeehaw, (&SomeSingleton, &flecs::Any)>();

    let observer = observer.term_at(0).singleton();

    observer.each_entity(move |entity, _| {
        let got = entity.has::<Player>();
        println!("has_player: {}", got);
        has_player.store(got, std::sync::atomic::Ordering::Relaxed);
    });
}
