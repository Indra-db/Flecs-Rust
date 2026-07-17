use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// The meta attribute captures reflection data for Color and Car.
#[derive(Debug, Component)]
#[repr(C)]
#[flecs(meta)]
pub enum Color {
    Red,
    Black,
    White,
    StainlessSteel,
}

#[derive(Debug, Component)]
#[flecs(meta)]
pub struct Car {
    pub brand: String,
    pub color: Color,
    pub speed: f32,
}

fn main() {
    let world = World::new();

    // Register Color explicitly so its reflection data is available when Car
    // is registered.
    world.component::<Color>();

    // Serialize as usual. Component registration automatically detects the
    // reflection data for Car.
    let value = Car {
        brand: "Delorean".to_string(),
        color: Color::StainlessSteel,
        speed: 1.21,
    };
    println!("{}", world.to_json::<Car>(&value));

    // Output:
    //  {"brand":"Delorean", "color":"StainlessSteel", "speed":1.2100000381}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_auto_define_enum".to_string());
}
