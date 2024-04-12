mod common;
use common::*;

#[derive(Component)]
struct Npc;

fn main() {
    let world = World::new();

    // Create a query for Position, !Npc. By adding the Npc component using the
    // "without" method, the component is not a part of the query type, and as a
    // result does not become part of the function signatures of each and iter.
    // This is useful for things like tags, which because they don't have a
    // value are less useful to pass to the each/iter functions as argument.
    //
    // The without method is short for:
    //   .term<Npc>().not_()
    let query = world.query::<&Position>().without_type::<&Npc>().build();

    // Create a few test entities for the Position query
    world
        .new_entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 });

    world
        .new_entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 });

    // This entity will not match as it has Npc
    world
        .new_entity_named(c"e3")
        .set(Position { x: 10.0, y: 20.0 })
        .add::<Npc>();

    // Note how the Npc tag is not part of the each signature
    query.each_entity(|entity, pos| {
        println!("Entity {}: {:?}", entity.name(), pos);
    });

    // Output:
    //  Entity e1: Position { x: 10.0, y: 20.0 }
    //  Entity e2: Position { x: 10.0, y: 20.0 }
}
