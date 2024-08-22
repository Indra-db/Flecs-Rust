use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Component, Default)]
pub struct Velocity {
    pub x: i32,
    pub y: i32,
}

fn main() {
    let world = World::new();

    world.set(Position { x: 1, y: 2 });

    let query = world
        .query::<(&Position, &Velocity)>()
        .term_at(0)
        .singleton()
        .set_cached()
        .build();

    world.entity().set(Velocity { x: 590, y: 20 });

    let query_entity = query.entity().id();

    let sys = world.system::<()>().run(move |it| {
        let world = it.world();
        let query = world.query_from(query_entity);
        query.run(|mut it| {
            while it.next() {
                let pos = &it.field::<&Position>(0).unwrap()[0]; //singleton
                let vel = it.field::<&Velocity>(1).unwrap();
                for i in it.iter() {
                    println!("{:?}, {:?}", pos, vel[i]);
                }
            }
        });
    });

    sys.run();

    // Output:
    //  Position { x: 1.0, y: 2.0 }, Velocity { x: 590.0, y: 20.0 }
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_pass_query_to_system".to_string());
}
