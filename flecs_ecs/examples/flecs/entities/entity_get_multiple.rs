use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// This code shows how to get multiple components in a single command

#[derive(Debug, Component)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Component)]
pub struct Mass {
    pub value: f64,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
}

fn main() {
    let world = World::new();

    // Create new entity, set Position and Mass component
    let e = world
        .entity()
        .set(Position { x: 10.0, y: 20.0 })
        .set(Mass { value: 100.0 });

    // Multiple components can be fetched mutably in a single get by using a
    // tuple of mutable references.
    e.get::<(&mut Position, &mut Mass)>(|(pos, mass)| {
        pos.x += 5.0;
        mass.value += 3.0;
        println!("Position: {{{}, {}}}", pos.x, pos.y);
        println!("Mass: {{{}}}", mass.value);
    });

    println!();

    // The same works with immutable references, which do not allow the
    // components to be modified.
    e.get::<(&Position, &Mass)>(|(pos, mass)| {
        println!("Position: {{{}, {}}}", pos.x, pos.y);
        println!("Mass: {{{}}}", mass.value);
    });

    println!();

    // A component that may be absent can be wrapped in an Option, which is
    // None when the entity does not have the component.
    e.get::<(&Position, Option<&Velocity>, &Mass)>(|(pos, velocity, mass)| {
        println!("Position: {{{}, {}}}", pos.x, pos.y);
        if let Some(velocity) = velocity {
            println!("Velocity: {{{}, {}}}", velocity.x, velocity.y);
        } else {
            println!("Velocity: not found");
        }
        println!("Mass: {{{}}}", mass.value);
    });

    // Output:
    //  Position: {15, 20}
    //  Mass: {103}
    //
    //  Position: {15, 20}
    //  Mass: {103}
    //
    //  Position: {15, 20}
    //  Velocity: not found
    //  Mass: {103}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("entity_get_multiple".to_string());
}
