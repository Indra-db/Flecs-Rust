use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

// Systems can be created with a custom run function that takes control over the
// entire iteration. By  a system is invoked once per matched table,
// which means the function can be called multiple times per frame. In some
// cases that's inconvenient, like when a system has things it needs to do only
// once per frame. For these use cases, the run callback can be used which is
// called once per frame per system.

extern "C" fn run_callback(it: *mut IterT) {
    let world_ref = unsafe { WorldRef::from_ptr((*it).world) };
    println!("Move begin");

    // Walk over the iterator, forward to the system callback
    while unsafe { flecs_ecs_sys::ecs_iter_next(it) } {
        unsafe { ((*it).callback).unwrap()(it) };
    }

    println!("Move end");
}

fn main() {
    let world = World::new();

    let system = world
        .system::<(&mut Position, &Velocity)>()
        // The run function has a signature that accepts a C iterator. By
        // forwarding the iterator to it->callback, the each function of the
        // system is invoked.
        .set_run_callback(Some(run_callback)) // this will be rustified in the future to take a closure
        .each_entity(|e, (pos, vel)| {
            pos.x += vel.x;
            pos.y += vel.y;
            println!("{}: {{ {}, {} }}", e.name(), pos.x, pos.y);
        });

    // Create a few test entities for a Position, Velocity query
    world
        .entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    // This entity will not match as it does not have Position, Velocity
    world.entity_named(c"e3").set(Position { x: 10.0, y: 20.0 });

    // Run the system
    system.run();

    // Output:
    //  Move begin
    //  e1: {11, 22}
    //  e2: {13, 24}
    //  Move end
}

#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("system_custom_runner".to_string());
}
