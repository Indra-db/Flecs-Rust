use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
// Startup systems are systems registered with the EcsOnStart phase, and are
// only ran during the first frame. Just like with regular phases, custom phases
// can depend on the EcsOnStart phase (see custom_phases example). Phases that
// depend on EcsOnStart are also only ran during the first frame.
//
// Other than that, startup systems behave just like regular systems (they can
// match components, can introduce merge points), with as only exception that
// they are guaranteed to always run on the main thread.

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Startup system
    world
        .system_named::<()>(c"Startup")
        .kind::<flecs::pipeline::OnStart>()
        .iter_only(|it| {
            fprintln!(it, "{}", it.system().name());
        });

    // Regular system
    world.system_named::<()>(c"Update").iter_only(|it| {
        fprintln!(it, "{}", it.system().name());
    });

    // First frame. This runs both the Startup and Update systems
    world.progress();

    // Second frame. This runs only the Update system
    world.progress();

    world.get::<&Snap>(|snap| snap.test("system_startup_system".to_string()));

    // Output:
    //  Startup
    //  Update
    //  Update
}
