mod common;
use common::*;

#[derive(Clone, Component, Default)]
struct Timeout {
    pub value: f32,
}

fn main() {
    let world = World::new();

    // System that deletes an entity after a timeout expires
    world
        .system_builder::<(&mut Timeout,)>()
        .on_each_iter(|it, index, (timeout,)| {
            timeout.value -= it.get_delta_time();
            if timeout.value <= 0.0 {
                // Delete the entity

                // To make sure that the storage doesn't change while a system
                // is iterating entities, and multiple threads can safely access
                // the data, mutations (like a delete) are added to a command
                // queue and executed when it's safe to do so.

                // When the entity to be mutated is not the same as the entity
                // provided by the system, an additional mut() call is required.
                // See the mutate_entity_handle example.
                let e = it.get_entity(index);
                e.destruct();
                println!("Expire: {} deleted!", e.get_name());
            }
        });

    // System that prints remaining expiry time
    world
        .system_builder::<(&Timeout,)>()
        .on_each_entity(|e, (timeout,)| {
            println!(
                "PrintExpire: {} has {:.2} seconds left",
                e.get_name(),
                timeout.value
            );
        });

    // Observer that triggers when entity is actually deleted
    world
        .observer_builder::<(&Timeout,)>()
        .add_event(ECS_ON_REMOVE)
        .on_each_entity(|e, (_timeout,)| {
            println!("Expired: {} actually deleted", e.get_name());
        });

    let e = world
        .new_entity_named(c"MyEntity")
        .set(Timeout { value: 3.0 });

    world.set_target_fps(1.0);

    while world.progress() {
        // If entity is no longer alive, exit
        if !e.is_alive() {
            break;
        }

        println!("Tick...");
    }

    // Output
    //  PrintExpire: MyEntity has 2.00 seconds left
    //  Tick...
    //  PrintExpire: MyEntity has 0.99 seconds left
    //  Tick...
    //  Expire: MyEntity deleted!
    //  PrintExpire: MyEntity has -0.03 seconds left
    //  Expired: MyEntity actually deleted
}
