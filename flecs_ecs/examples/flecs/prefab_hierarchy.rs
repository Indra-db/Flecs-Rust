use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
// When a prefab has children, they are instantiated for an instance when the
// IsA relationship to the prefab is added.

fn main() {
    let world = World::new();

    // Create a prefab hierarchy.
    let spaceship = world.prefab_named("SpaceShip");
    world.prefab_named("Engine").child_of_id(spaceship);
    world.prefab_named("Cockpit").child_of_id(spaceship);

    // Instantiate the prefab. This also creates an Engine and Cockpit child
    // for the instance.
    let inst = world.entity_named("my_spaceship").is_a_id(spaceship);

    // Because of the IsA relationship, the instance now has the Engine and Cockpit
    // children of the prefab. This means that the instance can look up the Engine
    // and Cockpit entities.
    if let Some(inst_engine) = inst.try_lookup_recursive("Engine") {
        if let Some(inst_cockpit) = inst.try_lookup_recursive("Cockpit") {
            println!("instance engine:  {:?}", inst_engine.path().unwrap());
            println!("instance cockpit: {:?}", inst_cockpit.path().unwrap());
        } else {
            println!("entity lookup failed");
        }
    }

    // Output:
    //  instance engine:  "::my_spaceship::Engine"
    //  instance cockpit: "::my_spaceship::Cockpit"
}

#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("prefab_hierarchy".to_string());
}
