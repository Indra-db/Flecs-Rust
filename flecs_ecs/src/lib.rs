//! [Flecs] is a fast and lightweight Entity Component System that lets you build games and simulations with millions of entities.
//!
//! This library provides a comprehensive and low-overhead Rust binding for [flecs].
//!
//! [Flecs]: https://www.flecs.dev/

//this is commented since `no_std` is not ready yet
//#![cfg_attr(not(feature = "std"), no_std)] // Enable `no_std` if `std` feature is disabled
#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(all(
    feature = "flecs_force_build_release_c",
    feature = "flecs_force_build_debug_c"
))]
compile_error!(
    "Features 'flecs_force_build_release_c' and 'flecs_force_build_debug_c' cannot be enabled at the same time."
);

#[cfg(not(feature = "std"))]
const _: () = panic!("no_std is not ready yet");

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
extern crate alloc;

pub use flecs_ecs_derive as macros;
pub use flecs_ecs_sys as sys;

pub mod core;

pub mod addons;

/// this is to allow using the proc macro's inside lib itself that implements its own traits.
extern crate self as flecs_ecs;

pub mod prelude;

/// Use the crash handler for unit tests
#[cfg(all(test, feature = "test-with-crash-handler"))]
#[ctor::ctor]
fn register_test_crash_handler() {
    test_crash_handler::register();
}

#[cfg(test)]
mod tests {
    use super::*;
    use flecs_ecs::prelude::*;

    #[derive(Debug, Component, Default)]
    pub struct Position {
        pub x: f32,
        pub y: f32,
    }

    impl World {
        pub fn add_(&self, id: impl IntoEntity) -> EntityView {
            let id = id.into_entity(self);
            EntityView::new_from(self, id).add_id(id)
        }
    }

    fn take_copy<T: Copy>(t: T) -> T {
        t
    }

    fn take_into_entity<T: IntoEntity>(t: T, world: &World) -> Entity {
        t.into_entity(world)
    }

    #[test]
    fn test_add_op() {
        let world = World::new();
        let e = world.entity();

        // all same API
        world.add_id(e);
        world.add_id(id::<Position>());
        world.add_id((e, e));
        world.add_id((id::<Position>(), e));
        world.add_id((e, id::<Position>()));

        assert!(world.has::<Position>());
        assert!(world.has_id(e));
        assert!(world.has_id(id::<Position>()));
        assert!(world.has_id((e, e)));
        assert!(world.has_id((id::<Position>(), e)));
        assert!(world.has_id((e, id::<Position>())));
    }
}
