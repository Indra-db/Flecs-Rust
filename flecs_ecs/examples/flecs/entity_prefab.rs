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

#[derive(Component)]
pub struct HasFlt;

fn main() {
    let world = World::new();

    // Add the traits to mark the components to be inherited
    world.component::<Position>().inheritable();
    world.component::<Defence>().inheritable();
    world.component::<ImpulseSpeed>().inheritable();
    world.component::<FreightCapacity>().inheritable();
    world.component::<HasFlt>().inheritable();

    // Create a prefab hierarchy.
    let spaceship = world
        .prefab_named("Spaceship")
        // Add components to prefab entity as usual
        .set(ImpulseSpeed { value: 50.0 })
        .set(Defence { value: 50.0 })
        .set(Position { x: 0.0, y: 0.0 })
        // By default components in an inheritance hierarchy are shared between
        // entities. The override function ensures that instances have a private
        // copy of the component.
        .auto_override::<Position>();

    let freighter = world
        .prefab_named("Freighter")
        .is_a_id(spaceship)
        .set(FreightCapacity { value: 100.0 })
        .set(Defence { value: 100.0 })
        .add::<HasFlt>();

    let mammoth_freighter = world
        .prefab_named("MammothFreighter")
        .is_a_id(freighter)
        .set(FreightCapacity { value: 500.0 })
        .set(Defence { value: 300.0 });

    world
        .prefab_named("Frigate")
        .is_a_id(spaceship)
        .add::<HasFlt>()
        .set(Attack { value: 100.0 })
        .set(Defence { value: 75.0 })
        .set(ImpulseSpeed { value: 125.0 });

    // Create a regular entity from a prefab.
    // The instance will have a private copy of the Position component, because
    // of the override in the spaceship entity. All other components are shared.
    let inst = world
        .entity_named("my_mammoth_freighter")
        .is_a_id(mammoth_freighter);

    // Inspect the type of the entity. This outputs:
    //    Position,(Identifier,Name),(IsA,MammothFreighter)
    println!("Instance type: [{}]", inst.archetype());

    // Even though the instance doesn't have a private copy of ImpulseSpeed, we
    // can still get it using the regular API (outputs 50)
    inst.try_get::<&ImpulseSpeed>(|impulse_speed| {
        println!("ImpulseSpeed: {}", impulse_speed.value);
    });

    // Prefab components can be iterated just like regular components:
    world.each_entity::<(&ImpulseSpeed, &mut Position)>(|entity, (impulse_speed, position)| {
        position.x += impulse_speed.value;
        println!("Entity {}: {:?}", entity.name(), position);
    });

    // Output:
    //  Instance type: [Position, (Identifier,Name), (IsA,MammothFreighter)]
    //  ImpulseSpeed: 50
    //  Entity my_mammoth_freighter: Position { x: 50.0, y: 0.0 }
}

#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("entity_prefab".to_string());
}
