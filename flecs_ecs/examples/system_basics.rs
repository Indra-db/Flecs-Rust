mod common;
use common::*;

fn main() {
    let world = World::new();

    // Create a system for Position, Velocity. Systems are like queries (see
    // queries) with a function that can be ran or scheduled (see pipeline).

    let s = world
        .system_builder::<(&mut Position, &Velocity)>()
        .on_each_entity(|e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
            println!("{}: {{ {}, {} }}", e.name(), p.x, p.y);
        });

    // Create a few test entities for a Position, Velocity query
    world
        .new_entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .new_entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    // This entity will not match as it does not have Position, Velocity
    world
        .new_entity_named(c"e3")
        .set(Position { x: 10.0, y: 20.0 });

    // Run the system
    s.run();

    // Output:
    //  e1: { 11, 22 }
    //  e2: { 13, 24 }
}
