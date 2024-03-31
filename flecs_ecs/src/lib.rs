#![allow(dead_code)]

#[cfg(all(
    feature = "flecs_force_build_release_c",
    feature = "flecs_force_build_debug_c"
))]
compile_error!("Features 'flecs_force_build_release_c' and 'flecs_force_build_debug_c' cannot be enabled at the same time.");

pub use flecs_ecs_derive as macros;
pub use flecs_ecs_sys as sys;

pub mod core;

pub mod addons;
