mod common;
use common::*;

fn main() {
    let world = World::new();

    // System that sets velocity using ecs_set for entities with Position.
    // While systems are progressing, operations like ecs_set are deferred until
    // it is safe to merge. By default this merge happens at the end of the
    // frame, but we can annotate systems to give the scheduler more information
    // about what it's doing, which allows it to insert sync points earlier.
    //
    // Note that sync points are never necessary/inserted for systems that write
    // components provided by their signature, as these writes directly happen
    // in the ECS storage and are never deferred.
    //
    // .inout_none() for Position tells the scheduler that while we
    // want to match entities with Position, we're not interested in reading or
    // writing the component value.

    world
        .system_named::<()>(c"SetVelocity")
        .with_type::<&Position>()
        .inout_none()
        .write_type::<&mut Velocity>() // Velocity is written, but shouldn't be matched
        .on_each_entity(|e, ()| {
            e.set(Velocity { x: 1.0, y: 2.0 });
        });

    // This system reads Velocity, which causes the insertion of a sync point.
    world
        .system_named::<(&mut Position, &Velocity)>(c"Move")
        .on_each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    // Print resulting Position
    world
        .system_named::<&Position>(c"PrintPosition")
        .on_each_entity(|e, p| {
            println!("{}: {{ {}, {} }}", e.name(), p.x, p.y);
        });

    // Create a few test entities for a Position, Velocity query
    world
        .new_entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .new_entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    // Run systems. Debug logging enables us to see the generated schedule.
    // NOTE flecs C / flecs_ecs_sys needs to be build in debug mode to see the logging.
    // use the feature flag "sys_build_debug" to enable debug build of flecs C.
    set_log_level(1);
    world.progress();
    set_log_level(-1);

    // Output:
    // info: pipeline rebuild
    // info: | schedule: threading: 0, staging: 1:
    // info: | | system SetVelocity
    // info: | | merge
    // info: | schedule: threading: 0, staging: 1:
    // info: | | system Move
    // info: | | system PrintPosition
    // info: | | merge
    // e1: { 11, 22 }
    // e2: { 11, 22 }

    // The "merge" lines indicate sync points.
    //
    // Removing '.write_type::<&mut Velocity>()' from the system will remove the first
    // sync point from the schedule.
}
