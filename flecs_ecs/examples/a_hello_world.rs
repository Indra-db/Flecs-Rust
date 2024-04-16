mod common;
use common::*;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    // Create a new world
    let world = World::new();

    // Register system
    let _sys = world
        .system::<(&mut Position, &Velocity)>()
        // .on_each_entity if you want the entity to be added in the parameter list
        .on_each(|(pos, vel)| {
            pos.x += vel.x;
            pos.y += vel.y;
        });

    // Create an entity with name Bob, add Position and food preference
    let bob = world
        .new_entity_named(c"Bob")
        .set(Position { x: 0.0, y: 0.0 })
        .set(Velocity { x: 1.0, y: 2.0 })
        .add::<(Eats, Apples)>();

    // Show us what you got
    fprintln!(snap, "{}'s got [{:?}]", bob.name(), bob.archetype());

    // Run systems twice. Usually this function is called once per frame
    world.progress();
    world.progress();

    //you can use `.unwrap_unchecked()` if you are sure the component exists or `get_unchecked()`
    let pos = bob.try_get::<Position>().unwrap();
    // See if Bob has moved (he has)
    //fprintln!(snap,"Bob's position: {:?}", pos);
    fprintln!(snap, "{}'s position: {:?}", bob.name(), pos);

    snap.test();

    // Output:
    //  Bob's got [Position, Velocity, (Identifier,Name), (Eats,Apples)]
    //  Bob's position: Position { x: 2.0, y: 4.0 }
}
