use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
// Just like how entities can be associated with a type (like components)
// prefabs can be associated with types as well. Types can be more convenient to
// work with than entity handles, for a few reasons:
//
// - There is no need to pass around or store entity handles
// - Prefabs automatically assume the name of the type
// - Nested types can be used to create prefab hierarchies
//
// While this functionality is not unique to prefabs (the same mechanism is
// used to distribute component handles), prefabs are a good fit, especially
// when combined with prefab slots (see slots example and code below).

// Create types that mirror the prefab hierarchy.
#[derive(Component)]
struct Base;
#[derive(Component)]
struct Head;

#[derive(Component)]
struct Turret;

#[derive(Component)]
struct Beam;
#[derive(Component)]
struct Railgun;

fn main() {
    let world = World::new();

    // Associate types with prefabs
    world.prefab_type::<Turret>();

    world
        .prefab_type::<Base>()
        .child_of(id::<Turret>())
        .slot_of(id::<Turret>());

    world
        .prefab_type::<Head>()
        .child_of(id::<Turret>())
        .slot_of(id::<Turret>());

    world.prefab_type::<Railgun>().is_a(id::<Turret>());
    world
        .prefab_type::<Beam>()
        .slot_of(id::<Railgun>())
        .child_of(id::<Railgun>());

    // Create prefab instance.
    let inst = world.entity_named("my_railgun").is_a(id::<Railgun>());

    // Get entities for slots
    let inst_base = inst.target(id::<Base>(), 0).unwrap();
    let inst_head = inst.target(id::<Head>(), 0).unwrap();
    let inst_beam = inst.target(id::<Beam>(), 0).unwrap();

    println!("instance base: {}", inst_base.path().unwrap());
    println!("instance head: {}", inst_head.path().unwrap());
    println!("instance beam: {}", inst_beam.path().unwrap());

    // Output:
    //  instance base: ::my_railgun::Base
    //  instance head: ::my_railgun::Head
    //  instance beam: ::my_railgun::Beam
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("prefab_typed".to_string());
}
