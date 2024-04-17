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
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // When one element of a pair is a component and the other element is a tag,
    // the pair assumes the type of the component.
    let e1 = world
        .entity()
        .set_pair_first::<_, Gigawatts>(Requires { amount: 1.21 });

    let require = e1.try_get_pair_first::<Requires, Gigawatts>();

    if let Some(r) = require {
        fprintln!(snap, "e1: requires: {}", r.amount);
    } else {
        fprintln!(
            snap,
            "e1: does not have a relationship with Requires, Gigawatts"
        );
    }

    // The component can be either the first or second part of a pair:
    let e2 = world
        .entity()
        .set_pair_second::<Gigawatts, Requires>(Requires { amount: 1.5 });

    let require = e2.try_get_pair_second::<Gigawatts, Requires>();

    if let Some(r) = require {
        fprintln!(snap, "e1: requires: {}", r.amount);
    } else {
        fprintln!(
            snap,
            "e1: does not have a relationship with Gigawatts, Requires"
        );
    }

    // Note that <Requires, Gigawatts> and <Gigawatts, Requires> are two
    // different pairs, and can be added to an entity at the same time.

    // If both parts of a pair are components, the pair assumes the type of
    // the first element:
    let e3 = world
        .entity()
        .set_pair_first::<Expires, Position>(Expires { timeout: 0.5 });

    let expires = e3.try_get_pair_first::<Expires, Position>();
    fprintln!(snap, "expires: {}", expires.unwrap().timeout);

    // You can prevent a pair from assuming the type of a component by adding
    // the Tag property to a relationship:
    world.component::<MustHave>().add::<Tag>();

    // Even though Position is a component, <MustHave, Position> contains no
    // data because MustHave has the Tag property.
    world.entity().add::<(MustHave, Position)>();

    // The id::type_id method can be used to find the component type for a pair:
    fprintln!(
        snap,
        "{}",
        world
            .id_pair::<Requires, Gigawatts>()
            .type_id()
            .path()
            .unwrap()
    );
    fprintln!(
        snap,
        "{}",
        world
            .id_pair::<Gigawatts, Requires>()
            .type_id()
            .path()
            .unwrap()
    );
    fprintln!(
        snap,
        "{}",
        world
            .id_pair::<Expires, Position>()
            .type_id()
            .path()
            .unwrap()
    );
    fprintln!(
        snap,
        "{}",
        world
            .id_pair::<MustHave, Position>()
            .type_id()
            .path()
            .unwrap()
    );

    // When querying for a relationship component, add the pair type as template
    // argument to the builder:
    let query = world
        .query::<&Requires>()
        .term_at(0)
        .select_second::<Gigawatts>()
        .build();

    query.each(|requires| {
        fprintln!(snap, "requires: {} gigawatts", requires.amount);
    });

    snap.test();

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
