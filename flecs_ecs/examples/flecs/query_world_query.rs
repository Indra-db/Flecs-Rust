use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create a few test entities for a Position, Velocity query
    world
        .entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    // This entity will not match as it does not have Position, Velocity
    world.entity_named(c"e3").set(Position { x: 10.0, y: 20.0 });

    // Ad hoc queries are bit slower to iterate than flecs::query, but are
    // faster to create, and in most cases require no allocations. Under the
    // hood this API uses flecs::filter, which can be used directly for more
    // complex queries.

    world.each_entity::<(&mut Position, &Velocity)>(|entity, (pos, vel)| {
        pos.x += vel.x;
        pos.y += vel.y;
        fprintln!(entity, "Entity {}: {:?}", entity.name(), pos);
    });

    world.get::<Snap>().test("query_world_query".to_string());

    // Output:
    //  Entity e1: Position { x: 11.0, y: 22.0 }
    //  Entity e2: Position { x: 13.0, y: 24.0 }
}
