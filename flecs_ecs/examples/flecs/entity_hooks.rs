use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Tag;

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    world
        .component::<Position>()
        .on_add(|entity, _pos| {
            fprintln!(entity, "added Position to {:?}", entity.name());
        })
        .on_remove(|entity, pos| {
            fprintln!(entity, "removed {:?} from {:?}", pos, entity.name());
        })
        .on_set(|entity, pos| {
            fprintln!(entity, "set {:?} for {:?}", pos, entity.name());
        });

    let entity = world.entity_named(c"Bob");

    entity.add::<Position>();

    entity.set(Position { x: 10.0, y: 20.0 });

    // This operation changes the entity's archetype, which invokes a move
    entity.add::<Tag>();

    entity.destruct();

    // Output:
    //  added Position { x: 0.0, y: 0.0 } to "Bob"
    //  set Position { x: 10.0, y: 20.0 } for "Bob"
    //  removed Position { x: 10.0, y: 20.0 } from "Bob"

    world.get::<&Snap>(|snap| snap.test("entity_hooks".to_string()));
}
