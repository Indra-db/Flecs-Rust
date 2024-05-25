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
        .query::<(&Position, Option<&Position>, &mut Position)>()
        .term_at(0)
        .set_second::<Local>()
        .term_at(1)
        .set_second::<WorldX>()
        .term_at(2)
        .set_second::<WorldX>()
        .term_at(1)
        .parent()
        .cascade()
        //.optional() -- `.optional()` is equivalent to `Option<&Position>` - however be aware that
        // this won't provide a nice API with `Option<&Position>` but rather return a slice where you have to do
        // `.as_ptr().is_null()` to check if the value is actually valid. This will likely be removed from the API once
        // we have tests in place to ensure that the `Option` API is working as expected.
        .build();

    query.run_iter(|it, (local, parent_world, world)| {
        for i in it.iter() {
            world[i].x = local[i].x;
            world[i].y = local[i].y;
            if parent_world.is_some() {
                let parent_world = parent_world.unwrap();
                world[i].x += parent_world[i].x;
                world[i].y += parent_world[i].y;
            }
        }
    });

    //TODO: pair wrapper class to clean up, beautify this API
    world
        .query::<&Position>()
        .term_at(0)
        .set_second::<WorldX>()
        .build()
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
