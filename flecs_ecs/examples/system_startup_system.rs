include!("common");

// Startup systems are systems registered with the EcsOnStart phase, and are
// only ran during the first frame. Just like with regular phases, custom phases
// can depend on the EcsOnStart phase (see custom_phases example). Phases that
// depend on EcsOnStart are also only ran during the first frame.
//
// Other than that, startup systems behave just like regular systems (they can
// match components, can introduce merge points), with as only exception that
// they are guaranteed to always run on the main thread.

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Startup system
    world
        .system_named::<()>(c"Startup")
        .kind::<flecs::pipeline::OnStart>()
        .iter_only(|it| {
            fprintln!(snap, "{}", it.system().name());
        });

    // Regular system
    world.system_named::<()>(c"Update").iter_only(|it| {
        fprintln!(snap, "{}", it.system().name());
    });

    // First frame. This runs both the Startup and Update systems
    world.progress();

    // Second frame. This runs only the Update system
    world.progress();

    Ok(snap)

    // Output:
    //  Startup
    //  Update
    //  Update
}
