#![allow(dead_code)]
mod common;
use common::*;

// This example shows how relationships can be combined with components to attach
// data to a relationship.

// Some demo components:

#[derive(Clone, Component, Debug, Default)]
struct Requires {
    amount: f32,
}

#[derive(Clone, Component, Debug, Default)]
struct Gigawatts;

#[derive(Clone, Component, Debug, Default)]
struct Expires {
    timeout: f32,
}

#[derive(Clone, Component, Debug, Default)]
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

    // Currently there's no support in Rust flecs for both parts of a pair to be components with data, One must be empty.
    // in Flecs C++ it is supported, but one can only be set at a time.

    /* missing example here since a decision was made to not support this due to the unsafe nature of it
    of users possibly transmuting wrong data and crashing their code base. It's open for discussion to add this in.
    Currently it's a not common use case.

    due to this, the tag property isn't supported either. We're missing an essential
    rust feature to essentially safely support this feature */

    // The id::type_id method can be used to find the component type for a pair:
    println!(
        "{}",
        world
            .get_id_pair::<Requires, Gigawatts>()
            .type_id()
            .get_hierarchy_path()
            .unwrap()
    );
    println!(
        "{}",
        world
            .get_id_pair::<Gigawatts, Requires>()
            .type_id()
            .get_hierarchy_path()
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
    //  e1: requires: 1.21
    //  e1: requires: 1.5
    //  ::Requires
    //  ::Requires
    //  requires: 1.21 gigawatts
}
