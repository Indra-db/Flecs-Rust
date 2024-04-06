#[allow(unused_variables)]
mod common;
use common::*;

fn main() {
    let world = World::new();

    // Create a few test entities for a Position query
    world
        .new_entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 });

    world
        .new_entity_named(c"e2")
        .set(Position { x: 20.0, y: 30.0 });

    // Create a simple query for component Position
    let query = world.query::<(&mut Position,)>();

    let entity: Option<Entity> = query.find(|(pos,)| pos.x == 20.0);

    if let Some(entity) = entity {
        println!("Entity found: {:?}", entity.path().unwrap());
    } else {
        println!("Entity not found");
    }

    // Output:
    //  Entity found: "::e2"
}
