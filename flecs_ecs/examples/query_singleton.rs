mod common;
use common::*;

#[derive(Debug, Component)]
struct Gravity {
    value: f32,
}

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Set singleton
    world.set(Gravity { value: 9.81 });

    // Set Velocity
    world.entity_named(c"e1").set(Velocity { x: 0.0, y: 0.0 });
    world.entity_named(c"e2").set(Velocity { x: 0.0, y: 1.0 });
    world.entity_named(c"e3").set(Velocity { x: 0.0, y: 2.0 });

    // Create query that matches Gravity as singleton
    let query = world
        .query::<(&mut Velocity, &Gravity)>()
        .term_at(1)
        .singleton()
        .build();

    // In a query string expression you can use the $ shortcut for singletons:
    //   Velocity, Gravity($)

    query.each_entity(|entity, (velocity, gravity)| {
        velocity.y += gravity.value;
        fprintln!(snap, "Entity {} has {:?}", entity.path().unwrap(), velocity);
    });

    snap.test();

    // Output:
    // Entity ::e1 has Velocity { x: 0.0, y: 9.81 }
    // Entity ::e2 has Velocity { x: 0.0, y: 10.81 }
    // Entity ::e3 has Velocity { x: 0.0, y: 11.81 }
}
