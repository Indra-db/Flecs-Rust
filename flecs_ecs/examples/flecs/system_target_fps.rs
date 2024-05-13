use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create system that prints delta_time. This system doesn't query for any
    // components which means it won't match any entities, but will still be ran
    // once for each call to ecs_progress.
    world.system::<()>().iter_only(|it| {
        fprintln!(it.world(), "Delta time: {}", it.delta_time());
    });

    // Set target FPS to 1 frame per second
    world.set_target_fps(1.0);

    // Run 3 frames
    for _ in 0..3 {
        // To make sure the frame doesn't run faster than the specified target
        // FPS ecs_progress will insert a sleep if the measured delta_time is
        // smaller than 1 / target_fps.
        //
        // By default ecs_progress uses the sleep function provided by the OS
        // which is not always very accurate. If more accuracy is required the
        // sleep function of the OS API can be overridden with a custom one.
        //
        // If a value other than 0 is provided to the delta_time argument of
        // ecs_progress, this value will be used to calculate the length of
        // the sleep to insert.
        world.progress();
    }

    assert!(world.map::<&Snap, _>(|snap| snap.count() > 0));

    // Output:
    //  Delta time: 1
    //  Delta time: 1.0182016
    //  Delta time: 1.0170991
}
