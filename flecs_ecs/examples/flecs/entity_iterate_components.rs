use crate::z_ignore_test_common::*;

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
pub struct Eats;

#[derive(Component)]
pub struct Apples;

#[derive(Component)]
pub struct Human;

fn iterate_components(entity: EntityView) {
    // 1. The easiest way to print the components is to use archetype
    println!("[{:?}]", entity.archetype());
    println!();
    // 2. To get individual component ids, use for_each
    let mut count_components = 0;
    entity.each_component(|id| {
        println!("{}: {}", count_components, id.to_str());
        count_components += 1;
    });
    println!();

    // 3. we can also inspect and print the ids in our own way. This is a
    // bit more complicated as we need to handle the edge cases of what can be
    // encoded in an id, but provides the most flexibility.
    count_components = 0;

    entity.each_component(|id| {
        let mut string: String = String::new();
        string.push_str(format!("{}: ", count_components).as_str());

        count_components += 1;
        if id.is_pair() {
            // If id is a pair, extract & print both parts of the pair
            let rel = id.first_id();
            let target = id.second_id();
            string.push_str(format!("rel: {}, target: {}", rel.name(), target.name()).as_str());
        } else {
            // Id contains a regular entity. Strip role before printing.
            let comp = id.entity_view();
            string.push_str(format!("entity: {}", comp.name()).as_str());
        }

        println!("{}", string);
    });
}

fn main() {
    let world = World::new();

    let bob = world
        .entity_named("Bob")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 1.0 })
        .add::<Human>()
        .add::<(Eats, Apples)>();

    println!("Bob's components:");
    iterate_components(bob);

    println!();

    // We can use the same function to iterate the components of a component
    println!("Position's components:");
    iterate_components(world.component::<Position>().entity());

    // Output:
    //  Bob's components:
    //  [Position, Velocity, Human, (Identifier,Name), (Eats,Apples)]
    //
    //  0: Position
    //  1: Velocity
    //  2: Human
    //  3: (Identifier,Name)
    //  4: (Eats,Apples)
    //
    //  0: entity: Position
    //  1: entity: Velocity
    //  2: entity: Human
    //  3: rel: Identifier, target: Name
    //  4: rel: Eats, target: Apples
    //
    //  Position's components:
    //  [Component, (Identifier,Name), (Identifier,Symbol)
    //
    //  0: Component
    //  1: (Identifier,Name)
    //  2: (Identifier,Symbol)
    //  3: (ChildOf,entity_iterate_components.common)

    //  0: entity: Component
    //  1: rel: Identifier, target: Name
    //  2: rel: Identifier, target: Symbol
    //  3: rel: ChildOf, target: common
}

#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("entity_iterate_components".to_string());
}
