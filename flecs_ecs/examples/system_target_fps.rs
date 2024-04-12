mod common;
use common::*;

fn main() {
    let world = World::new();

    // Create system that prints delta_time. This system doesn't query for any
    // components which means it won't match any entities, but will still be ran
    // once for each call to ecs_progress.
    world.system::<()>().on_iter_only(|it| {
        println!("Delta time: {}", it.delta_time());
    });

    // Set target FPS to 1 frame per second
    world.set_target_fps(1.0);

    // Run 5 frames
    for _ in 0..5 {
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

    // Output:
    //  Delta time: 1
    //  Delta time: 1.0182016
    //  Delta time: 1.0170991
    //  Delta time: 1.0179571
    //  Delta time: 1.0196676
}
