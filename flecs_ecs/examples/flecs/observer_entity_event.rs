use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
// Entity events are events that are emitted and observed for a specific entity.
// They are a thin wrapper around regular observers, which match against queries
// instead of single entities. While they work similarly under the hood, entity
// events provide a much simpler API.
//
// An entity event only needs two pieces of data:
// - The entity on which to emit the event
// - The event to emit

// An event without payload
#[derive(Component)]
struct Click;

// An event with payload
#[derive(Component)]
struct Resize {
    width: f32,
    height: f32,
}

#[derive(Debug, Copy, Clone)]
enum CloseReason {
    User,
    _System,
}

#[derive(Component)]
struct CloseRequested {
    reason: CloseReason,
}

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create an observer for the CloseRequested event to listen to any entity.
    world
        .observer::<CloseRequested, &flecs::Any>()
        .each_iter(|it, _index, _| {
            let reason = it.param().reason;
            fprintln!(it, "Close request with reason: {:?}", reason);
        });

    let widget = world.entity_named(c"MyWidget");
    fprintln!(&world, "widget: {:?}", widget);

    // Observe the Click event on the widget entity.
    widget.observe::<Click>(|| {
        println!("clicked!");
    });

    widget.observe_entity::<Click>(|entity| {
        fprintln!(entity, "clicked on {:?}", entity.name());
    });

    // Observe the Resize event on the widget entity.
    widget.observe_payload(|payload: &Resize| {
        println!(
            "widget resized to {{ {}, {} }}!",
            payload.width, payload.height
        );
    });

    widget.observe_payload_entity(|entity, payload: &Resize| {
        fprintln!(
            entity,
            "{} resized to {{ {}, {} }}!",
            entity.name(),
            payload.width,
            payload.height
        );
    });

    widget.emit(&Click);

    widget.emit(&Resize {
        width: 100.0,
        height: 200.0,
    });

    widget.emit(&CloseRequested {
        reason: CloseReason::User,
    });

    world.get::<&Snap>(|snap| 
        snap.test("observer_entity_event".to_string()));
    // Output:
    //  widget: Entity name: MyWidget -- id: 506 -- archetype: (Identifier,Name)
    //  clicked on "MyWidget"
    //  clicked!
    //  MyWidget resized to { 100, 200 }!
    //  widget resized to { 100, 200 }!
    //  Close request with reason: User
}
