use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

// The on_validate hook is invoked when a component is set, before the on_set
// hook and OnSet observers are invoked. When the hook returns false, the
// on_set hook and OnSet observers are not invoked for the entity. This can be
// used to guard application logic from invalid component values.

#[derive(Debug, Component)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

fn main() {
    let world = World::new();

    world
        .component::<Position>()
        // Returns false when the position is outside of the world bounds.
        .on_validate(|_entity, p| {
            p.x >= 0.0 && p.x <= 100.0 && p.y >= 0.0 && p.y <= 100.0
        })
        .on_set(|e, p| {
            println!("{} set to {{{}, {}}}", e.name(), p.x, p.y);
        });

    let e = world.entity_named("Entity");

    // Position is inside world bounds, on_set is invoked
    e.set(Position { x: 50.0, y: 50.0 });

    // Position is outside world bounds, on_set is not invoked
    e.set(Position { x: 150.0, y: 50.0 });

    // Output:
    //  Entity set to {50, 50}
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("entity_on_validate".to_string());
}
