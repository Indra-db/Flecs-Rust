mod common;
use common::*;

// Custom pipelines make it possible for applications to override which systems
// are ran by a pipeline and how they are ordered. Pipelines are queries under
// the hood, and custom pipelines override the query used for system matching.

// If you only want to use custom phases in addition or in place of the builtin
// phases see the custom_phases and custom_phases_no_builtin examples, as this
// does not require using a custom pipeline.

#[derive(Debug, Component, Default)]
struct Physics;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create a pipeline that matches systems with Physics. Note that this
    // pipeline does not require the use of phases (see custom_phases) or of the
    // DependsOn relationship.
    let pipeline = world
        .pipeline()
        .with_id(flecs::system::System::ID)
        .with::<&Physics>()
        .build();

    // Configure the world to use the custom pipeline
    world.set_pipeline(pipeline.entity);

    // Create system with Physics tag
    world.system::<()>().kind::<Physics>().on_iter_only(|_| {
        fprintln!(snap, "System with Physics ran!");
    });

    // Create system without Physics tag
    world.system::<()>().on_iter_only(|_| {
        fprintln!(snap, "System without Physics ran!");
    });

    // Runs the pipeline & system
    world.progress();

    snap.test();

    // Output:
    //   System ran!
}
