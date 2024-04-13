mod common;
use common::*;

#[derive(Component)]
struct Healthy;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    let apples = world.new_entity_named(c"Apples").add::<Healthy>();
    let salad = world.new_entity_named(c"Salad").add::<Healthy>();
    let burgers = world.new_entity_named(c"Burgers");
    let pizza = world.new_entity_named(c"Pizza");
    let chocolate = world.new_entity_named(c"Chocolate");

    world
        .new_entity_named(c"Bob")
        .add_pair_first::<Eats>(apples)
        .add_pair_first::<Eats>(burgers)
        .add_pair_first::<Eats>(pizza);

    world
        .new_entity_named(c"Alice")
        .add_pair_first::<Eats>(salad)
        .add_pair_first::<Eats>(chocolate)
        .add_pair_first::<Eats>(apples);

    // Here we're creating a rule that in the query DSL would look like this:
    //   Eats($This, $Food), Healthy($Food)
    //
    // Rules are similar to queries, but support more advanced features. This
    // example shows how the basics of how to use rules & variables.

    let rule = world
        .rule::<()>()
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
        .with_pair_name::<Eats>(c"$Food")
        .with_type::<&Healthy>()
        .select_src_name(c"$Food")
        .build();

    // Lookup the index of the variable. This will let us quickly lookup its
    // value while we're iterating.
    let food_var = rule.find_var(c"Food");

    // Iterate the rule
    rule.each_iter(|it, index, ()| {
        fprintln!(
            snap,
            "{} eats {}",
            it.entity(index).name(),
            it.get_var(food_var).name()
        );
    });

    // In CPP Rules need to be explicitly deleted.
    // with `rule.destruct()` however in Rust it is automatically dropped when out of scope
    // but you can still drop it manually if you want to
    rule.destruct();

    snap.test();

    // Output:
    // Bob eats Apples
    // Alice eats Apples
    // Alice eats Salad
}
