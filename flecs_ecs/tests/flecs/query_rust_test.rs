#![allow(dead_code)]
use flecs_ecs::core::*;

use crate::common_test::*;

#[test]
fn query_rust_pass_query_to_system() {
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
            let mut count = 0;
            while it.next() {
                let pos = &it.field::<&Position>(0).unwrap()[0]; //singleton
                let vel = it.field::<&Velocity>(1).unwrap();
                for i in it.iter() {
                    count += 1;
                    assert_eq!(pos.x, 1);
                    assert_eq!(pos.y, 2);
                    assert_eq!(vel[i].x, 590);
                    assert_eq!(vel[i].y, 20);
                }
            }
            assert_eq!(count, 1);
        });
    });

    sys.run();
}
