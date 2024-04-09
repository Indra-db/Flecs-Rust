mod common;
use common::*;

// This example shows how to run a system at a specified time interval.

#[derive(Component)]
struct Timeout {
    pub value: f32,
}

fn tick(it: &mut Iter) {
    println!("{}", it.system().name());
}

fn main() {
    let world = World::new();

    world.set(Timeout { value: 3.5 });
    let time_out = world.get::<Timeout>().unwrap();

    world
        .system_builder::<&mut Timeout>()
        .on_each_iter(|it, _index, timeout| {
            timeout.value -= it.delta_time();
        });

    world
        .system_builder_named::<()>(c"Tick")
        .interval(1.0)
        .on_iter_only(tick);

    world
        .system_builder_named::<()>(c"FastTick")
        .interval(0.5)
        .on_iter_only(tick);

    // Run the main loop at 60 FPS
    world.set_target_fps(60.0);

    while world.progress() {
        if time_out.value <= 0.0 {
            println!("Timed out!");
            break;
        }
    }

    // Output:
    // FastTick
    // Tick
    // FastTick
    // FastTick
    // Tick
    // FastTick
    // FastTick
    // Tick
    // FastTick
    // FastTick
    // Timed out!
}
