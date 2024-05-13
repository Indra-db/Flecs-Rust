use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
// This application demonstrates how to use custom phases for systems. The
// default pipeline will automatically run systems for custom phases as long as
// they have the flecs::Phase tag.

// Dummy system
fn sys(it: Iter) {
    fprintln!(it, "system {}", it.system().name());
}

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create two custom phases that branch off of EcsOnUpdate. Note that the
    // phases have the Phase tag, which is necessary for the builtin pipeline
    // to discover which systems it should run.
    let physics = world
        .entity()
        .add::<flecs::pipeline::Phase>()
        .depends_on::<flecs::pipeline::OnUpdate>();

    let collisions = world
        .entity()
        .add::<flecs::pipeline::Phase>()
        .depends_on_id(physics);

    // Create 3 dummy systems.
    world
        .system_named::<()>(c"CollisionSystem")
        .kind_id(collisions)
        .iter_only(sys);

    world
        .system_named::<()>(c"PhysicsSystem")
        .kind_id(physics)
        .iter_only(sys);

    world
        .system_named::<()>(c"GameSystem")
        .kind::<flecs::pipeline::OnUpdate>()
        .iter_only(sys);

    // Run pipeline
    world.progress();

    world.get::<&Snap>(|snap| snap.test("system_custom_phases".to_string()));

    // Output:
    //   system GameSystem
    //   system PhysicsSystem
    //   system CollisionSystem
}
