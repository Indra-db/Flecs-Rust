use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

fn main() {
    let world = World::new();

    let point = world
        .component_untyped_named("Point")
        .member(f32::id(), "x")
        .member(f32::id(), "y");

    let line = world
        .component_untyped_named("Line")
        .member(point, "start")
        .member(point, "stop");

    // Create entity, set value of line using reflection API
    let e = unsafe { world.entity().add_id_unchecked(line) };

    let ptr = e.get_untyped_mut(line);

    let mut cur = unsafe { world.cursor_id(line, ptr) };

    #[rustfmt::skip]
    fn cursor(cur: &mut Cursor) {
        cur.push();            // {
        cur.push();            //   {
        cur.set_float(10.0);   //     10
        cur.next();            //     ,
        cur.set_float(20.0);   //     20
        cur.pop();             //   }
        cur.next();            //   ,
        cur.push();            //   {
        cur.set_float(30.0);   //     30
        cur.next();            //     ,
        cur.set_float(40.0);   //     40
        cur.pop();             //   }
        cur.pop();             // }
    }

    // we use a function to format skip the comments for better understanding.
    // in normal cases, you can just write the code directly.
    cursor(&mut cur);

    // Convert component to string
    println!("{:?}", unsafe { world.to_expr_id(line, ptr) });

    // Output
    // {start: {x: 10.00, y: 20.00}, stop: {x: 30.00, y: 40.00}}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_runtime_nested_component".to_string());
}
