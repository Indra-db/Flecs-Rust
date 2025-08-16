use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// This example shows how to serialize a component with std::vectors

// Flecs Rust Meta framework pre-registers vector types of primitives.
// see `src/addons/meta/builtin.rs` for more information on what types exactly.

#[derive(Component, Debug)]
#[flecs(meta)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
#[flecs(meta)]
struct VectorComponent {
    points: Vec<Point>,
    strings: Vec<String>,
}

fn main() {
    let world = World::new();

    //register point component with meta
    world.component::<Point>().meta();

    // String and vec<String> are already pre-registered by the meta framework

    // register vec<Point> component with meta
    // we have to pass a default value for the Point struct that
    // will be used to create new elements in the vector
    meta_register_vector_type!(&world, Point { x: 0.0, y: 0.0 });

    // Register component with std::vector members
    world.component::<VectorComponent>().meta();

    // Create value & serialize it to JSON
    let mut v = VectorComponent {
        points: vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }],
        strings: vec!["foo".to_string(), "bar".to_string()],
    };

    println!("{:?}", world.to_json::<VectorComponent>(&v));

    // Deserialize new values from JSON into value
    world.from_json::<VectorComponent>(
        &mut v,
        "{\"points\": [{\"x\": 4.0, \"y\": 5.0}, {\"x\": 6.0, \"y\": 7.0}], \"strings\":[\"hello\", \"flecs\", \"reflection\"]}",
        None,
    );

    // Serialize again
    println!("{:?}", world.to_json::<VectorComponent>(&v));

    // Output:
    //  "{"points":[{"x":1, "y":2}, {"x":3, "y":4}], "strings":["foo", "bar"]}"
    //  "{"points":[{"x":4, "y":5}, {"x":6, "y":7}, {"x":0, "y":0}], "strings":["hello", "flecs", "reflection", ""]}"
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_ser_std_vector_custom_types".to_string());
}
