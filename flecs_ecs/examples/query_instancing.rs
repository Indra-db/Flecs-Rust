mod common;
use common::*;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create a query for Position, Velocity. We'll create a few entities that
    // have Velocity as owned and shared component.
    let query = world
        .query::<(&mut Position, &Velocity)>()
        .term_at(0)
        .self_()
        .instanced()
        .build();

    // Create a prefab with Velocity. Prefabs are not matched with queries.
    let prefab = world
        .prefab_named(c"Prefab")
        .set(Velocity { x: 1.0, y: 2.0 });

    // Create a few entities that own Position & share Velocity from the prefab.
    world
        .entity_named(c"e1")
        .is_a_id(prefab)
        .set(Position { x: 10.0, y: 20.0 });

    world
        .entity_named(c"e2")
        .is_a_id(prefab)
        .set(Position { x: 10.0, y: 20.0 });

    // Create a few entities that own all components
    world
        .entity_named(c"e3")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    world
        .entity_named(c"e4")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 4.0, y: 5.0 });

    // Iterate the instanced query. Note how when a query is instanced, it needs
    // to check whether a field is owned or not in order to know how to access
    // it. In the case of an owned field it is iterated as an array, whereas
    // in the case of a shared field, it is accessed as a pointer.
    query.iter(|it, (position, velocity)| {
        // Check if Velocity is owned, in which case it's accessed as array.
        // Position will always be owned, since we set the term to Self.
        if it.is_self(1) {
            fprintln!(snap, "Velocity is owned");

            for i in it.iter() {
                position[i].x += velocity[i].x;
                position[i].y += velocity[i].y;
                fprintln!(snap, "entity {} has {:?}", it.entity(i).name(), position[i]);
            }
        } else {
            fprintln!(snap, "Velocity is shared");

            for i in it.iter() {
                position[i].x += velocity[0].x;
                position[i].y += velocity[0].y;
                fprintln!(snap, "entity {} has {:?}", it.entity(i).name(), position[i]);
            }
        }
    });

    snap.test();

    // Output:
    //  Velocity is shared
    //  entity e1 has Position { x: 11.0, y: 22.0 }
    //  entity e2 has Position { x: 11.0, y: 22.0 }
    //  Velocity is owned
    //  entity e3 has Position { x: 13.0, y: 24.0 }
    //  entity e4 has Position { x: 14.0, y: 25.0 }
}
