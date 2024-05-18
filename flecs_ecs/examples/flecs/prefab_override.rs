use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct Attack {
    pub value: f32,
}

#[derive(Component, Debug)]
pub struct Defence {
    pub value: f32,
}

#[derive(Component)]
pub struct Damage {
    pub value: f32,
}

fn main() {
    let world = World::new();

    // Add the traits to mark the components to be inherited
    world.component::<Defence>().inheritable();
    world.component::<Attack>().inheritable();

    // Attack and Damage are properties that can be shared across many
    // spaceships. This saves memory, and speeds up prefab creation as we don't
    // have to copy the values of Attack and Defense to private components.
    let spaceship = world
        .prefab_named(c"SpaceShip")
        .set(Attack { value: 75.0 })
        .set(Defence { value: 100.0 });

    // Damage is a property that is private to a spaceship, so add an auto
    // override for it. This ensures that each prefab instance will have a
    // private copy of the component.
    spaceship.set_auto_override(Damage { value: 0.0 });

    // Create a prefab instance.
    let inst = world.entity_named(c"my_spaceship").is_a_id(spaceship);

    // The entity will now have a private copy of the Damage component, but not
    // of the Attack and Defense components. We can see this when we look at the
    // type of the instance:
    println!("{}", inst.archetype());

    // Even though Attack was not automatically overridden, we can always
    // override it manually afterwards by adding it:
    inst.add::<Attack>();

    // The Attack component now shows up in the entity type:
    println!("{}", inst.archetype());

    // We can get all components on the instance, regardless of whether they
    // are overridden or not. Note that the overridden components (Attack and
    // Damage) are initialized with the values from the prefab component:
    inst.try_get::<(&Attack, &Defence, &Damage)>(|(attack, defence, damage)| {
        println!("attack: {}", attack.value);
        println!("defence: {}", defence.value);
        println!("damage: {}", damage.value);
    });

    // Output:
    //  Damage, (Identifier,Name), (IsA,SpaceShip)
    //  Attack, Damage, (Identifier,Name), (IsA,SpaceShip)
    //  attack: 75
    //  defence: 100
    //  damage: 0
}

#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("prefab_override".to_string());
}
