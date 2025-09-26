//! Tests from observersmanual.md

#![allow(unused_imports, unused_variables, dead_code, non_snake_case, path_statements, unreachable_code, unused_mut)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::common_test::*;

#[test]
fn observers_example_01() {
    let world = World::new();
    // Create observer that is invoked whenever Position is set
    world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(|e, p| {
            println!("Position set: {{ {}, {} }}", p.x, p.y);
        });

    world.entity().set(Position { x: 10.0, y: 20.0 }); // Invokes observer
}

#[test]
fn observers_the_basics_onadd_events_02() {
    let world = World::new();
    let e = world.entity();

    // OnAdd observer fires
    e.add(Position::id());

    // OnAdd observer doesn't fire, entity already has component
    e.add(Position::id());
}

#[test]
fn observers_the_basics_onset_events_03() {
    let world = World::new();
    let e = world.entity();

    // OnAdd observer fires first, then OnSet observer fires
    e.set(Position { x: 10.0, y: 20.0 });

    // OnAdd observer doesn't fire, OnSet observer fires
    e.set(Position { x: 10.0, y: 20.0 });
}

#[test]
fn observers_the_basics_onset_events_adding_an_isa_pair_04() {
    let world = World::new();
    let p = world.prefab().set(Position { x: 10.0, y: 20.0 });

    // Produces OnSet event for Position
    let i = world.entity().is_a(p);
}

#[test]
fn observers_the_basics_onset_events_removing_an_override_05() {
    let world = World::new();
    let p = world.prefab().set(Position { x: 10.0, y: 20.0 });

    // Produces OnSet event for inherited Position component
    let i = world.entity().is_a(p);

    // Override component. Produces regular OnSet event.
    i.set(Position { x: 20.0, y: 30.0 });

    // Reexposes inherited component, produces OnSet event
    i.remove(Position::id());
}

#[test]
fn observers_the_basics_onset_events_setting_an_inherited_component_06() {
    let world = World::new();
    let p = world.prefab().set(Position { x: 10.0, y: 20.0 });

    // Produces OnSet event for Position
    let i = world.entity().is_a(p);
}

#[test]
fn observers_the_basics_onremove_events_07() {
    let world = World::new();
    let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    // OnRemove observer fires
    e.remove(Position::id());

    // OnRemove observer doesn't fire, entity doesn't have the component
    e.remove(Position::id());
}

#[test]
fn observers_multi_event_observers_08() {
    let world = World::new();
    // Observer that listens for both OnAdd and OnRemove events
    world
        .observer::<flecs::OnAdd, ()>()
        .with(Position::id())
        .add_event(flecs::OnRemove::id())
        .each_entity(|e, _| {
    // ...
    });
}

#[test]
fn observers_multi_event_observers_09() {
    let world = World::new();
    world
        .observer::<flecs::OnAdd, ()>()
        .with(Position::id())
        .add_event(flecs::OnRemove::id())
        .each_iter(|it, i, _| {
            if it.event() == flecs::OnAdd::ID {
            // ...
            } else if it.event() == flecs::OnRemove::ID {
            // ...
            }
        });
}

#[test]
fn observers_multi_event_observers_10() {
    let world = World::new();
    // Observer that listens for all events for Position
    world
        .observer::<flecs::Wildcard, &Position>()
        .each_entity(|e, p| {
            // ...
        });
}

#[test]
fn observers_multi_term_observers_11() {
    let world = World::new();
    // Observer that listens for entities with both Position and Velocity
    world
        .observer::<flecs::OnAdd, ()>()
        .with(Position::id())
        .with(Velocity::id())
        .each_entity(|e, _| {
            // ...
        });
}

#[test]
fn observers_multi_term_observers_12() {
    let world = World::new();
    let e = world.entity();

    // Does not trigger "Position, Velocity" observer
    e.add(Position::id());

    // Entity now matches "Position, Velocity" query, triggers observer
    e.add(Velocity::id());
}

