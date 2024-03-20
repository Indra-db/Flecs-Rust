mod common;
use common::*;

// When a prefab has children, they are instantiated for an instance when the
// IsA relationship to the prefab is added.

fn main() {
    let world = World::new();

    // Create a prefab hierarchy.
    let spaceship = world.prefab_named(CStr::from_bytes_with_nul(b"SpaceShip\0").unwrap());
    world
        .prefab_named(CStr::from_bytes_with_nul(b"Engine\0").unwrap())
        .child_of(&spaceship);
    world
        .prefab_named(CStr::from_bytes_with_nul(b"Cockpit\0").unwrap())
        .child_of(&spaceship);

    // Instantiate the prefab. This also creates an Engine and Cockpit child
    // for the instance.
    let inst = world
        .new_entity_named(CStr::from_bytes_with_nul(b"my_spaceship\0").unwrap())
        .is_a(&spaceship);

    // Because of the IsA relationship, the instance now has the Engine and Cockpit
    // children of the prefab. This means that the instance can look up the Engine
    // and Cockpit entities.
    if let Some(inst_engine) =
        inst.lookup_entity_by_name(CStr::from_bytes_with_nul(b"Engine\0").unwrap(), true)
    {
        if let Some(inst_cockpit) =
            inst.lookup_entity_by_name(CStr::from_bytes_with_nul(b"Cockpit\0").unwrap(), true)
        {
            println!(
                "instance engine:  {:?}",
                inst_engine.get_hierarchy_path().unwrap()
            );
            println!(
                "instance cockpit: {:?}",
                inst_cockpit.get_hierarchy_path().unwrap()
            );
        } else {
            println!("entity lookup failed");
        }
    }
    // Output:
    //  instance engine:  "::my_spaceship::Engine"
    //  instance cockpit: "::my_spaceship::Cockpit"
}
