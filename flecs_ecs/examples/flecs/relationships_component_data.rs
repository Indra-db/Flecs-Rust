use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

// This example shows how relationships can be combined with components to attach
// data to a relationship.

// Some demo components:

#[derive(Debug, Component)]
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

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // When one element of a pair is a component and the other element is a tag,
    // the pair assumes the type of the component.
    let e1 = world
        .entity()
        .set_first::<_, Gigawatts>(Requires { amount: 1.21 });

    let require = e1.try_get_first::<Requires, Gigawatts>();

    if let Some(r) = require {
        fprintln!(&world, "e1: requires: {}", r.amount);
    } else {
        fprintln!(
            &world,
            "e1: does not have a relationship with Requires, Gigawatts"
        );
    }

    // The component can be either the first or second part of a pair:
    let e2 = world
        .entity()
        .set_second::<Gigawatts, Requires>(Requires { amount: 1.5 });

    let require = e2.try_get_second::<Gigawatts, Requires>();

    if let Some(r) = require {
        fprintln!(&world, "e1: requires: {}", r.amount);
    } else {
        fprintln!(
            &world,
            "e1: does not have a relationship with Gigawatts, Requires"
        );
    }

    // Note that <Requires, Gigawatts> and <Gigawatts, Requires> are two
    // different pairs, and can be added to an entity at the same time.

    // If both parts of a pair are components, the pair assumes the type of
    // the first element:
    let e3 = world
        .entity()
        .set_first::<Expires, Position>(Expires { timeout: 0.5 });

    let expires = e3.get_first::<Expires, Position>();
    fprintln!(&world, "expires: {}", expires.timeout);

    // You can prevent a pair from assuming the type of a component by adding
    // the Tag property to a relationship:
    world.component::<MustHave>().add::<Tag>();

    // Even though Position is a component, <MustHave, Position> contains no
    // data because MustHave has the Tag property.
    world.entity().add::<(MustHave, Position)>();

    // The id::type_id method can be used to find the component type for a pair:
    fprintln!(
        &world,
        "{}",
        world
            .id_from::<(Requires, Gigawatts)>()
            .type_id()
            .path()
            .unwrap()
    );
    fprintln!(
        &world,
        "{}",
        world
            .id_from::<(Gigawatts, Requires)>()
            .type_id()
            .path()
            .unwrap()
    );
    fprintln!(
        &world,
        "{}",
        world
            .id_from::<(Expires, Position)>()
            .type_id()
            .path()
            .unwrap()
    );
    fprintln!(
        &world,
        "{}",
        world
            .id_from::<(MustHave, Position)>()
            .type_id()
            .path()
            .unwrap()
    );

    // When querying for a relationship component, add the pair type as template
    // argument to the builder:
    let query = world
        .query::<&Requires>()
        .term_at(0)
        .set_second::<Gigawatts>()
        .build();

    query.each_entity(|entity, requires| {
        fprintln!(entity, "requires: {} gigawatts", requires.amount);
    });

    world
        .get::<Snap>()
        .test("relationships_component_data".to_string());

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
