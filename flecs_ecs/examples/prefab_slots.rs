mod common;
use common::*;

// Slots can be combined with prefab hierarchies to make it easier to access
// the child entities created for an instance.
//
// To create a slot, the SlotOf relationship is added to the child of a prefab,
// with as relationship target the prefab for which to register the slot. When
// the prefab is instantiated, each slot will be added as a relationship pair
// to the instance that looks like this:
//   (PrefabChild, InstanceChild)
//
// For a SpaceShip prefab and an Engine child, that pair would look like this:
//   (SpaceShip.Engine, Instance.Engine)
//
// To get the entity for a slot, an application can use the regular functions
// to inspect relationships and relationship targets (see code).
//
// Slots can be added to any level of a prefab hierarchy, as long as it is above
// (a parent of) the slot itself. When the prefab tree is instantiated, the
// slots are added to the entities that correspond with the prefab children.
//
// Without slots, an application would have to rely on manually looking up
// entities by name to get access to the instantiated children, like what the
// hierarchy example does.

fn main() {
    let world = World::new();

    // Create the same prefab hierarchy as from the hierarchy example, but now
    // with the SlotOf relationship.
    let spaceship = world.prefab_named(c"SpaceShip");
    let engine = world
        .prefab_named(c"Engine")
        .child_of_id(spaceship)
        .slot_of_id(spaceship);

    let cockpit = world
        .prefab_named(c"Cockpit")
        .child_of_id(spaceship)
        .slot_of_id(spaceship);

    // Add an additional child to the Cockpit prefab to demonstrate how
    // slots can be different from the parent. This slot could have been
    // added to the Cockpit prefab, but instead we register it on the top
    // level SpaceShip prefab.

    let pilot_seat = world
        .prefab_named(c"PilotSeat")
        .child_of_id(cockpit)
        .slot_of_id(spaceship);

    // Create a prefab instance.
    let inst: Entity = world.new_entity_named(c"my_spaceship").is_a_id(spaceship);

    // Get the instantiated entities for the prefab slots
    let inst_engine = inst.get_target_from_entity(engine, 0);
    let inst_cockpit = inst.get_target_from_entity(cockpit, 0);
    let inst_seat = inst.get_target_from_entity(pilot_seat, 0);

    println!(
        "instance engine: {}",
        inst_engine.get_hierarchy_path().unwrap()
    );

    println!(
        "instance cockpit: {}",
        inst_cockpit.get_hierarchy_path().unwrap()
    );

    println!("instance seat: {}", inst_seat.get_hierarchy_path().unwrap());

    // Output:
    //  instance engine: ::my_spaceship::Engine
    //  instance cockpit: ::my_spaceship::Cockpit
    //  instance seat: ::my_spaceship::Cockpit::PilotSeat
}
