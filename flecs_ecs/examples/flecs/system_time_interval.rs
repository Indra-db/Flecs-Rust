use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
// This example shows how to run a system at a specified time interval.

#[derive(Component)]
struct Timeout {
    pub value: f32,
}

fn tick(it: Iter) {
    println!("{}", it.system().name());
}

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    world.set(Timeout { value: 3.5 });

    world
        .system::<&mut Timeout>()
        .each_iter(|it, _index, timeout| {
            timeout.value -= it.delta_time();
        });

    world
        .system_named::<()>(c"Tick")
        .interval(1.0)
        .iter_only(tick);

    world
        .system_named::<()>(c"FastTick")
        .interval(0.5)
        .iter_only(tick);

    // Run the main loop at 60 FPS
    world.set_target_fps(60.0);

    while world.progress() {
        if world.map::<&Timeout, _>(|timeout| timeout.value <= 0.0) {
            fprintln!(&world, "Timed out!");
            break;
        }
    }

    assert!(world.map::<&Snap, _>(|snap| snap.str.last().unwrap().contains("Timed out!")));

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
