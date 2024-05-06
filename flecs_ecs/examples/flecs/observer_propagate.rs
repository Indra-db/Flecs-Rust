use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// Events are propagated along relationship edges. This means that observers can
// listen for events from a parent or prefab, like triggering when a component
// inherited from a prefab was set.
//
// Event propagation happens automatically when an observer contains a filter
// with the EcsUp flag set (indicating upwards traversal). Observers use the
// same matching logic as queries: if a query with upwards traversal matches an
// entity, so will an observer.
//
// Events are only propagated along traversable relationship edges.

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create observer that listens for events from both self and parent
    world
        .observer::<flecs::OnSet, (&Position, &Position)>()
        .term_at(1)
        .parent()
        .each_iter(|it, index, (pos_self, pos_parent)| {
            fprintln!(
                it,
                " - {}: {}: {}: self: {{ {}, {} }}, parent: {{ {}, {} }}",
                it.event().name(),
                it.event_id().to_str(),
                it.entity(index).name(),
                pos_self.x,
                pos_self.y,
                pos_parent.x,
                pos_parent.y
            );
        });

    // Create entity and parent
    let parent = world.entity_named(c"p");
    let entity = world.entity_named(c"e").child_of_id(parent);

    // Set Position on entity. This doesn't trigger the observer yet, since the
    // parent doesn't have Position yet.
    entity.set(Position { x: 10.0, y: 20.0 });

    // Set Position on parent. This event will be propagated and trigger the
    // observer, as the observer query now matches.
    parent.set(Position { x: 1.0, y: 2.0 });

    world.get::<Snap>().test("observer_propagate".to_string());

    // Output:
    //  - OnSet: Position: e: self: { 10, 20 }, parent: { 1, 2 }
}
