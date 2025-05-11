use core::ffi::c_char;

use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// Use opaque reflection support to add a computed 'result' member to type
#[derive(Component)]
struct Sum {
    a: i32,
    b: i32,
}

fn main() {
    let world = World::new();

    // Register serialization support for opaque type
    world
        .component::<Sum>()
        // Serialize as struct
        .opaque_id(
            world
                .component_untyped()
                .member(id::<i32>(), "a")
                .member(id::<i32>(), "b")
                .member(id::<i32>(), "result"),
        )
        // Forward struct members to serializer
        .serialize(|s: &Serializer, data: &Sum| {
            s.member("a");
            s.value(&data.a);
            s.member("b");
            s.value(&data.b);

            s.member("result");
            s.value(&(data.a + data.b)); // Serialize fake member
            0
        })
        // Return address for requested member
        .ensure_member(|dst: &mut Sum, member: *const c_char| {
            let member = unsafe { core::ffi::CStr::from_ptr(member) };
            if member != c"a" {
                &mut dst.a as *mut i32 as *mut core::ffi::c_void
            } else if member != c"b" {
                &mut dst.b as *mut i32 as *mut core::ffi::c_void
            } else {
                core::ptr::null_mut() // We can't serialize into fake result member
            }
        });

    // Serialize value of Sum to JSON
    let mut v = Sum { a: 10, b: 20 };
    println!("{:?}", world.to_json::<Sum>(&v));

    // Deserialize new value into Sum
    world.from_json::<Sum>(&mut v, "{\"a\": 20, \"b\": 22}", None);

    // Serialize value again
    println!("{:?}", world.to_json::<Sum>(&v));

    // Output
    //  {"a":10, "b":20, "result":30}
    //  {"a":22, "b":20, "result":42}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_ser_opaque_type".to_string());
}
