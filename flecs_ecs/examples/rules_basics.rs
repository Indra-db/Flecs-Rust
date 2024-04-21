mod common;
use common::*;

#[derive(Component)]
struct Healthy;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    let apples = world.entity_named(c"Apples").add::<Healthy>();
    let salad = world.entity_named(c"Salad").add::<Healthy>();
    let burgers = world.entity_named(c"Burgers");
    let pizza = world.entity_named(c"Pizza");
    let chocolate = world.entity_named(c"Chocolate");

    world
        .entity_named(c"Bob")
        .add_first::<Eats>(apples)
        .add_first::<Eats>(burgers)
        .add_first::<Eats>(pizza);

    world
        .entity_named(c"Alice")
        .add_first::<Eats>(salad)
        .add_first::<Eats>(chocolate)
        .add_first::<Eats>(apples);

    // Here we're creating a rule that in the query DSL would look like this:
    //   Eats($This, $Food), Healthy($Food)
    //
    // Rules are similar to queries, but support more advanced features. This
    // example shows how the basics of how to use rules & variables.

    let rule = world
        .query::<()>()
        // Identifiers that start with _ are query variables. Query variables
        // are like wildcards, but enforce that the entity substituted by the
        // wildcard is the same across terms.
        //
        // For example, in this query it is not guaranteed that the entity
        // substituted by the * for Eats is the same as for Healthy:
        //   (Eats, *), Healthy(*)
        //
        // By replacing * with _Food, both terms are constrained to use the
        // same entity.
        .with_first_name::<&Eats>(c"$food")
        .with::<&Healthy>()
        .set_src_name(c"$food")
        .build();

    // Lookup the index of the variable. This will let us quickly lookup its
    // value while we're iterating.
    let food_var = rule.find_var(c"food");

    // Iterate the rule
    rule.each_iter(|it, index, ()| {
        fprintln!(
            snap,
            "{} eats {}",
            it.entity(index).name(),
            it.get_var(food_var.unwrap()).name()
        );
    });

    snap.test();

    // Output:
    // Bob eats Apples
    // Alice eats Apples
    // Alice eats Salad
}
