include!("common");

// This example shows how to use rules for testing facts. A fact is a query that
// has no variable elements. Consider a regular ECS query like this:
//   Position, Velocity
//
// When written out in full, this query looks like:
//   Position($This), Velocity($This)
//
// "This" is a (builtin) query variable that is unknown before we evaluate the
// query. Therefore this query does not test a fact, we can't know which values
// This will assume.
//
// An example of a fact-checking query is:
//   IsA(Cat, Animal)
//
// This is a fact: the query has no elements that are unknown before evaluating
// the query. A rule that checks a fact does not return entities, but will
// instead return the reasons why a fact is true (if it is true).

#[derive(Component)]
struct Likes;

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    let bob = world.entity_named(c"Bob");
    let alice = world.entity_named(c"Alice");
    let john = world.entity_named(c"John");
    let jane = world.entity_named(c"Jane");

    bob.add_first::<Likes>(alice);
    alice.add_first::<Likes>(bob);
    john.add_first::<Likes>(jane);
    jane.add_first::<Likes>(john);
    bob.add_first::<Likes>(jane); // inserting a bit of drama

    // Create a rule that checks if two entities like each other. By itself this
    // rule is not a fact, but we can use it to check facts by populating both
    // of its variables.
    //
    // The equivalent query in the DSL is:
    //  Likes($X, $Y), Likes($Y, $X)
    //
    // Instead of using variables we could have created a rule that referred the
    // entities directly, but then we would have to create a rule for each
    // fact, vs reusing a single rule for multiple facts.

    let friends = world
        .query::<()>()
        .with_first_name::<&Likes>(c"$Y")
        .set_src_name(c"$X")
        .with_first_name::<&Likes>(c"$X")
        .set_src_name(c"$Y")
        .build();

    let x_var = friends.find_var(c"X").unwrap();
    let y_var = friends.find_var(c"Y").unwrap();

    // Check a few facts

    fprintln!(
        &world,
        "Are Bob and Alice friends? {}",
        if friends
            .iterable()
            .set_var(x_var, bob)
            .set_var(y_var, alice)
            .is_true()
        {
            "Yes"
        } else {
            "No"
        }
    );

    fprintln!(
        &world,
        "Are Bob and John friends? {}",
        if friends
            .iterable()
            .set_var(x_var, bob)
            .set_var(y_var, john)
            .is_true()
        {
            "Yes"
        } else {
            "No"
        }
    );

    fprintln!(
        &world,
        "Are Jane and John friends? {}",
        if friends
            .iterable()
            .set_var(x_var, jane)
            .set_var(y_var, john)
            .is_true()
        {
            "Yes"
        } else {
            "No"
        }
    );

    // It doesn't matter who we assign to X or Y. After the variables are
    // substituted, either yields a fact that is true.

    fprintln!(
        &world,
        "Are John and Jane friends? {}",
        if friends
            .iterable()
            .set_var(x_var, john)
            .set_var(y_var, jane)
            .is_true()
        {
            "Yes"
        } else {
            "No"
        }
    );

    Ok(Snap::from(&world))

    // Output:
    //  Are Bob and Alice friends? Yes
    //  Are Bob and John friends? No
    //  Are Jane and John friends? Yes
    //  Are John and Jane friends? Yes
}