#[test]
fn observers_multi_term_observers_filter_terms_13() {
    let world = World::new();
    // Observer that only triggers on Position, not on Velocity
    world
        .observer::<flecs::OnAdd, ()>()
        .with(Position::id())
        .with(Velocity::id())
        .filter()
        .each_entity(|e, _| {
            // ...
        });

    let e = world.entity();

    // Doesn't trigger, entity doesn't have Velocity
    e.set(Position { x: 10.0, y: 20.0 });

    // Doesn't trigger, Velocity is a filter term
    e.set(Velocity { x: 1.0, y: 2.0 });

    // Triggers, entity now matches observer query
    e.set(Position { x: 20.0, y: 30.0 });
}

#[test]
fn observers_multi_term_observers_query_variables_14() {
    let world = World::new();
    // Observer that listens for spaceships docked to planets. The observer triggers
    // only when the SpaceShip tag or DockedTo pair is added to an entity. It will
    // not trigger when Planet is added to the target of a DockedTo pair.
    //
    // The DSL notation for this query is
    //   SpaceShip, (DockedTo, $object), Planet($object)
    world
        .observer::<flecs::OnAdd,()>()
        .with(SpaceShip::id())
        .with((DockedTo,"$object")).set_inout_none()
        .with(Planet::id()).set_src("$object")
        .each_entity(|e, _| {
            // ...
        });
}

#[test]
fn observers_multi_term_observers_event_downgrading_15() {
    let world = World::new();
    // OnSet observer with both component and tag
    world
        .observer::<flecs::OnSet, &Position>()
        .with(Npc::id()) // Tag
        .each_entity(|e, p| {
            // ...
        });

    let e = world.entity();

    // Doesn't trigger, entity doesn't have Npc
    e.set(Position { x: 10.0, y: 20.0 });

    // Produces and OnAdd event & triggers observer
    e.add(Npc::id());

    // Produces an OnSet event & triggers observer
    e.set(Position { x: 20.0, y: 30.0 });
}

#[test]
fn observers_multi_term_observers_event_inversion_16() {
    let world = World::new();
    // Observer with a Not term
    world
        .observer::<flecs::OnAdd, ()>()
        .with(Position::id())
        .without(Velocity::id())
        .each_entity(|e, _| {
            // ...
        });

    let e = world.entity();

    // Triggers the observer
    e.set(Position { x: 10.0, y: 20.0 });

    // Doesn't trigger the observer, entity doesn't match the observer query
    e.set(Velocity { x: 1.0, y: 2.0 });

    // Triggers the observer, as the Velocity term was inverted to OnRemove
    e.remove(Velocity::id());
}

#[test]
fn observers_monitors_17() {
    let world = World::new();
    // Monitor observer for Position, (ChildOf, *)
    world
        .observer::<flecs::Monitor, &Position>()
        .with((flecs::ChildOf, flecs::Wildcard))
        .each_iter(|it, i, p| {
            if it.event() == flecs::OnAdd::ID {
                // Entity started matching query
            } else if it.event() == flecs::OnRemove::ID {
                // Entity stopped matching query
            }
        });

    let p_a = world.entity();
    let p_b = world.entity();
    let e = world.entity();

    // Doesn't trigger the monitor, entity doesn't match
    e.set(Position { x: 10.0, y: 20.0 });

    // Entity now matches, triggers monitor with OnAdd event
    e.child_of(p_a);

    // Entity still matches the query, monitor doesn't trigger
    e.child_of(p_b);

    // Entity no longer matches, triggers monitor with OnRemove event
    e.remove(Position::id());
}

#[test]
fn observers_yield_existing_18() {
    let world = World::new();
    // Entity created before the observer
    let e1 = world.entity().set(Position { x: 10.0, y: 20.0 });

    // Yield existing observer
    world
        .observer::<flecs::OnAdd, ()>()
        .with(Position::id())
        .with(Velocity::id())
        .yield_existing()
        .each_iter(|it, i, _| {
            // ...
        });

    // Observer is invoked for e1

    // Fires observer as usual
    let e2 = world.entity().set(Position { x: 10.0, y: 20.0 });
}

