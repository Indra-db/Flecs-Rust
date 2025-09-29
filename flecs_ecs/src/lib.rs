//! [Flecs] is a fast and lightweight Entity Component System that lets you build games and simulations with millions of entities.
//!
//! This library provides a comprehensive and low-overhead Rust binding for [flecs].
//!
//! [Flecs]: https://www.flecs.dev/

//this is commented since `no_std` is not ready yet
//#![cfg_attr(not(feature = "std"), no_std)] // Enable `no_std` if `std` feature is disabled
#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
#[cfg(test)]
mod tests {
    mod crash_handler {
        /// Use the crash handler for unit tests
        #[cfg(feature = "test-with-crash-handler")]
        #[ctor::ctor]
        fn register_test_crash_handler() {
            test_crash_handler::register();
        }
    }
}
