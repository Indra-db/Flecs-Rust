use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// The meta attribute captures reflection data for Point and Line.
#[derive(Debug, Component)]
#[flecs(meta)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Component)]
#[flecs(meta)]
pub struct Line {
    pub start: Point,
    pub stop: Point,
}

fn main() {
    let world = World::new();

    // Register Point explicitly so its reflection data is available when Line
    // is registered.
    world.component::<Point>();

    // Serialize as usual. Component registration automatically detects the
    // reflection data for Line.
    let value = Line {
        start: Point { x: 1, y: 2 },
        stop: Point { x: 3, y: 4 },
    };
    println!("{}", world.to_json::<Line>(&value));

    // Output:
    //  {"start":{"x":1, "y":2}, "stop":{"x":3, "y":4}}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_auto_define_nested_struct".to_string());
}
