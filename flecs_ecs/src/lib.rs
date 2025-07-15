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
            EntityView::new_from(self, id).add(id)
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
        #[derive(Debug, Component, Default)]
        struct Position {
            x: f32,
            y: f32,
        }

        #[derive(Debug, Component)]
        struct Tag;

        let world = World::new();
        let e = world.entity();

        // let e_pair = ecs_pair(e, e);
        // // dbg!(check_add_id_validity(world.ptr_mut(), e));
        // // dbg!(check_add_id_validity(world.ptr_mut(), e_pair));
        // // dbg!(check_add_id_validity(world.ptr_mut(), p));
        // // dbg!(check_add_id_validity(world.ptr_mut(), t));
        // let p_t = ecs_pair(p, t);
        // let t_p = ecs_pair(t, p);
        // // dbg!(check_add_id_validity(world.ptr_mut(), p_t));
        // dbg!(check_add_id_validity(world.ptr_mut(), t_p));

        // // all same API
        //
        // dbg!(flecs_ecs::core::utility::id::Id::<Position>::IF_ID_IS_DEFAULT);
        // dbg!(flecs_ecs::core::utility::id::Id::<Position>::IS_TYPED);
        // dbg!(flecs_ecs::core::utility::id::Id::<Tag>::IF_ID_IS_DEFAULT);
        // dbg!(flecs_ecs::core::utility::id::Id::<Tag>::IS_TYPED);
        // dbg!(
        //     <(
        //         flecs_ecs::core::utility::id::Id::<Tag>,
        //         flecs_ecs::core::utility::id::Id::<Position>
        //     )>::IS_PAIR
        // );
        // dbg!(
        //     <(
        //         flecs_ecs::core::utility::id::Id::<Tag>,
        //         flecs_ecs::core::utility::id::Id::<Position>
        //     )>::IS_TYPED_SECOND
        // );
        // dbg!(
        //     <(
        //         flecs_ecs::core::utility::id::Id::<Tag>,
        //         flecs_ecs::core::utility::id::Id::<Position>
        //     )>::IF_ID_IS_DEFAULT_SECOND
        // );

        e.add((Tag, Position::id()));
        e.add((Position::id(), Position::id()));
        e.add((Position::id(), Tag));

        e.add(e);
        e.add((e, e));
        e.add((Position::id(), e));
        e.add((e, Position::id()));

        // //ignore
        // assert!(e.has(Position::id()));
        // assert!(e.has(e));
        // assert!(e.has(Position::id()));
        // assert!(e.has((e, e)));
        // assert!(e.has((Position::id(), e)));
        // assert!(e.has((e, Position::id())));
    }
}
