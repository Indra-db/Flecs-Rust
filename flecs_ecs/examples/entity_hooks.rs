mod common;
use common::*;

fn main() {
    let world = World::new();

    world
        .component::<Position>()
        .on_add(|entity, pos| {
            println!("added {:?} to {:?}", pos, entity.get_name());
        })
        .on_remove(|entity, pos| {
            println!("removed {:?} from {:?}", pos, entity.get_name());
        })
        .on_set(|entity, pos| {
            println!("set {:?} for {:?}", pos, entity.get_name());
        });

    let entity = world.new_entity_named(CStr::from_bytes_with_nul(b"Bob\0").unwrap());

    entity.add::<Position>();

    entity.set(Position { x: 10.0, y: 20.0 });

    // This operation changes the entity's archetype, which invokes a move
    entity.add::<Tag>();

    entity.destruct();

    // Output
    //  added Position { x: 0.0, y: 0.0 } to "Bob"
    //  set Position { x: 10.0, y: 20.0 } for "Bob"
    //  removed Position { x: 10.0, y: 20.0 } from "Bob"
}
