include!("common");

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

#[derive(Debug)]
enum CloseReason {
    User,
    _System,
}

#[derive(Component)]
struct CloseRequested {
    reason: CloseReason,
}

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create an observer for the CloseRequested event to listen to any entity.
    world
        .observer::<()>()
        .add_event::<CloseRequested>()
        .with::<&flecs::Any>()
        .each_iter(|it, _index, _| {
            let close_requested = unsafe { it.param::<CloseRequested>() };
            fprintln!(
                snap,
                "Close request with reason: {:?}",
                close_requested.reason
            );
        });

    let widget = world.entity_named(c"MyWidget");
    fprintln!(snap, "widget: {:?}", widget);

    // Observe the Click event on the widget entity.
    widget.observe::<Click>(|| {
        fprintln!(snap, "clicked!");
    });

    widget.observe_entity::<Click>(|entity| {
        fprintln!(snap, "clicked on {:?}", entity.name());
    });

    // Observe the Resize event on the widget entity.
    widget.observe_payload(|payload: &mut Resize| {
        fprintln!(
            snap,
            "widget resized to {{ {}, {} }}!",
            payload.width,
            payload.height
        );
    });

    widget.observe_payload_entity(|entity, payload: &mut Resize| {
        fprintln!(
            snap,
            "{} resized to {{ {}, {} }}!",
            entity.name(),
            payload.width,
            payload.height
        );
    });

    widget.emit::<Click>();

    widget.emit_payload(Resize {
        width: 100.0,
        height: 200.0,
    });

    widget.emit_payload(CloseRequested {
        reason: CloseReason::User,
    });

    Ok(snap)

    // Output:
    //  widget: Entity name: MyWidget -- id: 506 -- archetype: (Identifier,Name)
    //  clicked on "MyWidget"
    //  clicked!
    //  MyWidget resized to { 100, 200 }!
    //  widget resized to { 100, 200 }!
    //  Close request with reason: User
}
