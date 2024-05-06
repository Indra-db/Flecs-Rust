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

#[derive(Component)]
pub struct Mass {
    pub value: f32,
}

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    let query = world.new_query::<(&mut Position, &Velocity)>();

    // Create a few test entities for a Position, Velocity query
    world
        .entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    world
        .entity_named(c"e3")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 4.0, y: 5.0 })
        .set(Mass { value: 50.0 });

    // The iter function provides a flecs::iter object which contains all sorts
    // of information on the entities currently being iterated.
    // The function passed to iter is by default called for each table the query
    // is matched with.
    query.iter(|it, (position, velocity)| {
        fprintln!(it);
        // Print the table & number of entities matched in current callback
        fprintln!(it, "Table: {:?}", it.archetype());
        fprintln!(it, " - number of entities: {}", it.count());
        fprintln!(it);

        // Print information about the components being matched
        for i in 0..it.field_count() {
            fprintln!(it, " - term {} : ", i);
            fprintln!(it, "   - component: {}", it.id(i).to_str());
            fprintln!(it, "   - type size: {}", it.size(i));
        }

        fprintln!(it);

        for i in it.iter() {
            position[i].x += velocity[i].x;
            position[i].y += velocity[i].y;
            fprintln!(
                it,
                " - entity {}: has {:?}",
                it.entity(i).name(),
                position[i]
            );
        }

        fprintln!(it);
    });

    world.get::<Snap>().test("query_iter".to_string());

    // Output:
    //  Table: Position, Velocity, (Identifier,Name)
    //  - number of entities: 2
    //
    //  - term 1 :
    //    - component: Position
    //    - type size: 8
    //  - term 2 :
    //    - component: Velocity
    //    - type size: 8
    //
    //  - entity e1: has Position { x: 11.0, y: 22.0 }
    //  - entity e2: has Position { x: 13.0, y: 24.0 }
    //
    //
    // Table: Position, Velocity, Mass, (Identifier,Name)
    //  - number of entities: 1
    //
    //  - term 1 :
    //    - component: Position
    //    - type size: 8
    //  - term 2 :
    //    - component: Velocity
    //    - type size: 8
    //
    //  - entity e3: has Position { x: 14.0, y: 25.0 }
    //
}
