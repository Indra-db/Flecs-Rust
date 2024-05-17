use crate::z_snapshot_test::*;
snapshot_test!();
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

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Add the traits to mark the components to be inherited
    world.component::<Defence>().inheritable();
    world.component::<ImpulseSpeed>().inheritable();
    world.component::<FreightCapacity>().inheritable();
    world.component::<HasFlt>().inheritable();

    // Create a prefab hierarchy.
    let spaceship = world
        .prefab_named(c"Spaceship")
        // Add components to prefab entity as usual
        .set(ImpulseSpeed { value: 50.0 })
        .set(Defence { value: 50.0 })
        .set(Position { x: 0.0, y: 0.0 })
        // By default components in an inheritance hierarchy are shared between
        // entities. The override function ensures that instances have a private
        // copy of the component.
        .auto_override::<Position>();

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
        .entity_named(c"my_mammoth_freighter")
        .is_a_id(mammoth_freighter);

    // Inspect the type of the entity. This outputs:
    //    Position,(Identifier,Name),(IsA,MammothFreighter)
    fprintln!(&world, "Instance type: [{}]", inst.archetype());

    // Even though the instance doesn't have a private copy of ImpulseSpeed, we
    // can still get it using the regular API (outputs 50)
    inst.try_get::<&ImpulseSpeed>(|impulse_speed| {
        fprintln!(&world, "ImpulseSpeed: {}", impulse_speed.value);
    });

    // Prefab components can be iterated just like regular components:
    world.each_entity::<(&ImpulseSpeed, &mut Position)>(|entity, (impulse_speed, position)| {
        position.x += impulse_speed.value;
        fprintln!(entity, "Entity {}: {:?}", entity.name(), position);
    });

    world.get::<&Snap>(|snap| snap.test("entity_prefab".to_string()));

    // Output:
    //  Instance type: [Position, (Identifier,Name), (IsA,MammothFreighter)]
    //  ImpulseSpeed: 50
    //  Entity my_mammoth_freighter: Position { x: 50.0, y: 0.0 }
}
