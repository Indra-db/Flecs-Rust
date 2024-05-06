use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
#[derive(Component)]
pub struct PositionSP {
    x: f32,
    y: f32,
}

#[derive(Component)]
pub struct VelocitySP {
    x: f32,
    y: f32,
}

#[test]
#[ignore = "`set_log_level` is not safe in parallel tests"]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // System that sets velocity using ecs_set for entities with PositionSP.
    // While systems are progressing, operations like ecs_set are deferred until
    // it is safe to merge. By default this merge happens at the end of the
    // frame, but we can annotate systems to give the scheduler more information
    // about what it's doing, which allows it to insert sync points earlier.
    //
    // Note that sync points are never necessary/inserted for systems that write
    // components provided by their signature, as these writes directly happen
    // in the ECS storage and are never deferred.
    //
    // .inout_none() for PositionSP tells the scheduler that while we
    // want to match entities with PositionSP, we're not interested in reading or
    // writing the component value.

    world
        .system_named::<()>(c"SetVelocitySP")
        .with::<&PositionSP>()
        .set_inout_none()
        .write::<&mut VelocitySP>() // VelocitySP is written, but shouldn't be matched
        .each_entity(|e, ()| {
            e.set(VelocitySP { x: 1.0, y: 2.0 });
        });

    // This system reads VelocitySP, which causes the insertion of a sync point.
    world
        .system_named::<(&mut PositionSP, &VelocitySP)>(c"Move")
        .each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    // Print resulting PositionSP
    world
        .system_named::<&PositionSP>(c"PrintPositionSP")
        .each_entity(|e, p| {
            fprintln!(e, "{}: {{ {}, {} }}", e.name(), p.x, p.y);
        });

    // Create a few test entities for a PositionSP, VelocitySP query
    world
        .entity_named(c"e1")
        .set(PositionSP { x: 10.0, y: 20.0 })
        .set(VelocitySP { x: 1.0, y: 2.0 });

    world
        .entity_named(c"e2")
        .set(PositionSP { x: 10.0, y: 20.0 })
        .set(VelocitySP { x: 3.0, y: 4.0 });

    // Run systems. Debug logging enables us to see the generated schedule.
    // NOTE flecs C / flecs_ecs_sys needs to be build in debug mode to see the logging.
    // use the feature flag "sys_build_debug" to enable debug build of flecs C.
    set_log_level(1);
    world.progress();
    set_log_level(-1);

    world.get::<Snap>().test("system_sync_point".to_string());

    // Output:
    // info: pipeline rebuild
    // info: | schedule: threading: 0, staging: 1:
    // info: | | system SetVelocitySP
    // info: | | merge
    // info: | schedule: threading: 0, staging: 1:
    // info: | | system Move
    // info: | | system PrintPositionSP
    // info: | | merge
    // e1: { 11, 22 }
    // e2: { 11, 22 }

    // The "merge" lines indicate sync points.
    //
    // Removing '.write::<&mut VelocitySP>()' from the system will remove the first
    // sync point from the schedule.
}
