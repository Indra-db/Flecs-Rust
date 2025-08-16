use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// This example shows how to serialize a component with std::vectors

// Flecs Rust Meta framework pre-registers vector types of primitives.
// see `src/addons/meta/builtin.rs` for more information on what types exactly.

#[derive(Component, Debug)]
#[flecs(meta)]
struct VectorComponent {
    ints: Vec<i32>,
    strings: Vec<String>,
}

fn main() {
    let world = World::new();

    // Create value & serialize it to JSON
    let mut v = VectorComponent {
        ints: vec![1, 2, 3],
        strings: vec!["foo".to_string(), "bar".to_string()],
    };

    println!("{:?}", world.to_json::<VectorComponent>(&v));

    // Deserialize new values from JSON into value
    world.from_json::<VectorComponent>(
        &mut v,
        "{\"ints\": [4, 5], \"strings\":[\"hello\", \"flecs\", \"reflection\"]}",
        None,
    );

    // Serialize again
    println!("{:?}", world.to_json::<VectorComponent>(&v));

    // Output:
    //   {"ints":[1, 2, 3], "strings":["foo", "bar"]}
    //   {"ints":[4, 5], "strings":["hello", "flecs", "reflection"]}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_ser_std_vector_builtin_types".to_string());
}
