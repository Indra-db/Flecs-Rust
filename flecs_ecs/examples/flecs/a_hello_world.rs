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

    // - get panics if the component is not present, use try_get for a non-panicking version which does not run the callback.
    // - or use Option to handle the individual component missing.
    bob.get::<&Position>(|pos| {
        // See if Bob has moved (he has)
        fprintln!(&world, "{}'s position: {:?}", bob.name(), pos);
    });

    // Option example
    let has_run = bob.try_get::<Option<&Position>>(|pos| {
        if let Some(pos) = pos {
            // See if Bob has moved (he has)
            fprintln!(&world, "{}'s try_get position: {:?}", bob.name(), pos);
        }
    });

    if has_run {
        fprintln!(
            &world,
            "Bob has a position component, so the try_get callback ran."
        );
    }

    world.get::<&Snap>(|snap| snap.test("hello world".to_string()));

    // Output:
    //  Bob's got [Position, Velocity, (Identifier,Name), (Eats,Apples)]
    //  Bob's position: Position { x: 2.0, y: 4.0 }
    //  Bob's try_get position: Position { x: 2.0, y: 4.0 }
    //  Bob has a position component, so the try_get callback ran.
}
