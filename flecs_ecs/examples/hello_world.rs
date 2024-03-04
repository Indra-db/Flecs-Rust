use std::ffi::CStr;

use flecs_ecs::core::*;
use flecs_ecs_derive::Component;

#[derive(Debug, Default, Clone, Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Default, Clone, Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Default, Clone, Component)]
struct Eats {}
#[derive(Default, Clone, Component)]
struct Apples {}

fn main() {
    // Create a new world
    let world = World::new();

    // Register system
    let _sys = world
        .system_builder::<(Position, Velocity)>()
        .on_each_entity(|_entity, (pos, vel)| {
            pos.x += vel.x;
            pos.y += vel.y;
        });

    // Create an entity with name Bob, add Position and food preference
    let bob = world
        .new_entity_named(CStr::from_bytes_with_nul(b"Bob\0").unwrap())
        .set_component(Position { x: 0.0, y: 0.0 })
        .set_component(Velocity { x: 1.0, y: 2.0 })
        .add_pair::<Eats, Apples>();

    // Show us what you got
    println!(
        "{}'s got [{}]",
        bob.get_name(),
        bob.get_archetype().to_string().unwrap()
    );

    // Run systems twice. Usually this function is called once per frame
    world.progress();
    world.progress();

    //you can use `.unwrap_unchecked()` if you are sure the component exists or `get_unchecked()`
    let pos = bob.get::<Position>().unwrap();
    // See if Bob has moved (he has)
    println!("Bob's position: {:?}", pos);

    // Output
    //  Bob's got [Position, Velocity, (Identifier,Name), (Eats,Apples)]
    //  Bob's position: Position { x: 2.0, y: 4.0 }

    //do the same for bevy
}
