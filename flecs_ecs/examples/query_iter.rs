mod common;
use common::*;

fn main() {
    let world = World::new();

    let query = world.query::<(&mut Position, &Velocity)>();

    // Create a few test entities for a Position, Velocity query
    world
        .new_entity_named(c"e1")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 2.0 });

    world
        .new_entity_named(c"e2")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 3.0, y: 4.0 });

    world
        .new_entity_named(c"e3")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 4.0, y: 5.0 })
        .set(Mass { value: 50.0 });

    // The iter function provides a flecs::iter object which contains all sorts
    // of information on the entities currently being iterated.
    // The function passed to iter is by default called for each table the query
    // is matched with.
    query.iter(|it, (position, velocity)| {
        println!();
        // Print the table & number of entities matched in current callback
        println!("Table: {}", it.get_archetype());
        println!(" - number of entities: {}", it.count());
        println!();

        // Print information about the components being matched
        for i in 1..=it.get_field_count() {
            println!(" - term {} : ", i);
            println!("   - component: {}", it.get_field_id(i).to_str());
            println!("   - type size: {}", it.get_field_size(i));
        }

        println!();

        for i in it.iter() {
            position[i].x += velocity[i].x;
            position[i].y += velocity[i].y;
            println!(
                " - entity {}: has {:?}",
                it.get_entity(i).get_name(),
                position[i]
            );
        }

        println!();
    });

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
