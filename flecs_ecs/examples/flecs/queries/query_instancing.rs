use crate::z_ignore_test_common::*;

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

fn main() {
    let world = World::new();

    // Add the traits to mark the component to be inherited
    world
        .component::<Velocity>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

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
        .prefab_named("Prefab")
        .set(Velocity { x: 1.0, y: 2.0 });

    // Create a few entities that own Position & share Velocity from the prefab.
    world
        .entity_named("e1")
        .is_a_id(prefab)
        .set(Position { x: 10.0, y: 20.0 });

    world
        .entity_named("e2")
        .is_a_id(prefab)
        .set(Position { x: 10.0, y: 20.0 });

    // Create a few entities that own all components
    world
        .entity_named("e3")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    world
        .entity_named("e4")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 4.0, y: 5.0 });

    // Iterate the instanced query. Note how when a query is instanced, it needs
    // to check whether a field is owned or not in order to know how to access
    // it. In the case of an owned field it is iterated as an array, whereas
    // in the case of a shared field, it is accessed as a pointer.
    query.run_iter(|it, (position, velocity)| {
        // Check if Velocity is owned, in which case it's accessed as array.
        // Position will always be owned, since we set the term to Self.
        if it.is_self(1) {
            println!("Velocity is owned");

            for i in it.iter() {
                position[i].x += velocity[i].x;
                position[i].y += velocity[i].y;
                println!("entity {} has {:?}", it.entity(i).name(), position[i]);
            }
        } else {
            println!("Velocity is shared");

            for i in it.iter() {
                position[i].x += velocity[0].x;
                position[i].y += velocity[0].y;
                println!("entity {} has {:?}", it.entity(i).name(), position[i]);
            }
        }
    });

    // Output:
    //  Velocity is shared
    //  entity e1 has Position { x: 11.0, y: 22.0 }
    //  entity e2 has Position { x: 11.0, y: 22.0 }
    //  Velocity is owned
    //  entity e3 has Position { x: 13.0, y: 24.0 }
    //  entity e4 has Position { x: 14.0, y: 25.0 }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_instancing".to_string());
}
