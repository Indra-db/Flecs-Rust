mod common;
use common::*;

// This example shows how to run a system at a specified time interval.

fn tick(it: Iter) {
    println!("{}", it.system().name());
}

fn main() {
    let world = World::new();

    world
        .system_builder_named::<()>(c"Tick")
        .interval(1.0)
        .iter(tick);

    world
        .system_builder_named::<()>(c"FastTick")
        .interval(0.5)
        .iter(tick);

    // Run the main loop at 60 FPS
    world.set_target_fps(60.0);

    while world.progress() {}

    // Output:
    //  FastTick
    //  Tick
    //  FastTick
    //  FastTick
    //  Tick
    //  FastTick
    //  FastTick
    //  Tick
    //  ... (infinite while loop)
}
