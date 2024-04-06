mod common;
use common::*;

// Startup systems are systems registered with the EcsOnStart phase, and are
// only ran during the first frame. Just like with regular phases, custom phases
// can depend on the EcsOnStart phase (see custom_phases example). Phases that
// depend on EcsOnStart are also only ran during the first frame.
//
// Other than that, startup systems behave just like regular systems (they can
// match components, can introduce merge points), with as only exception that
// they are guaranteed to always run on the main thread.

fn main() {
    let world = World::new();

    // Startup system
    world
        .system_builder_named::<()>(c"Startup")
        .kind_id(ECS_ON_START)
        .on_iter_only(|it| {
            println!("{}", it.system().name());
        });

    // Regular system
    world
        .system_builder_named::<()>(c"Update")
        .on_iter_only(|it| {
            println!("{}", it.system().name());
        });

    // First frame. This runs both the Startup and Update systems
    world.progress();

    // Second frame. This runs only the Update system
    world.progress();

    // Output:
    //  Startup
    //  Update
    //  Update
}
