mod common;
use common::*;

// When an application calls world.progress(), the world is put in readonly mode.
// This ensures that systems (on multiple threads) can safely iterate
// components, without having to worry about components moving around while
// they're being read. This has as side effect that any operations (like adding
// or removing components) are not visible until the end of the frame (see the
// sync_point example for more details).
// Sometimes this is not what you want, and you need a change to be visible
// immediately. For these use cases, applications can use a no_readonly system.
// This temporarily takes the world out of readonly mode, so a system can make
// changes that are directly visible.
// Because they mutate the world directly, no_readonly systems are never ran on
// more than one thread, and no other systems are ran at the same time.

#[derive(Component)]
struct Waiter;

#[derive(Component)]
struct Plate;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Create query to find all waiters without a plate
    let mut q_waiter = world
        .query::<&Waiter>()
        .without::<(&Plate, flecs::Wildcard)>()
        .build();

    // System that assigns plates to waiter. By making this system no_readonly
    // plate assignments are assigned directly (not deferred) to waiters, which
    // ensures that we won't assign plates to the same waiter more than once.
    world
        .system_named::<&Plate>(c"AssignPlate")
        .without::<(&Waiter, flecs::Wildcard)>()
        .immediate(true)
        .on_iter_only(|it| {
            for i in it.iter() {
                let plate = it.entity(i);

                // Find an available waiter
                if let Some(waiter) = q_waiter.first() {
                    // An available waiter was found, assign a plate to it so
                    // that the next plate will no longer find it.
                    // The defer_suspend function temporarily suspends deferring
                    // operations, which ensures that our plate is assigned
                    // immediately. Even though this is a no_readonly system,
                    // deferring is still enabled by default as adding/removing
                    // components to the entities being iterated would interfere
                    // with the system iterator.
                    it.world().defer_suspend();
                    waiter.add_pair_first::<&Plate>(plate);
                    it.world().defer_resume();

                    // Now that deferring is resumed, we can safely also add the
                    // waiter to the plate. We can't do this while deferring is
                    // suspended, because the plate is the entity we're
                    // currently iterating, and we don't want to move it to a
                    // different table while we're iterating it.

                    plate.add_pair_first::<&Waiter>(waiter);

                    fprintln!(snap, "Assigned {} to {}!", waiter.name(), plate.name());
                }
            }
        });

    let waiter_1 = world.new_entity_named(c"waiter_1").add::<Waiter>();
    world.new_entity_named(c"waiter_2").add::<Waiter>();
    world.new_entity_named(c"waiter_3").add::<Waiter>();

    world.new_entity_named(c"plate_1").add::<Plate>();
    let plate_2 = world.new_entity_named(c"plate_2").add::<Plate>();
    world.new_entity_named(c"plate_3").add::<Plate>();

    waiter_1.add_pair_first::<&Plate>(plate_2);
    plate_2.add_pair_first::<&Waiter>(waiter_1);

    // run systems
    world.progress();

    snap.test();

    // Output:
    //  Assigned waiter_3 to plate_1!
    //  Assigned waiter_2 to plate_3!
}
