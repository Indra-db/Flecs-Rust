//! This code shows how to get & set multiple components in a single command

mod common;
use common::*;

fn main() {
    let world = World::new();

    // Create a prefab hierarchy.
    let spaceship = world
        .prefab_named(c"Spaceship")
        // Add components to prefab entity as usual
        .set(ImpulseSpeed { value: 50.0 })
        .set(Defence { value: 50.0 })
        // By default components in an inheritance hierarchy are shared between
        // entities. The override function ensures that instances have a private
        // copy of the component.
        .override_type::<Position>();

    let freighter = world
        .prefab_named(c"Freighter")
        .is_a_id(spaceship)
        .set(FreightCapacity { value: 100.0 })
        .set(Defence { value: 100.0 })
        .add::<HasFlt>();

    let mammoth_freighter = world
        .prefab_named(c"MammothFreighter")
        .is_a_id(freighter)
        .set(FreightCapacity { value: 500.0 })
        .set(Defence { value: 300.0 });

    world
        .prefab_named(c"Frigate")
        .is_a_id(spaceship)
        .add::<HasFlt>()
        .set(Attack { value: 100.0 })
        .set(Defence { value: 75.0 })
        .set(ImpulseSpeed { value: 125.0 });

    // Create a regular entity from a prefab.
    // The instance will have a private copy of the Position component, because
    // of the override in the spaceship entity. All other components are shared.
    let inst = world
        .new_entity_named(c"my_mammoth_freighter")
        .is_a_id(mammoth_freighter);

    // Inspect the type of the entity. This outputs:
    //    Position,(Identifier,Name),(IsA,MammothFreighter)
    println!("Instance type: [{}]", inst.archetype());

    // Even though the instance doesn't have a private copy of ImpulseSpeed, we
    // can still get it using the regular API (outputs 50)
    let impulse_speed = inst.get::<ImpulseSpeed>();
    println!("ImpulseSpeed: {}", impulse_speed.unwrap().value);

    // Prefab components can be iterated just like regular components:
    world.each(
        |entity: Entity, impulse_speed: &ImpulseSpeed, position: &mut Position| {
            position.x += impulse_speed.value;
            println!("Entity {}: {:?}", entity.name(), position);
        },
    );

    // Output:
    //  Instance type: [Position, (Identifier,Name), (IsA,MammothFreighter)]
    //  ImpulseSpeed: 50
    //  Entity my_mammoth_freighter: Position { x: 50.0, y: 0.0 }
}
