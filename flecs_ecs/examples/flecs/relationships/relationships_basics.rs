use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct Eats;

fn main() {
    let world = World::new();

    // Entity used for Grows relationship
    let grows = world.entity_named("Grows");

    // Relationship objects
    let apples = world.entity_named("Apples");
    let pears = world.entity_named("Pears");

    // Create an entity with 3 relationships. Relationships are like regular components,
    // but combine two types/identifiers into an (relationship, object) pair.
    let bob = world
        .entity_named("Bob")
        // Pairs can be constructed from a type and entity
        .add((Eats, apples))
        .add((Eats, pears))
        // Pairs can also be constructed from two entity ids
        .add((grows, pears));

    // Has can be used with relationships as well
    println!("Bob eats apples? {}", bob.has((Eats, apples)));

    // Wildcards can be used to match relationships
    println!(
        "Bob grows food? {}, {}",
        bob.has((grows, flecs::Wildcard::ID)),
        //or you can do
        bob.has((grows, id::<flecs::Wildcard>()))
    );

    println!();

    // Print the type of the entity. Should output:
    //   (Identifier,Name),(Eats,Apples),(Eats,Pears),(Grows,Pears)
    println!("Bob's type: [{}]", bob.archetype());

    println!();

    // Relationships can be iterated for an entity. This iterates (Eats, *):
    bob.each_target(Eats, |second| {
        println!("Bob eats {}", second.name());
    });

    println!();

    // Iterate by explicitly providing the pair. This iterates (*, Pears):
    bob.each_pair(flecs::Wildcard::ID, pears, |id| {
        println!("Bob {} pears", id.first_id().name());
    });

    println!();

    // Get first target of relationship
    println!("Bob eats {}", bob.target(Eats, 0).unwrap().name());

    // Get second target of relationship
    println!("Bob also eats {}", bob.target(Eats, 1).unwrap().name());

    // Output:
    //  Bob eats apples? true
    //  Bob grows food? true, true

    //  Bob's type: [(Identifier,Name), (Eats,Apples), (Eats,Pears), (Grows,Pears)]

    //  Bob eats Apples
    //  Bob eats Pears

    //  Bob Eats pears
    //  Bob Grows pears

    //  Bob eats Apples
    //  Bob also eats Pears
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("relationships_basics".to_string());
}
