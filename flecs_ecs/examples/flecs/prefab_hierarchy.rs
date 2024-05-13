use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
// When a prefab has children, they are instantiated for an instance when the
// IsA relationship to the prefab is added.

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create a prefab hierarchy.
    let spaceship = world.prefab_named(c"SpaceShip");
    world.prefab_named(c"Engine").child_of_id(spaceship);
    world.prefab_named(c"Cockpit").child_of_id(spaceship);

    // Instantiate the prefab. This also creates an Engine and Cockpit child
    // for the instance.
    let inst = world.entity_named(c"my_spaceship").is_a_id(spaceship);

    // Because of the IsA relationship, the instance now has the Engine and Cockpit
    // children of the prefab. This means that the instance can look up the Engine
    // and Cockpit entities.
    if let Some(inst_engine) = inst.try_lookup_recursive(c"Engine") {
        if let Some(inst_cockpit) = inst.try_lookup_recursive(c"Cockpit") {
            fprintln!(
                &world,
                "instance engine:  {:?}",
                inst_engine.path().unwrap()
            );
            fprintln!(
                &world,
                "instance cockpit: {:?}",
                inst_cockpit.path().unwrap()
            );
        } else {
            fprintln!(&world, "entity lookup failed");
        }
    }

    world.get::<&Snap>(|snap| 
        snap.test("prefab_hierarchy".to_string()));

    // Output:
    //  instance engine:  "::my_spaceship::Engine"
    //  instance cockpit: "::my_spaceship::Cockpit"
}
