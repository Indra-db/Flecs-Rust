use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Component)]
#[flecs(meta)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
#[flecs(meta)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
#[flecs(meta)]
pub struct Mass {
    pub value: f32,
}

fn main() {
    let world = World::new();

    world
        .entity_named("a")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .entity_named("b")
        .set(Position { x: 20.0, y: 30.0 })
        .set(Velocity { x: 2.0, y: 3.0 });

    world
        .entity_named("c")
        .set(Position { x: 30.0, y: 40.0 })
        .set(Velocity { x: 3.0, y: 4.0 })
        .set(Mass { value: 10.0 });

    world
        .entity_named("d")
        .set(Position { x: 30.0, y: 40.0 })
        .set(Velocity { x: 4.0, y: 5.0 })
        .set(Mass { value: 20.0 });

    // Query for components
    let q = world.new_query::<(&mut Position, &Velocity)>();

    println!("{}", q.iterable().to_json(None).unwrap());

    // Output:
    //  {
    //      "results": [
    //        {
    //          "name": "a",
    //          "fields": {
    //            "values": [
    //              {
    //                "x": 10,
    //                "y": 20
    //              },
    //              {
    //                "x": 1,
    //                "y": 2
    //              }
    //            ]
    //          }
    //        },
    //        {
    //          "name": "b",
    //          "fields": {
    //            "values": [
    //              {
    //                "x": 20,
    //                "y": 30
    //              },
    //              {
    //                "x": 2,
    //                "y": 3
    //              }
    //            ]
    //          }
    //        },
    //        {
    //          "name": "c",
    //          "fields": {
    //            "values": [
    //              {
    //                "x": 30,
    //                "y": 40
    //              },
    //              {
    //                "x": 3,
    //                "y": 4
    //              }
    //            ]
    //          }
    //        },
    //        {
    //          "name": "d",
    //          "fields": {
    //            "values": [
    //              {
    //                "x": 30,
    //                "y": 40
    //              },
    //              {
    //                "x": 4,
    //                "y": 5
    //              }
    //            ]
    //          }
    //        }
    //      ]
    //    }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_query_to_json".to_string());
}
