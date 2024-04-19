mod common;
use common::*;

// This example shows how a rule may have terms with cyclic dependencies on
// variables.

#[derive(Component)]
struct Likes;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    let bob = world.entity_named(c"Bob");
    let alice = world.entity_named(c"Alice");
    let john = world.entity_named(c"John");
    let jane = world.entity_named(c"Jane");

    bob.add_first::<Likes>(alice);
    alice.add_first::<Likes>(bob);
    john.add_first::<Likes>(jane);
    jane.add_first::<Likes>(john);
    bob.add_first::<Likes>(jane); // inserting a bit of drama

    // The following rule will only return entities that have a cyclic Likes
    // relationship- that is they must both like each other.
    //
    // The equivalent query in the DSL is:
    //   Likes($X, $Y), Likes($Y, $X)
    //
    // This is also an example of a query where all sources are variables. By
    // default queries use the builtin "This" variable as subject, which is what
    // populates the entities array in the query result (accessed by the
    // iter::entity function).
    //
    // Because this query does not use This at all, the entities array will not
    // be populated, and it.count() will always be 0.

    let rule = world
        .query::<()>()
        .with_first_name::<&Likes>(c"$Y")
        .select_src_name(c"$X")
        .with_first_name::<&Likes>(c"$X")
        .select_src_name(c"$Y")
        .build();

    // Lookup the index of the variables. This will let us quickly lookup their
    // values while we're iterating.
    let x_var = rule.find_var(c"X").unwrap();
    let y_var = rule.find_var(c"Y").unwrap();

    // Because the query doesn't use the This variable we cannot use "each"
    // which iterates the entities array. Instead we can use iter like this:
    rule.iter_only(|it| {
        let x = it.get_var(x_var);
        let y = it.get_var(y_var);
        fprintln!(snap, "{} likes {}", x.name(), y.name());
    });

    snap.test();

    // Output:
    //  Alice likes Bob
    //  John likes Jane
    //  Jane likes John
    //  Bob likes Alice

    // Note that the rule returns each pair twice. The reason for this is that
    // the goal of the rule engine is to return all "facts" that are true
    // within the given constraints. Since we did not give it any constraints
    // that would favor a person being matched by X or Y, the rule engine
    // returns both.
}
