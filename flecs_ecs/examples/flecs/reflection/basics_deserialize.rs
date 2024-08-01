use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Default, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[test]
fn main() {
    let world = World::new();

    // Register the Position component with reflection data
    world
        .component::<Position>()
        .member::<f32>("x", 1 /* count */, offset_of!(Position, x))
        .member::<f32>("y", 1, offset_of!(Position, y));

    // Create a new entity, set value of position using reflection API
    let e = world.entity().add::<Position>();

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
