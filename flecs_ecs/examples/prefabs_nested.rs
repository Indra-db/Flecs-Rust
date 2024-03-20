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

#[derive(Default, Debug, Clone, Component)]
struct TirePressure {
    value: f32,
}
fn main() {
    let world = World::new();

    // Create a Wheel prefab, make sure each instantiated wheel has a private
    // copy of the TirePressure component.
    let wheel = world.prefab_named(CStr::from_bytes_with_nul(b"Wheel\0").unwrap());
    wheel.set_override(TirePressure { value: 32.0 });

    // Create a Car prefab with four wheels. Note how we're using the scope
    // method, which has the same effect as adding the (ChildOf, Car) pair.
    let car = world.prefab_named(CStr::from_bytes_with_nul(b"Car\0").unwrap());
    car.run_in_scope(|| {
        world
            .prefab_named(CStr::from_bytes_with_nul(b"FrontLeft\0").unwrap())
            .is_a(&wheel);

        world
            .prefab_named(CStr::from_bytes_with_nul(b"FrontRight\0").unwrap())
            .is_a(&wheel);

        world
            .prefab_named(CStr::from_bytes_with_nul(b"BackLeft\0").unwrap())
            .is_a(&wheel);

        world
            .prefab_named(CStr::from_bytes_with_nul(b"BackRight\0").unwrap())
            .is_a(&wheel);
    });

    // Create a prefab instance.
    let inst_car = world.new_entity_named(CStr::from_bytes_with_nul(b"my_car\0").unwrap());
    inst_car.is_a(&car);

    // Lookup one of the wheels
    if let Some(inst) =
        inst_car.lookup_entity_by_name(CStr::from_bytes_with_nul(b"FrontLeft\0").unwrap(), true)
    {
        // The type shows that the child has a private copy of the TirePressure
        // component, and an IsA relationship to the Wheel prefab.
        println!("{:?}", inst.get_archetype());

        // Get the TirePressure component & print its value
        if let Some(p) = inst.get::<TirePressure>() {
            println!("pressure: {}", p.value);
        };
    } else {
        println!("entity lookup failed");
    }

    // Output:
    //  TirePressure, (Identifier,Name), (ChildOf,my_car), (IsA,Wheel)
    //  pressure: 32
}
