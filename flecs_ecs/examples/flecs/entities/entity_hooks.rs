use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Tag;

fn main() {
    let world = World::new();

    world
        .component::<Position>()
        .on_add(|entity, _pos| {
            println!("added Position to {:?}", entity.name());
        })
        .on_remove(|entity, pos| {
            println!("removed {:?} from {:?}", pos, entity.name());
        })
        .on_set(|entity, pos| {
            println!("set {:?} for {:?}", pos, entity.name());
        });

    let entity = world.entity_named("Bob");

    entity.set(Position { x: 10.0, y: 20.0 });

    // This operation changes the entity's archetype, which invokes a move
    // add is used for adding tags.
    entity.add(Tag::id());

    entity.destruct();

    // Output:
    //  added Position { x: 0.0, y: 0.0 } to "Bob"
    //  set Position { x: 10.0, y: 20.0 } for "Bob"
    //  removed Position { x: 10.0, y: 20.0 } from "Bob"
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("entity_hooks".to_string());
}
