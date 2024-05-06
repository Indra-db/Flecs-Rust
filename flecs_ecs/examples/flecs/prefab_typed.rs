use crate::z_snapshot_test::*;
snapshot_test!();
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

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Associate types with prefabs
    world.prefab_type::<Turret>();

    world
        .prefab_type::<Base>()
        .child_of::<Turret>()
        .slot_of::<Turret>();

    world
        .prefab_type::<Head>()
        .child_of::<Turret>()
        .slot_of::<Turret>();

    world.prefab_type::<Railgun>().is_a::<Turret>();
    world
        .prefab_type::<Beam>()
        .slot_of::<Railgun>()
        .child_of::<Railgun>();

    // Create prefab instance.
    let inst = world.entity_named(c"my_railgun").is_a::<Railgun>();

    // Get entities for slots
    let inst_base = inst.target::<Base>(0);
    let inst_head = inst.target::<Head>(0);
    let inst_beam = inst.target::<Beam>(0);

    fprintln!(&world, "instance base: {}", inst_base.path().unwrap());
    fprintln!(&world, "instance head: {}", inst_head.path().unwrap());
    fprintln!(&world, "instance beam: {}", inst_beam.path().unwrap());

    world.get::<Snap>().test("prefab_typed".to_string());

    // Output:
    //  instance base: ::my_railgun::Base
    //  instance head: ::my_railgun::Head
    //  instance beam: ::my_railgun::Beam
}
