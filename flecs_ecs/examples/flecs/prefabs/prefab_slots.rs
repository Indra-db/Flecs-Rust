use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
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
    let spaceship = world.prefab_named("SpaceShip");
    let engine = world
        .prefab_named("Engine")
        .child_of(spaceship)
        .slot_of(spaceship);

    let cockpit = world
        .prefab_named("Cockpit")
        .child_of(spaceship)
        .slot_of(spaceship);

    // Add an additional child to the Cockpit prefab to demonstrate how
    // slots can be different from the parent. This slot could have been
    // added to the Cockpit prefab, but instead we register it on the top
    // level SpaceShip prefab.

    let pilot_seat = world
        .prefab_named("PilotSeat")
        .child_of(cockpit)
        .slot_of(spaceship);

    // Create a prefab instance.
    let inst = world.entity_named("my_spaceship").is_a(spaceship);

    // Get the instantiated entities for the prefab slots
    let inst_engine = inst.target(engine, 0).unwrap();
    let inst_cockpit = inst.target(cockpit, 0).unwrap();
    let inst_seat = inst.target(pilot_seat, 0).unwrap();

    println!("instance engine: {}", inst_engine.path().unwrap());

    println!("instance cockpit: {}", inst_cockpit.path().unwrap());

    println!("instance seat: {}", inst_seat.path().unwrap());

    // Output:
    //  instance engine: ::my_spaceship::Engine
    //  instance cockpit: ::my_spaceship::Cockpit
    //  instance seat: ::my_spaceship::Cockpit::PilotSeat
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("prefab_slots".to_string());
}