#[test]
fn observers_yield_existing_yield_existing_flags_19() {
    let world = World::new();
    // TODO
}

#[test]
fn observers_fixed_source_terms_20() {
    let world = World::new();
    // Entity used for fixed source
    let game = world.entity().set(TimeOfDay(0.0));

    // Observer with fixed source
    world
        .observer::<flecs::OnSet, &TimeOfDay>()
        .term_at(0)
        .set_src(game) // Match TimeOfDay on game
        .each_iter(|it, i, time| {
            // ...
        });

    // Triggers observer
    game.set(TimeOfDay(1.0));
}

#[test]
fn observers_fixed_source_terms_singletons_21() {
    let world = World::new();

    world.component::<TimeOfDay>().add_trait::<flecs::Singleton>();

    world.set(TimeOfDay(0.0));

    // Observer with singleton source
    world
        .observer::<flecs::OnSet, &TimeOfDay>()
        .each_iter(|it, i, time| {
            // ...
        });

    // Triggers observer
    world.set(TimeOfDay(1.0));
}

#[test]
fn observers_event_propagation_22() {
    let world = World::new();
    // Create an observer that matches OnSet(Position) events on self and a parent
    world
        .observer::<flecs::OnSet, &Position>()
        .term_at(0)
        .self_()
        .up() // .trav(flecs::ChildOf) (default)
        .each_entity(|e, p| {
            // ...
        });

    let parent = world.entity();
    let child = world.entity().child_of(parent);

    // Invokes observer twice: once for the parent and once for the child
    parent.set(Position { x: 10.0, y: 20.0 });
}

#[test]
fn observers_event_forwarding_23() {
    let world = World::new();
    // Create an observer that matches OnAdd(Position) events on a parent
    world
        .observer::<flecs::OnAdd, ()>()
        .with(Position::id())
        .term_at(0)
        .up() // .trav(flecs::ChildOf) (default)
        .each_entity(|e, _| {
            // ...
        });

    let parent = world.entity().set(Position { x: 10.0, y: 20.0 });

    // Forwards OnAdd event for Position to child
    let child = world.entity().child_of(parent);
}

#[test]
fn observers_custom_events_24() {
    let world = World::new();
    // Create a custom event
    #[derive(Component)]
    struct Synchronized;

    // Alternatively, an plain entity could also be used as event
    // let Synchronized = world.entity();

    // Create an observer that matches a custom event
    world
        .observer::<Synchronized, &Position>()
        .each_entity(|e, p| {
    // ...
    });

    let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    // Emit custom event
    world
        .event()
        .add(Position::id())
        .entity(e)
        .emit(&Synchronized);
}

#[test]
fn observers_custom_events_entity_observers_25() {
    let world = World::new();
    // Create a custom event
    #[derive(Component)]
    struct Clicked;

    // Create entity
    let widget = world.entity_named("widget");

    // Create an entity observer
    widget.observe::<Clicked>(|| {
        // ...
    });

    // Emit entity event
    widget.emit(&Clicked);
}

#[test]
fn observers_custom_events_entity_observers_26() {
    let world = World::new();
    // Create a custom event
    #[derive(Component)]
    struct Resize {
        width: u32,
        height: u32,
    }

    // Create entity
    let widget = world.entity_named("widget");

    // Create an entity observer
    widget.observe_payload::<&Resize>(|r| {
    // ...
    });

    // Emit entity event
    widget.emit(&Resize {
    width: 100,
    height: 200,
    });
}

#[test]
fn observers_observer_execution_27() {
    let world = World::new();
    let e = world.entity();
    world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(|e, p| {
            // ...
        });

    // Observer is invoked as part of operation
    e.set(Position { x: 10.0, y: 20.0 });

    world.defer_begin();
    e.set(Position { x: 20.0, y: 30.0 });
    // Operation is delayed until here, observer is also invoked here
    world.defer_end();
}