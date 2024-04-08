mod common;
use common::*;

// This application demonstrates how to use custom phases for systems. The
// default pipeline will automatically run systems for custom phases as long as
// they have the flecs::Phase tag.

// Dummy system
fn sys(it: &mut Iter) {
    println!("system {}", it.system().name());
}

fn main() {
    let world = World::new();

    // Create three custom phases. Note that the phases have the Phase tag,
    // which is necessary for the builtin pipeline to discover which systems it
    // should run.

    let update = world.new_entity().add::<flecs::pipeline::Phase>();

    let physics = world
        .new_entity()
        .add::<flecs::pipeline::Phase>()
        .depends_on_id(update);

    let collisions = world
        .new_entity()
        .add::<flecs::pipeline::Phase>()
        .depends_on_id(physics);

    // Create 3 dummy systems.
    world
        .system_builder_named::<()>(c"CollisionSystem")
        .kind_id(collisions)
        .on_iter_only(sys);

    world
        .system_builder_named::<()>(c"PhysicsSystem")
        .kind_id(physics)
        .on_iter_only(sys);

    world
        .system_builder_named::<()>(c"GameSystem")
        .kind_id(update)
        .on_iter_only(sys);

    // Run pipeline
    world.progress();

    // Output
    //   system GameSystem
    //   system PhysicsSystem
    //   system CollisionSystem
}
