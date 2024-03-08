mod common;
use common::*;

fn main() {
    let world = World::new();

    // Create a few test entities for a Position query
    world
        .new_entity_named(CStr::from_bytes_with_nul(b"e1\0").unwrap())
        .set(Position { x: 10.0, y: 20.0 });

    world
        .new_entity_named(CStr::from_bytes_with_nul(b"e2\0").unwrap())
        .set(Position { x: 10.0, y: 20.0 });

    // Create a simple query for component Position
    let mut query = world.query::<(&mut Position,)>();

    let entity: Option<Entity> = todo!("find function not implemented on query");

    if let Some(entity) = entity {
        println!("Entity found: {:?}", entity.get_hierarchy_path());
    } else {
        println!("Entity not found");
    }
}
