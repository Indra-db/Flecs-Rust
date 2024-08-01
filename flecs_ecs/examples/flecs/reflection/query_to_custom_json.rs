//! Same example as `query_to_json`, but with customized serializer parameters

use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
use flecs_ecs::sys;
use json::IterToJsonDesc;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Mass {
    pub value: f32,
}

#[test]
fn main() {
    let world = World::new();

    // Register components with reflection data
    world
        .component::<Position>()
        .member::<f32>("x", 1, offset_of!(Position, x))
        .member::<f32>("y", 1, offset_of!(Position, y));

    world
        .component::<Velocity>()
        .member::<f32>("x", 1, offset_of!(Velocity, x))
        .member::<f32>("y", 1, offset_of!(Velocity, y));

    world
        .component::<Mass>()
        .member::<f32>("value", 1, offset_of!(Mass, value));

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

    // Serialize query to JSON. Customize serializer to only serialize entity names and component values.
    let desc = IterToJsonDesc {
        serialize_values: true,
        serialize_fields: true,
        ..Default::default()
    };

    println!("{}", q.iterable().to_json(Some(&desc)).unwrap());

    // Output:
    //   {
    //       "results": [
    //         {
    //           "name": "a",
    //           "fields": {
    //             "values": [
    //               {
    //                 "x": 10,
    //                 "y": 20
    //               },
    //               {
    //                 "x": 1,
    //                 "y": 2
    //               }
    //             ]
    //           }
    //         },
    //         {
    //           "name": "b",
    //           "fields": {
    //             "values": [
    //               {
    //                 "x": 20,
    //                 "y": 30
    //               },
    //               {
    //                 "x": 2,
    //                 "y": 3
    //               }
    //             ]
    //           }
    //         },
    //         {
    //           "name": "c",
    //           "fields": {
    //             "values": [
    //               {
    //                 "x": 30,
    //                 "y": 40
    //               },
    //               {
    //                 "x": 3,
    //                 "y": 4
    //               }
    //             ]
    //           }
    //         },
    //         {
    //           "name": "d",
    //           "fields": {
    //             "values": [
    //               {
    //                 "x": 30,
    //                 "y": 40
    //               },
    //               {
    //                 "x": 4,
    //                 "y": 5
    //               }
    //             ]
    //           }
    //         }
    //       ]
    //     }
}
