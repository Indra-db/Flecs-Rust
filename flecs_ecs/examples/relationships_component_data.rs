#![allow(dead_code)]
mod common;
use common::*;

// This example shows how relationships can be combined with components to attach
// data to a relationship.

// Some demo components:

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
        .new_entity()
        .set_pair_first::<_, Gigawatts>(Requires { amount: 1.21 });

    let require = e1.get_pair_first::<Requires, Gigawatts>();

    if let Some(r) = require {
        println!("e1: requires: {}", r.amount);
    } else {
        println!("e1: does not have a relationship with Requires, Gigawatts");
    }

    // The component can be either the first or second part of a pair:
    let e2 = world
        .new_entity()
        .set_pair_second::<Gigawatts, Requires>(Requires { amount: 1.5 });

    let require = e2.get_pair_second::<Gigawatts, Requires>();

    if let Some(r) = require {
        println!("e1: requires: {}", r.amount);
    } else {
        println!("e1: does not have a relationship with Gigawatts, Requires");
    }

    // Note that <Requires, Gigawatts> and <Gigawatts, Requires> are two
    // different pairs, and can be added to an entity at the same time.

    // If both parts of a pair are components, the pair assumes the type of
    // the first element:
    let e3 = world
        .new_entity()
        .set_pair_first::<Expires, Position>(Expires { timeout: 0.5 });

    let expires = e3.get_pair_first::<Expires, Position>();
    println!("expires: {}", expires.unwrap().timeout);

    // You can prevent a pair from assuming the type of a component by adding
    // the Tag property to a relationship:
    world.component::<MustHave>().add_id(ECS_TAG);

    // Even though Position is a component, <MustHave, Position> contains no
    // data because MustHave has the Tag property.
    world.new_entity().add::<(MustHave, Position)>();

    // The id::type_id method can be used to find the component type for a pair:
    println!(
        "{}",
        world
            .get_id::<(Requires, Gigawatts)>()
            .type_id()
            .get_path()
            .unwrap()
    );
    println!(
        "{}",
        world
            .get_id::<(Gigawatts, Requires)>()
            .type_id()
            .get_path()
            .unwrap()
    );
    println!(
        "{}",
        world
            .get_id::<(Expires, Position)>()
            .type_id()
            .get_path()
            .unwrap()
    );
    println!(
        "{}",
        world
            .get_id::<(MustHave, Position)>()
            .type_id()
            .get_path()
            .unwrap()
    );

    // When querying for a relationship component, add the pair type as template
    // argument to the builder:
    let query = world
        .query_builder::<(&Requires,)>()
        .term_at(1)
        .select_second::<Gigawatts>()
        .build();

    query.each(|(requires,)| {
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
