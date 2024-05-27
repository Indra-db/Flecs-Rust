use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// This example shows how relationships can be combined with components to attach
// data to a relationship.

// Some demo components:

#[derive(Debug, Component, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Tag;

#[derive(Component)]
struct Requires {
    amount: f32,
}

#[derive(Component)]
struct Gigawatts;

#[derive(Component)]
struct Expires {
    timeout: f32,
}

#[derive(Component)]
struct MustHave;

fn main() {
    let world = World::new();

    // When one element of a pair is a component and the other element is a tag,
    // the pair assumes the type of the component.
    let e1 = world
        .entity()
        .set_pair::<Requires, Gigawatts>(Requires { amount: 1.21 });

    let require = e1.try_get::<Option<&(Requires, Gigawatts)>>(|req| {
        if let Some((req)) = req {
            println!("e1: requires: {}", req.amount);
        } else {
            println!("e1: does not have a relationship with Requires, Gigawatts");
        }
    });

    // The component can be either the first or second part of a pair:
    let e2 = world
        .entity()
        .set_pair::<Gigawatts, Requires>(Requires { amount: 1.5 });

    let require = e2.try_get::<Option<&(Gigawatts, Requires)>>(|req| {
        if let Some((req)) = req {
            println!("e2: requires: {}", req.amount);
        } else {
            println!("e2: does not have a relationship with Gigawatts, Requires");
        }
    });

    // Note that <Requires, Gigawatts> and <Gigawatts, Requires> are two
    // different pairs, and can be added to an entity at the same time.

    // If both parts of a pair are components, the pair assumes the type of
    // the first element:
    let e3 = world
        .entity()
        .set_pair::<Expires, Position>(Expires { timeout: 0.5 });

    let expires = e3.try_get::<&(Expires, Position)>(|expires| {
        println!("expires: {}", expires.timeout);
    });

    // You can prevent a pair from assuming the type of a component by adding
    // the Tag property to a relationship:
    world
        .component::<MustHave>()
        .add_trait::<flecs::PairIsTag>();

    // Even though Position is a component, <MustHave, Position> contains no
    // data because MustHave has the Tag property.
    world.entity().add::<(MustHave, Position)>(); // due to a Rust limitation, Position requires Default to add this sort of relationship.

    println!(
        "{}",
        world
            .id_from::<(Requires, Gigawatts)>()
            .type_id()
            .path()
            .unwrap()
    );
    println!(
        "{}",
        world
            .id_from::<(Gigawatts, Requires)>()
            .type_id()
            .path()
            .unwrap()
    );
    println!(
        "{}",
        world
            .id_from::<(Expires, Position)>()
            .type_id()
            .path()
            .unwrap()
    );
    println!(
        "{}",
        world
            .id_from::<(MustHave, Position)>()
            .type_id()
            .path()
            .unwrap()
    );

    // When querying for a relationship component, add the pair type as template
    // argument to the builder:
    let query = world.query::<&(Requires, Gigawatts)>().build();

    query.each_entity(|entity, requires| {
        println!("requires: {} gigawatts", requires.amount);
    });

    // Output:
    // e1: requires: 1.21
    // e1: requires: 1.5
    // expires: 0.5
    // ::Requires
    // ::Requires
    // ::Expires
    // 0
    // requires: 1.21 gigawatts
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("relationships_component_data".to_string());
}
