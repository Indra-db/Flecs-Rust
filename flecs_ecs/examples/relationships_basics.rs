mod common;
pub use common::*;

fn main() {
    let world = World::new();

    // Entity used for Grows relationship
    let grows = world.new_entity_named(c"Grows");

    // Relationship objects
    let apples = world.new_entity_named(c"Apples");
    let pears = world.new_entity_named(c"Pears");

    // Create an entity with 3 relationships. Relationships are like regular components,
    // but combine two types/identifiers into an (relationship, object) pair.
    let bob = world
        .new_entity_named(c"Bob")
        // Pairs can be constructed from a type and entity
        .add_pair_second_id::<Eats>(apples.into())
        .add_pair_second_id::<Eats>(pears.into())
        // Pairs can also be constructed from two entity ids
        .add_pair_ids(grows.into(), pears.into());

    // Has can be used with relationships as well
    println!(
        "Bob eats apples? {}",
        bob.has_pair_first::<Eats>(apples.into())
    );

    // Wildcards can be used to match relationships
    println!(
        "Bob grows food? {}",
        bob.has_pair_ids(grows.into(), ECS_WILDCARD)
    );

    println!();

    // Print the type of the entity. Should output:
    //   (Identifier,Name),(Eats,Apples),(Eats,Pears),(Grows,Pears)
    println!("Bob's type: [{}]", bob.get_archetype());

    println!();

    // Relationships can be iterated for an entity. This iterates (Eats, *):
    bob.for_each_target_in_relationship::<Eats>(|second| {
        println!("Bob eats {}", second.get_name());
    });

    println!();

    // Iterate by explicitly providing the pair. This iterates (*, Pears):
    bob.for_each_matching_pair(ECS_WILDCARD, pears.into(), |id| {
        println!("Bob {} pears", id.first().get_name());
    });

    println!();

    // Get first target of relationship
    println!(
        "Bob eats {}",
        bob.get_target_from_component::<Eats>(0).get_name()
    );

    // Get second target of relationship
    println!(
        "Bob also eats {}",
        bob.get_target_from_component::<Eats>(1).get_name()
    );

    // Output:
    //  Bob eats apples? true
    //  Bob grows food? true

    //  Bob's type: [(Identifier,Name), (Eats,Apples), (Eats,Pears), (Grows,Pears)]

    //  Bob eats Apples
    //  Bob eats Pears

    //  Bob Eats pears
    //  Bob Grows pears

    //  Bob eats Apples
    //  Bob also eats Pears
}
