use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
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

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    //todo v4 broken example bug flecs core
    /*

    // Register the LocatedIn relationship as transitive
    world.component::<LocatedIn>().add::<flecs::Transitive>();

    // Populate the store with locations
    let earth = world.entity_named(c"Earth").add::<Planet>();

    // Continents
    let north_america = world
        .entity_named(c"NorthAmerica")
        .add::<Continent>()
        .add_first::<LocatedIn>(earth);

    let europe = world
        .entity_named(c"Europe")
        .add::<Continent>()
        .add_first::<LocatedIn>(earth);

    // Countries
    let united_states = world
        .entity_named(c"UnitedStates")
        .add::<Country>()
        .add_first::<LocatedIn>(north_america);

    let netherlands = world
        .entity_named(c"Netherlands")
        .add::<Country>()
        .add_first::<LocatedIn>(europe);

    // States
    let california = world
        .entity_named(c"California")
        .add::<State>()
        .add_first::<LocatedIn>(united_states);

    let washington = world
        .entity_named(c"Washington")
        .add::<State>()
        .add_first::<LocatedIn>(united_states);

    let noord_holland = world
        .entity_named(c"NoordHolland")
        .add::<State>()
        .add_first::<LocatedIn>(netherlands);

    // Cities
    let san_francisco = world
        .entity_named(c"SanFrancisco")
        .add::<City>()
        .add_first::<LocatedIn>(california);

    let seattle = world
        .entity_named(c"Seattle")
        .add::<City>()
        .add_first::<LocatedIn>(washington);

    let amsterdam = world
        .entity_named(c"Amsterdam")
        .add::<City>()
        .add_first::<LocatedIn>(noord_holland);

    // Inhabitants
    world
        .entity_named(c"Bob")
        .add::<Person>()
        .add_first::<LocatedIn>(san_francisco);

    world
        .entity_named(c"Alice")
        .add::<Person>()
        .add_first::<LocatedIn>(seattle);

    world
        .entity_named(c"Job")
        .add::<Person>()
        .add_first::<LocatedIn>(amsterdam);

    // Create a query that finds the countries persons live in. Note that these
    // have not been explicitly added to the Person entities, but because the
    // LocatedIn is transitive, the rule engine will traverse the relationship
    // until it found something that is a country.
    //
    // The equivalent of this query in the DSL is:
    //   Person, (LocatedIn, $Location), Country($Location)

    let rule = world
        .query::<()>()
        .with::<&Person>()
        .with_first_name::<&LocatedIn>(c"$Location")
        .with::<&Country>()
        .set_src_name(c"$Location")
        .build();

    // Lookup the index of the variable. This will let us quickly lookup its
    // value while we're iterating.
    let location_var = rule.find_var(c"Location").unwrap();

    // Iterate the rule
    rule.iterable().each_iter(|it, index, _| {
        fprintln!(
            snap,
            "{} lives in {}",
            it.entity(index),
            it.get_var(location_var)
        );
    });

    world.get::<&Snap>(|snap| snap.test("rules_transitive_queries".to_string()));
    */

    // Output:
    //  Bob lives in UnitedStates
    //  Alice lives in UnitedStates
    //  Job lives in Netherlands
}
