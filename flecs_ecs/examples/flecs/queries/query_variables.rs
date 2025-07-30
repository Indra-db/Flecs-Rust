use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct Eats;

#[derive(Component)]
struct Healthy;

fn main() {
    let world = World::new();

    let apples = world.entity_named("Apples").add(Healthy);
    let salad = world.entity_named("Salad").add(Healthy);
    let burgers = world.entity_named("Burgers");
    let pizza = world.entity_named("Pizza");
    let chocolate = world.entity_named("Chocolate");

    world
        .entity_named("Bob")
        .add((Eats, apples))
        .add((Eats, burgers))
        .add((Eats, pizza));

    world
        .entity_named("Alice")
        .add((Eats, salad))
        .add((Eats, chocolate))
        .add((Eats, apples));

    // Here we're creating a rule that in the query DSL would look like this:
    //   Eats($This, $Food), Healthy($Food)
    //
    // example shows how the basics of how to use queries & variables.

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
        .with((Eats, "$food"))
        .with(&Healthy)
        .set_src("$food")
        .build();

    // Lookup the index of the variable. This will let us quickly lookup its
    // value while we're iterating.
    let food_var = rule.find_var("food");

    // Iterate the rule
    rule.run(|mut it| {
        while it.next() {
            for i in it.iter() {
                let food = it.get_var(food_var.unwrap());
                let entity = it.entity(i).unwrap();
                println!("{entity} eats {food}");
            }
        }
    });

    // Output:
    // Bob eats Apples
    // Alice eats Apples
    // Alice eats Salad
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_variables".to_string());
}
