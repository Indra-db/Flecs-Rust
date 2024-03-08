#[allow(unused_variables)]
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
    let mut _query = world.query::<(&mut Position,)>();

    let _entity: Option<Entity> = None;
    todo!("^ find function not implemented on query, placeholder None");

    #[allow(unreachable_code)]
    if let Some(entity) = _entity {
        println!("Entity found: {:?}", entity.get_hierarchy_path());
    } else {
        println!("Entity not found");
    }
}
