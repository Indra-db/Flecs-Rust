use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct CpuUtilization {
    pub value: f64,
}

/*
int main() {
    flecs::world ecs;

    ecs.component<CpuUtilization>()
        .member<double>("value")
            .range(0.0, 100.0)        // Specifics values that the member can assume
            .warning_range(0.0, 60.0) // Values outside this range are considered a warning
            .error_range(0.0, 80.0);  // Values outside this range are considered an error

    ecs.entity("MachineA").set<CpuUtilization>({ 50.0 });
    ecs.entity("MachineB").set<CpuUtilization>({ 75.0 });
    ecs.entity("MachineC").set<CpuUtilization>({ 90.0 });

    // Uncomment this line and open
    //   https://www.flecs.dev/explorer?show=query&query=CpuUtilization
    // to see how ranges affect visualization:
    // ecs.app().enable_rest().run();
}

*/

#[test]
fn main() {
    let mut world = World::new();

    world
        .component::<CpuUtilization>()
        .member::<f64>("value", 1, offset_of!(CpuUtilization, value))
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
