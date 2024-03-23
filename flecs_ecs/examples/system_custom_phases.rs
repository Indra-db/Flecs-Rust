mod common;
use common::*;

// This application demonstrates how to use custom phases for systems. The
// default pipeline will automatically run systems for custom phases as long as
// they have the flecs::Phase tag.

// Dummy system
fn sys(it: &mut Iter) {
    println!("system {}", it.system().get_name());
}

fn main() {
    let world = World::new();

    // Create two custom phases that branch off of EcsOnUpdate. Note that the
    // phases have the Phase tag, which is necessary for the builtin pipeline
    // to discover which systems it should run.
    let physics = world
        .new_entity()
        .add_id(ECS_PHASE)
        .depends_on_id(ECS_ON_UPDATE);

    let collisions = world
        .new_entity()
        .add_id(ECS_PHASE)
        .depends_on_id(physics.into());

    // Create 3 dummy systems.
    world
        .system_builder_named::<()>(c"CollisionSystem")
        .kind_id(collisions.into())
        .on_iter_only(sys);

    world
        .system_builder_named::<()>(c"PhysicsSystem")
        .kind_id(physics.into())
        .on_iter_only(sys);

    world
        .system_builder_named::<()>(c"GameSystem")
        .kind_id(ECS_ON_UPDATE)
        .on_iter_only(sys);

    // Run pipeline
    world.progress();

    // Output
    //   system GameSystem
    //   system PhysicsSystem
    //   system CollisionSystem
}
