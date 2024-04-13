mod common;
use common::*;

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
fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create a Wheel prefab, make sure each instantiated wheel has a private
    // copy of the TirePressure component.
    let wheel = world.prefab_named(c"Wheel");
    wheel.set_override(TirePressure { value: 32.0 });

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
    let inst_car = world.new_entity_named(c"my_car");
    inst_car.is_a_id(car);

    // Lookup one of the wheels
    if let Some(inst) = inst_car.lookup_name_optional(c"FrontLeft", true) {
        // The type shows that the child has a private copy of the TirePressure
        // component, and an IsA relationship to the Wheel prefab.
        fprintln!(snap, "{:?}", inst.archetype());

        // Get the TirePressure component & print its value
        if let Some(p) = inst.get::<TirePressure>() {
            fprintln!(snap, "pressure: {}", p.value);
        };
    } else {
        fprintln!(snap, "entity lookup failed");
    }

    snap.test();

    // Output:
    //  TirePressure, (Identifier,Name), (ChildOf,my_car), (IsA,Wheel)
    //  pressure: 32
}
