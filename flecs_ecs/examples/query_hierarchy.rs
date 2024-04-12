mod common;
use common::*;

#[derive(Debug, Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Component)]
struct Local;

#[derive(Debug, Component)]
struct World;

fn main() {
    let world = flecs_ecs::core::World::new();

    let sun = world
        .new_entity_named(c"Sun")
        .add::<(Position, World)>()
        .set_pair_first::<Position, Local>(Position { x: 1.0, y: 1.0 });

    world
        .new_entity_named(c"Mercury")
        .child_of_id(sun)
        .add::<(Position, World)>()
        .set_pair_first::<Position, Local>(Position { x: 1.0, y: 1.0 });

    world
        .new_entity_named(c"Venus")
        .child_of_id(sun)
        .add::<(Position, World)>()
        .set_pair_first::<Position, Local>(Position { x: 2.0, y: 2.0 });

    let earth = world
        .new_entity_named(c"Earth")
        .child_of_id(sun)
        .add::<(Position, World)>()
        .set_pair_first::<Position, Local>(Position { x: 3.0, y: 3.0 });

    world
        .new_entity_named(c"Moon")
        .child_of_id(earth)
        .add::<(Position, World)>()
        .set_pair_first::<Position, Local>(Position { x: 0.1, y: 0.1 });

    let query = world
        .query::<(&Position, Option<&Position>, &mut Position)>()
        .term_at(1)
        .select_second::<Local>()
        .term_at(2)
        .select_second::<World>()
        .term_at(3)
        .select_second::<World>()
        .term_at(2)
        .parent()
        .cascade()
        //.optional() -- `.optional()` is equivalent to `Option<&Position>` - however be aware that
        // this won't provide a nice API with `Option<&Position>` but rather return a slice where you have to do
        // `.as_ptr().is_null()` to check if the value is actually valid. This will likely be removed from the API once
        // we have tests in place to ensure that the `Option` API is working as expected.
        .build();

    query.iter(|it, (local, parent_world, world)| {
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
        .filter::<&Position>()
        .term_at(1)
        .select_second::<World>()
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
