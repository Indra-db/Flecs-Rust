use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// The meta attribute captures reflection data for Position.
#[derive(Debug, Component)]
#[flecs(meta)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

fn main() {
    let world = World::new();

    // Serialize as usual. Component registration automatically detects the
    // reflection data.
    let value = Position { x: 10.0, y: 20.0 };
    println!("{}", world.to_json::<Position>(&value));

    // Output:
    //  {"x":10, "y":20}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_auto_define_struct".to_string());
}
