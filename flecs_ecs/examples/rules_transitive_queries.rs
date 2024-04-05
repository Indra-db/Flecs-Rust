mod common;
use common::*;

// Transitive relationships make it possible to tell the ECS that if an entity
// has a relationship (R, X) and X has relationship (R, Y), the entity should be
// treated as if it also has (R, Y). In practice this is useful for expressing
// things like:
//
// Bob lives in SanFrancisco
// San Francisco is located in the United States
// Therefore Bob also lives in the United States.
//
// An example of transitivity can be seen in the component_inheritance example.
// This example uses the builtin IsA relationship, which is transitive. This
// example shows how to achieve similar behavior with a user-defined relationship.

#[derive(Component)]
struct LocatedIn;

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Continent;

#[derive(Component)]
struct Country;

#[derive(Component)]
struct State;

#[derive(Component)]
struct City;

#[derive(Component)]
struct Person;

fn main() {
    let world = World::new();

    // Register the LocatedIn relationship as transitive
    world.component::<LocatedIn>().add_id(ECS_TRANSITIVE);

    // Populate the store with locations
    let earth = world.new_entity_named(c"Earth").add::<Planet>();

    // Continents
    let north_america = world
        .new_entity_named(c"NorthAmerica")
        .add::<Continent>()
        .add_pair_first::<LocatedIn>(earth);

    let europe = world
        .new_entity_named(c"Europe")
        .add::<Continent>()
        .add_pair_first::<LocatedIn>(earth);

    // Countries
    let united_states = world
        .new_entity_named(c"UnitedStates")
        .add::<Country>()
        .add_pair_first::<LocatedIn>(north_america);

    let netherlands = world
        .new_entity_named(c"Netherlands")
        .add::<Country>()
        .add_pair_first::<LocatedIn>(europe);

    // States
    let california = world
        .new_entity_named(c"California")
        .add::<State>()
        .add_pair_first::<LocatedIn>(united_states);

    let washington = world
        .new_entity_named(c"Washington")
        .add::<State>()
        .add_pair_first::<LocatedIn>(united_states);

    let noord_holland = world
        .new_entity_named(c"NoordHolland")
        .add::<State>()
        .add_pair_first::<LocatedIn>(netherlands);

    // Cities
    let san_francisco = world
        .new_entity_named(c"SanFrancisco")
        .add::<City>()
        .add_pair_first::<LocatedIn>(california);

    let seattle = world
        .new_entity_named(c"Seattle")
        .add::<City>()
        .add_pair_first::<LocatedIn>(washington);

    let amsterdam = world
        .new_entity_named(c"Amsterdam")
        .add::<City>()
        .add_pair_first::<LocatedIn>(noord_holland);

    // Inhabitants
    world
        .new_entity_named(c"Bob")
        .add::<Person>()
        .add_pair_first::<LocatedIn>(san_francisco);

    world
        .new_entity_named(c"Alice")
        .add::<Person>()
        .add_pair_first::<LocatedIn>(seattle);

    world
        .new_entity_named(c"Job")
        .add::<Person>()
        .add_pair_first::<LocatedIn>(amsterdam);

    // Create a query that finds the countries persons live in. Note that these
    // have not been explicitly added to the Person entities, but because the
    // LocatedIn is transitive, the rule engine will traverse the relationship
    // until it found something that is a country.
    //
    // The equivalent of this query in the DSL is:
    //   Person, (LocatedIn, $Location), Country($Location)

    let rule = world
        .rule_builder::<()>()
        .with_type::<&Person>()
        .with_pair_name::<LocatedIn>(c"$Location")
        .with_type::<&Country>()
        .select_src_name(c"$Location")
        .build();

    // Lookup the index of the variable. This will let us quickly lookup its
    // value while we're iterating.
    let location_var = rule.find_var(c"Location");

    // Iterate the rule
    rule.iterable().each_iter(|it, index, _| {
        println!(
            "{} lives in {}",
            it.get_entity(index),
            it.get_var(location_var)
        );
    });

    // Output:
    //  Bob lives in UnitedStates
    //  Alice lives in UnitedStates
    //  Job lives in Netherlands
}
