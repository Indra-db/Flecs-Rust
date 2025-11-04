use crate::z_ignore_test_common::*;
use flecs_ecs::prelude::*;

// not meant to make sense on why these attributes are added
// just for demonstration purposes
#[derive(Debug, Component)]
#[flecs(
    name = "Position",
    meta,
    // component traits
    traits((With, WorldPosition)),
    // adding components
    add((Eats, Apples)),
    // setting components
    set((Likes { score: 10 }, Bob)),
    // hooks
    hooks(
        on_add(|e, _| { println!("added Position to {}", e.name()); }),
        on_set(|e, p: &mut Position| { println!("set {:?} for {}", p, e.name()); }),
        on_remove(|e, p: &mut Position| { println!("removed {:?} from {}", p, e.name()); })
    )
)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default)]
pub struct WorldPosition;

#[derive(Component)]
pub struct Eats;

#[derive(Component)]
pub struct Apples;

#[derive(Debug, Component)]
pub struct Likes {
    pub score: i32,
}

#[derive(Component)]
pub struct Bob;

#[test]
fn main() {
    let world = World::new();

    let e = world.entity_named("Bob");
    e.set(Position { x: 1.0, y: 2.0 });
    e.remove(Position::id());
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("component_attributes".to_string());
}
