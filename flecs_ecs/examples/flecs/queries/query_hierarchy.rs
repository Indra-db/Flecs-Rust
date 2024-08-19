use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
struct Local;

#[derive(Debug, Component)]
struct WorldX;

fn main() {
    let world = World::new();

    let sun = world
        .entity_named("Sun")
        .set_pair::<Position, WorldX>(Position::default())
        .set_pair::<Position, Local>(Position { x: 1.0, y: 1.0 });

    world
        .entity_named("Mercury")
        .child_of_id(sun)
        .set_pair::<Position, WorldX>(Position::default())
        .set_pair::<Position, Local>(Position { x: 1.0, y: 1.0 });

    world
        .entity_named("Venus")
        .child_of_id(sun)
        .set_pair::<Position, WorldX>(Position::default())
        .set_pair::<Position, Local>(Position { x: 2.0, y: 2.0 });

    let earth = world
        .entity_named("Earth")
        .child_of_id(sun)
        .set_pair::<Position, WorldX>(Position::default())
        .set_pair::<Position, Local>(Position { x: 3.0, y: 3.0 });

    world
        .entity_named("Moon")
        .child_of_id(earth)
        .set_pair::<Position, WorldX>(Position::default())
        .set_pair::<Position, Local>(Position { x: 0.1, y: 0.1 });

    let query = world
        .query::<(
            &(Position, Local),
            Option<&(Position, WorldX)>,
            &mut (Position, WorldX),
        )>()
        .term_at(1)
        .parent()
        .cascade()
        .build();

    query.each(|(local, parent_world, world)| {
        world.x = local.x;
        world.y = local.y;
        if parent_world.is_some() {
            let parent_world = parent_world.unwrap();
            world.x += parent_world.x;
            world.y += parent_world.y;
        }
    });

    world
        .new_query::<&(Position, WorldX)>()
        .each_entity(|entity, position| {
            println!(
                "Entity {} is at ({}, {})",
                entity.name(),
                position.x,
                position.y
            );
        });

    // Output:
    //  Entity Sun is at (1, 1)
    //  Entity Mercury is at (2, 2)
    //  Entity Venus is at (3, 3)
    //  Entity Earth is at (4, 4)
    //  Entity Moon is at (4.1, 4.1)
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_hierarchy".to_string());
}
