use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// This example shows how to serialize a component with an std::string

// std::String is already registered in the Rust Meta framework (unlike in CPP)
// in case you wish to see how, find it in `src/addons/meta/builtin.rs`
// thus serializing components with String is already supported out of the box

#[derive(Component, Debug)]
#[meta]
struct StringComponent {
    a: String,
    b: String,
}

fn main() {
    let world = World::new();

    // Register component with std::string members
    world.component::<StringComponent>().meta();

    // Create value & serialize it
    let mut v = StringComponent {
        a: "foo".to_string(),
        b: "bar".to_string(),
    };

    println!("{:?}", world.to_json::<StringComponent>(&v));

    // Deserialize new strings into value
    world.from_json::<StringComponent>(&mut v, "{\"a\": \"hello\", \"b\": \"world\"}", None);
    println!("{:?}", v);

    // Output:
    //   {"a": "foo", "b": "bar"}
    //   {a: "hello", b: "world"}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_ser_std_string".to_string());
}
