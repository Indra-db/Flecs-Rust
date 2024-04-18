mod common;
use common::*;

// When a prefab has children, they are instantiated for an instance when the
// IsA relationship to the prefab is added.

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create a prefab hierarchy.
    let spaceship = world.prefab_named(c"SpaceShip");
    world.prefab_named(c"Engine").child_of_id(spaceship);
    world.prefab_named(c"Cockpit").child_of_id(spaceship);

    // Instantiate the prefab. This also creates an Engine and Cockpit child
    // for the instance.
    let inst = world.new_entity_named(c"my_spaceship").is_a_id(spaceship);

    // Because of the IsA relationship, the instance now has the Engine and Cockpit
    // children of the prefab. This means that the instance can look up the Engine
    // and Cockpit entities.
    if let Some(inst_engine) = inst.try_lookup_name(c"Engine", true) {
        if let Some(inst_cockpit) = inst.try_lookup_name(c"Cockpit", true) {
            fprintln!(snap, "instance engine:  {:?}", inst_engine.path().unwrap());
            fprintln!(snap, "instance cockpit: {:?}", inst_cockpit.path().unwrap());
        } else {
            fprintln!(snap, "entity lookup failed");
        }
    }

    snap.test();

    // Output:
    //  instance engine:  "::my_spaceship::Engine"
    //  instance cockpit: "::my_spaceship::Cockpit"
}
