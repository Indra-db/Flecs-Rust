use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

fn main() {
    let world = World::new();

    let position = world
        .component_untyped_named("Position")
        .member::<f32>("x")
        .member::<f32>("y");

    // Create entity, set value of position using reflection API
    let e = world.entity().add_id(position);

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
