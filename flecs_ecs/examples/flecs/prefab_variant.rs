use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Attack {
    pub value: f32,
}

#[derive(Component, Debug)]
pub struct Defence {
    pub value: f32,
}
#[derive(Component)]
pub struct FreightCapacity {
    pub value: f32,
}

#[derive(Component)]
pub struct ImpulseSpeed {
    pub value: f32,
}

// Prefabs can inherit from each other, which creates prefab variants. With
// variants applications can reuse a common set of components and specialize it
// by adding or overriding components on the variant.

fn main() {
    let world = World::new();

    // Create a base prefab for SpaceShips.
    let spaceship = world
        .prefab_named(c"SpaceShip")
        .set(ImpulseSpeed { value: 50.0 })
        .set(Defence { value: 25.0 });

    // Create a Freighter variant which inherits from SpaceShip
    let freighter = world
        .prefab_named(c"Freighter")
        .is_a_id(spaceship)
        .set(FreightCapacity { value: 100.0 })
        .set(Defence { value: 50.0 });

    // Create a MammotFreighter variant which inherits from Freighter
    let mammoth_freighter = world
        .prefab_named(c"MammothFreighter")
        .is_a_id(freighter)
        .set(FreightCapacity { value: 500.0 });

    // Create a Frigate variant which inherits from SpaceShip
    world
        .prefab_named(c"Frigate")
        .is_a_id(spaceship)
        .set(Attack { value: 100.0 })
        .set(Defence { value: 75.0 })
        .set(ImpulseSpeed { value: 125.0 });

    // Create an instance of the MammothFreighter. This entity will inherit the
    // ImpulseSpeed from SpaceShip, Defence from Freighter and FreightCapacity
    // from MammothFreighter.
    let inst = world
        .entity_named(c"my_freighter")
        .is_a_id(mammoth_freighter);

    // Add a private Position component.
    inst.set(Position { x: 10.0, y: 20.0 });

    // Instances can override inherited components to give them a private copy
    // of the component. This freighter got an armor upgrade:
    inst.set(Defence { value: 100.0 });

    // Queries can match components from multiple levels of inheritance
    world.each_entity::<(&Position, &ImpulseSpeed, &Defence, &FreightCapacity)>(
        |e, (p, s, d, c)| {
            println!("{}:", e.name());
            println!(" - position: {}, {}", p.x, p.y);
            println!(" - impulse speed: {}", s.value);
            println!(" - defense: {}", d.value);
            println!(" - capacity: {}", c.value);
        },
    );

    // Output:
    //   my_freighter:
    //    - position: 10, 20
    //    - impulse speed: 50
    //    - defense: 100
    //    - capacity: 500
}

#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("prefab_variant".to_string());
}
