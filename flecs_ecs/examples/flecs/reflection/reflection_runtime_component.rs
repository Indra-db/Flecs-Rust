use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

fn main() {
    let world = World::new();

    let position = world
        .component_untyped_named("Position")
        .member(f32::id(), "x")
        .member(f32::id(), "y");

    // Create entity
    let e = world.entity();

    // unchecked add id due to position being uninitialized and not having a ctor.
    unsafe {
        e.add_id_unchecked(position);
    }

    // set value of position using reflection API
    let ptr = e.get_untyped_mut(position);

    let mut cur = world.cursor_id(position, ptr);
    cur.push();
    cur.set_float(10.0);
    cur.next();
    cur.set_float(20.0);
    cur.pop();

    // Convert component to string
    println!("{:?}", world.to_expr_id(position, ptr));

    // Output
    //  {x: 10, y: 20}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_runtime_component".to_string());
}
