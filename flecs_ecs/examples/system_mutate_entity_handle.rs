mod common;
use common::*;

// This example is the same as the mutate_entity example, but instead stores the
// handle of the to be deleted entity in a component.

#[derive(Component)]
struct Timeout {
    pub to_delete: EntityId,
    pub value: f32,
}

#[derive(Component)]
struct Tag;

fn main() {
    let world = World::new();

    // System that deletes an entity after a timeout expires
    world
        .system_builder::<&mut Timeout>()
        .on_each_iter(|it, _index, timeout| {
            timeout.value -= it.delta_time();
            println!("{}", timeout.value);
            if timeout.value <= 0.0 {
                // Delete the entity

                // To make sure the delete operation is enqueued (see
                // mutate_entity example for more details) we need to provide it
                // with a mutable context (stage) using the mut() function. If
                // we don't provide a mutable context, the operation will be
                // attempted on the context stored in the flecs::entity object,
                // which would throw a readonly error.

                // To catch these errors at compile time, replace the type of
                // to_delete with flecs::entity_view. This class does not have
                // any methods for mutating the entity, which forces the code to
                // first call mut().

                // The it.world() function can be used to provide the context:
                //   t.to_delete.mut(it.world()).destruct();
                //
                // The current entity can also be used to provide context. This
                // is useful for functions that accept a flecs::entity:
                //   t.to_delete.mut(it.entity(index)).destruct();
                //
                // A shortcut is to use the iterator directly:
                let world = it.world();
                let to_delete = world.get_alive(timeout.to_delete);
                println!("Expire: {} deleted!", to_delete.name());
                to_delete.destruct();
            }
        });

    // System that prints remaining expiry time
    world
        .system_builder::<&Timeout>()
        .on_each_entity(|e, timeout| {
            let world = e.world();
            let to_delete = world.get_alive(timeout.to_delete);
            println!(
                "PrintExpire: {} has {:.2} seconds left",
                to_delete.name(),
                timeout.value
            );
        });

    // Observer that triggers when entity is actually deleted
    world
        .observer_builder::<&Tag>()
        .add_event::<flecs::OnRemove>()
        .on_each_entity(|e, _tag| {
            println!("Expired: {} actually deleted", e.name());
        });

    let to_delete = world.new_entity_named(c"ToDelete").add::<Tag>();

    world.new_entity_named(c"MyEntity").set(Timeout {
        to_delete: EntityId(to_delete.raw_id),
        value: 3.0,
    });

    world.set_target_fps(1.0);

    while world.progress() {
        // If entity is no longer alive, exit
        if !to_delete.is_alive() {
            break;
        }

        println!("Tick...");
    }

    // Output
    //  PrintExpire: ToDelete has 2.00 seconds left
    //  Tick...
    //  PrintExpire: ToDelete has 0.98 seconds left
    //  Tick...
    //  Expire: ToDelete deleted!
    //  PrintExpire: ToDelete has -0.03 seconds left
    //  Expired: ToDelete actually deleted
}
