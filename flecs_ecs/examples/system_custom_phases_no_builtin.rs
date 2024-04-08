mod common;
use common::*;

// This application demonstrates how to use custom phases for systems. The
// default pipeline will automatically run systems for custom phases as long as
// they have the flecs::Phase tag.

// Dummy system
fn sys(it: Iter<'_>) {
    println!("system {}", it.system().name());
}

fn main() {
    let world = World::new();

    // Create three custom phases. Note that the phases have the Phase tag,
    // which is necessary for the builtin pipeline to discover which systems it
    // should run.

    let update = world.new_entity().add_id(ECS_PHASE);

    let physics = world.new_entity().add_id(ECS_PHASE).depends_on_id(update);

    let collisions = world.new_entity().add_id(ECS_PHASE).depends_on_id(physics);

    // Create 3 dummy systems.
    world
        .system_builder_named::<()>(c"CollisionSystem")
        .kind_id(collisions)
        .iter(sys);

    world
        .system_builder_named::<()>(c"PhysicsSystem")
        .kind_id(physics)
        .iter(sys);

    world
        .system_builder_named::<()>(c"GameSystem")
        .kind_id(update)
        .iter(sys);

    // Run pipeline
    world.progress();

    // Output
    //   system GameSystem
    //   system PhysicsSystem
    //   system CollisionSystem
}
