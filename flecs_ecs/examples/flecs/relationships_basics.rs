use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct Eats;

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Entity used for Grows relationship
    let grows = world.entity_named(c"Grows");

    // Relationship objects
    let apples = world.entity_named(c"Apples");
    let pears = world.entity_named(c"Pears");

    // Create an entity with 3 relationships. Relationships are like regular components,
    // but combine two types/identifiers into an (relationship, object) pair.
    let bob = world
        .entity_named(c"Bob")
        // Pairs can be constructed from a type and entity
        .add_first::<Eats>(apples)
        .add_first::<Eats>(pears)
        // Pairs can also be constructed from two entity ids
        .add_id((grows, pears));

    // Has can be used with relationships as well
    fprintln!(&world, "Bob eats apples? {}", bob.has_first::<Eats>(apples));

    // Wildcards can be used to match relationships
    fprintln!(
        &world,
        "Bob grows food? {}, {}",
        bob.has_id((grows, flecs::Wildcard::ID)),
        //or you can do
        bob.has_second::<flecs::Wildcard>(grows)
    );

    fprintln!(&world);

    // Print the type of the entity. Should output:
    //   (Identifier,Name),(Eats,Apples),(Eats,Pears),(Grows,Pears)
    fprintln!(&world, "Bob's type: [{}]", bob.archetype());

    fprintln!(&world);

    // Relationships can be iterated for an entity. This iterates (Eats, *):
    bob.each_target::<Eats>(|second| {
        fprintln!(&world, "Bob eats {}", second.name());
    });

    fprintln!(&world);

    // Iterate by explicitly providing the pair. This iterates (*, Pears):
    bob.each_pair(flecs::Wildcard::ID, pears, |id| {
        fprintln!(&world, "Bob {} pears", id.first_id().name());
    });

    fprintln!(&world);

    // Get first target of relationship
    fprintln!(&world, "Bob eats {}", bob.target::<Eats>(0).name());

    // Get second target of relationship
    fprintln!(&world, "Bob also eats {}", bob.target::<Eats>(1).name());

    world.get::<&Snap>(|snap| snap.test("relationships_basics".to_string()));

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
