use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
// Nested prefabs make it possible to reuse an existing prefab inside another
// prefab. An example of where this could be useful is a car with four wheels:
// instead of defining four times what a wheel is a Car prefab can reference an
// existing Wheel prefab.
//
// Nested prefabs can be created by adding a child that is a variant (inherits
// from) another prefab. For more information on variants, see the variants
// example.
//
// Instantiated children from a nested prefab still inherit from the original
// prefab. The reason for this is that an instantiated child is an exact copy
// of the prefab child, and the prefab child only has an IsA relationship to the
// nested prefab.
//
// This example shows how auto overriding (see the auto override example) can be
// used to give instantiated children from a nested prefab a private copy of an
// inherited component.

#[derive(Debug, Component)]
struct TirePressure {
    value: f32,
}

#[test]
#[ignore = "is a hierarchy traversal not supported with new get callback"]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create a Wheel prefab, make sure each instantiated wheel has a private
    // copy of the TirePressure component.
    let wheel = world.prefab_named(c"Wheel");
    wheel.set_auto_override(TirePressure { value: 32.0 });

    // Create a Car prefab with four wheels. Note how we're using the scope
    // method, which has the same effect as adding the (ChildOf, Car) pair.
    let car = world.prefab_named(c"Car");
    car.run_in_scope(|| {
        world.prefab_named(c"FrontLeft").is_a_id(wheel);

        world.prefab_named(c"FrontRight").is_a_id(wheel);

        world.prefab_named(c"BackLeft").is_a_id(wheel);

        world.prefab_named(c"BackRight").is_a_id(wheel);
    });

    // Create a prefab instance.
    let inst_car = world.entity_named(c"my_car");
    inst_car.is_a_id(car);

    // Lookup one of the wheels
    if let Some(inst) = inst_car.try_lookup_recursive(c"FrontLeft") {
        // The type shows that the child has a private copy of the TirePressure
        // component, and an IsA relationship to the Wheel prefab.
        fprintln!(&world, "{:?}", inst.archetype());

        // Get the TirePressure component & print its value
        inst.try_get::<Option<&TirePressure>>(|p| {
            if let Some(p) = p {
                fprintln!(&world, "pressure: {}", p.value);
            }
        });
    } else {
        fprintln!(&world, "entity lookup failed");
    }

    world.get::<&Snap>(|snap| snap.test("prefab_nested".to_string()));

    // Output:
    //  TirePressure, (Identifier,Name), (ChildOf,my_car), (IsA,Wheel)
    //  pressure: 32
}
