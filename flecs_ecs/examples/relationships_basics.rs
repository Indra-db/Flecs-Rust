mod common;
pub use common::*;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

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
        .add_pair_first::<Eats>(apples)
        .add_pair_first::<Eats>(pears)
        // Pairs can also be constructed from two entity ids
        .add_id((grows, pears));

    // Has can be used with relationships as well
    fprintln!(
        snap,
        "Bob eats apples? {}",
        bob.has_pair_first::<Eats>(apples)
    );

    // Wildcards can be used to match relationships
    fprintln!(
        snap,
        "Bob grows food? {}, {}",
        bob.has_id((grows, flecs::Wildcard::ID)),
        //or you can do
        bob.has_pair_second::<flecs::Wildcard>(grows)
    );

    fprintln!(snap);

    // Print the type of the entity. Should output:
    //   (Identifier,Name),(Eats,Apples),(Eats,Pears),(Grows,Pears)
    fprintln!(snap, "Bob's type: [{}]", bob.archetype());

    fprintln!(snap);

    // Relationships can be iterated for an entity. This iterates (Eats, *):
    bob.for_each_target::<Eats>(|second| {
        fprintln!(snap, "Bob eats {}", second.name());
    });

    fprintln!(snap);

    // Iterate by explicitly providing the pair. This iterates (*, Pears):
    bob.for_each_matching_pair(flecs::Wildcard::ID, pears, |id| {
        fprintln!(snap, "Bob {} pears", id.first().name());
    });

    fprintln!(snap);

    // Get first target of relationship
    fprintln!(snap, "Bob eats {}", bob.target::<Eats>(0).name());

    // Get second target of relationship
    fprintln!(snap, "Bob also eats {}", bob.target::<Eats>(1).name());

    snap.test();

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
