use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
// Type for Platoon relationship
#[derive(Component)]
struct Platoon;

fn main() {
    let world = World::new();

    // Register Platoon as exclusive relationship. This ensures that an entity
    // can only belong to a single Platoon.
    world.component::<Platoon>().add(flecs::Exclusive);

    // Create two platoons
    let platoon_1 = world.entity();
    let platoon_2 = world.entity();

    // Create a unit
    let unit = world.entity();

    // Add unit to platoon 1
    unit.add((Platoon, platoon_1));

    // Log platoon of unit
    println!("Unit in platoon 1: {}", unit.has((Platoon, platoon_1))); // true
    println!("Unit in platoon 2: {}", unit.has((Platoon, platoon_2))); // false

    println!();

    // Add unit to platoon 2. Because Platoon is an exclusive relationship, this
    // both removes (Platoon, platoon_1) and adds (Platoon, platoon_2) in a
    // single operation.
    unit.add((Platoon, platoon_2));

    // Log platoon of unit
    println!("Unit in platoon 1: {}", unit.has((Platoon, platoon_1))); // false
    println!("Unit in platoon 2: {}", unit.has((Platoon, platoon_2))); // true

    // Output:
    //  Unit in platoon 1: true
    //  Unit in platoon 2: false
    //
    //  Unit in platoon 1: false
    //  Unit in platoon 2: true
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("relationships_exclusive".to_string());
}
