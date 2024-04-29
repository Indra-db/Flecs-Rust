include!("common");

#[derive(Component)]
struct Npc;

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create a query for Position, Npc. By adding the Npc component using the
    // "with" method, the component is not a part of the query type, and as a
    // result does not become part of the function signatures of each and iter.
    // This is useful for things like tags, which because they don't have a
    // value are less useful to pass to the each/iter functions as argument.
    let query = world.query::<&Position>().with::<&Npc>().build();

    // Create a few test entities for the Position, Npc query
    world
        .entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 })
        .add::<Npc>();

    world
        .entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 })
        .add::<Npc>();

    // This entity will not match as it does not have Position, Npc
    world.entity_named(c"e3").set(Position { x: 10.0, y: 20.0 });

    // Note how the Npc tag is not part of the each signature
    query.each_entity(|entity, pos| {
        fprintln!(entity, "Entity {}: {:?}", entity.name(), pos);
    });

    Ok(Snap::from(&world))

    // Output:
    //  Entity e1: Position { x: 10.0, y: 20.0 }
    //  Entity e2: Position { x: 10.0, y: 20.0 }
}
