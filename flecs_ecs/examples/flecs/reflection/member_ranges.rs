use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Component)]
#[meta]
pub struct CpuUtilization {
    pub value: f64,
}

fn main() {
    let mut world = World::new();

    world
        .component::<CpuUtilization>()
        .meta()
        .range(0.0, 100.0) // Specifics values that the member can assume
        .warning_range(0.0, 60.0) // Values outside this range are considered a warning
        .error_range(0.0, 80.0); // Values outside this range are considered an error

    world
        .entity_named("MachineA")
        .set(CpuUtilization { value: 50.0 });
    world
        .entity_named("MachineB")
        .set(CpuUtilization { value: 75.0 });
    world
        .entity_named("MachineC")
        .set(CpuUtilization { value: 90.0 });

    // Uncomment this line and open
    //   https://www.flecs.dev/explorer?show=query&query=CpuUtilization
    // to see how ranges affect visualization:
    // world.app().enable_rest(0).run();
}
