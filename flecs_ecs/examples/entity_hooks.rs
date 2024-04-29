include!("common");
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    let snap = Arc::new(Mutex::new(Snap::setup_snapshot_test()));
    let snap_clone_add = Arc::clone(&snap);
    let snap_clone_remove = Arc::clone(&snap);
    let snap_clone_set = Arc::clone(&snap);

    let world = World::new();

    world
        .component::<Position>()
        .on_add(move |entity, _pos| {
            let mut snap = snap_clone_add.lock().unwrap();
            fprintln!(snap, "added Position to {:?}", entity.name());
        })
        .on_remove(move |entity, pos| {
            let mut snap = snap_clone_remove.lock().unwrap();
            fprintln!(snap, "removed {:?} from {:?}", pos, entity.name());
        })
        .on_set(move |entity, pos| {
            let mut snap = snap_clone_set.lock().unwrap();
            fprintln!(snap, "set {:?} for {:?}", pos, entity.name());
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

    //snap shot testing, ignore
    let mut guard = snap.lock().unwrap();
    let value = std::mem::replace(&mut *guard, Snap::setup_snapshot_test());
    Ok(value)
}
