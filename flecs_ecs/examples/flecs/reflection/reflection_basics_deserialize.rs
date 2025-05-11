use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Default, Component)]
#[meta]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

fn main() {
    let world = World::new();

    // Register the Position component with reflection data
    world.component::<Position>().meta();

    /* Alternatively, you can do it manually like so (without the derive macro)
    .member(id::<f32>(),"x", 1 /* count */, core::mem::offset_of!(Position, x))
    .member(id::<f32>(),"y", 1, core::mem::offset_of!(Position, y));
    */

    // Create a new entity, set value of position using reflection API
    let e = world.entity().add(id::<Position>());

    e.get::<&mut Position>(|pos| {
        let mut cur = world.cursor::<Position>(pos);
        cur.push(); // {
        cur.set_float(10.0); //   10
        cur.next(); //   ,
        cur.set_float(20.0); //   20
        cur.pop(); // }

        println!("{}", world.to_expr(pos));
    });

    // Use member names before assigning values
    e.get::<&mut Position>(|pos| {
        let mut cur = world.cursor::<Position>(pos);
        cur.push(); // {
        cur.member("y"); //   y:
        cur.set_float(10.0); //   10
        cur.member("x"); //   x:
        cur.set_float(20.0); //   20
        cur.pop(); // }

        println!("{}", world.to_expr(pos));
    });

    // Output:
    //  {x: 10, y: 20}
    //  {x: 20, y: 10}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("reflection_deserialize".to_string());
}
