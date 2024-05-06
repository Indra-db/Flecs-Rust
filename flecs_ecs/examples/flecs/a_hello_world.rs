use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Eats;

#[derive(Component)]
pub struct Apples;

#[test]
fn main() {
    // Create a new world
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Register system
    let _sys = world
        .system::<(&mut Position, &Velocity)>()
        // .each_entity if you want the entity to be added in the parameter list
        .each(|(pos, vel)| {
            pos.x += vel.x;
            pos.y += vel.y;
        });

    // Create an entity with name Bob, add Position and food preference
    let bob = world
        .entity_named(c"Bob")
        .set(Position { x: 0.0, y: 0.0 })
        .set(Velocity { x: 1.0, y: 2.0 })
        .add::<(Eats, Apples)>();

    // Show us what you got
    fprintln!(&world, "{}'s got [{:?}]", bob.name(), bob.archetype());

    // Run systems twice. Usually this function is called once per frame
    world.progress();
    world.progress();

    //you can use `.unwrap_unchecked()` if you are sure the component exists or `get_unchecked()`
    let pos = bob.try_get::<Position>().unwrap();
    // See if Bob has moved (he has)
    //fprintln!(snap,"Bob's position: {:?}", pos);
    fprintln!(&world, "{}'s position: {:?}", bob.name(), pos);

    world.get::<Snap>().test("hello world".to_string());

    // Output:
    //  Bob's got [Position, Velocity, (Identifier,Name), (Eats,Apples)]
    //  Bob's position: Position { x: 2.0, y: 4.0 }
}
