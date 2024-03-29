#![allow(dead_code)]
mod common;
use common::*;

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
#[derive(Default, Clone, Component)]
struct Base;
#[derive(Default, Clone, Component)]
struct Head;

#[derive(Default, Clone, Component)]
struct Turret;

#[derive(Default, Clone, Component)]
struct Beam;
#[derive(Default, Clone, Component)]
struct Railgun;

fn main() {
    let world = World::new();

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
    let inst = world.new_entity_named(c"my_railgun").is_a::<Railgun>();

    // Get entities for slots
    let inst_base = inst.get_target::<Base>(0);
    let inst_head = inst.get_target::<Head>(0);
    let inst_beam = inst.get_target::<Beam>(0);

    println!("instance base: {}", inst_base.get_path().unwrap());
    println!("instance head: {}", inst_head.get_path().unwrap());
    println!("instance beam: {}", inst_beam.get_path().unwrap());

    // Output:
    //  instance base: ::my_railgun::Base
    //  instance head: ::my_railgun::Head
    //  instance beam: ::my_railgun::Beam
}
