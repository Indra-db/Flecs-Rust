use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
// Prefabs are entities that can be used as templates for other entities. They
// are created with a builtin Prefab tag, which by default excludes them from
// queries and systems.
//
// Prefab instances are entities that have an IsA relationship to the prefab.
// The IsA relationship causes instances to inherit the components from the
// prefab. By default all instances for a prefab share its components.
//
// Inherited components save memory as they only need to be stored once for all
// prefab instances. They also speed up the creation of prefabs, as inherited
// components don't need to be copied to the instances.
//
// To get a private copy of a component, an instance can add it which is called
// an override. Overrides can be manual (by using add) or automatic (see the
// auto_override example).
//
// If a prefab has children, adding the IsA relationship instantiates the prefab
// children for the instance (see hierarchy example).

#[derive(Component, Debug)]
pub struct Defence {
    pub value: f32,
}

fn main() {
    let world = World::new();

    // Add the traits to mark the component to be inherited
    world.component::<Defence>().inheritable();

    // Create a prefab with Position and Velocity components
    let spaceship = world.prefab_named("Prefab").set(Defence { value: 50.0 });

    // Create a prefab instance
    let inst = world.entity_named("my_spaceship").is_a_id(spaceship);

    // Because of the IsA relationship, the instance now shares the Defense
    // component with the prefab, and can be retrieved as a regular component:
    inst.try_get::<&Defence>(|d_inst| {
        println!("{:?}", d_inst);
        // Because the component is shared, changing the value on the prefab will
        // also change the value for the instance:
        // this is safe during a table lock because it also has the component and won't cause the table to move.
        spaceship.set(Defence { value: 100.0 });
        println!("after set: {:?}", d_inst);
    });

    // Prefab components can be iterated like regular components:
    world.each_entity::<&Defence>(|entity, d| {
        println!("{}: defence: {}", entity.path().unwrap(), d.value);
    });

    // Output:
    //  Defence { value: 50.0 }
    //  after set: Defence { value: 100.0 }
    //  ::my_spaceship: 100
}

#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("prefab_basics".to_string());
}
