//! Raw FFI bindings to the [`flecs`] library.
//!
//! These are only intended for use by the higher level bindings that build atop these.
//!
//! [`flecs`]: https://www.flecs.dev/

#[cfg(all(feature = "force_build_release", feature = "force_build_debug"))]
compile_error!(
    "Features 'force_build_release' and 'force_build_debug' cannot be enabled at the same time."
);

//skips whole file from cargo fmt --all --check for CI
//the two variants match the compiled C profile: several structs carry
//FLECS_DEBUG-gated fields, so debug and release C builds have different layouts
#[cfg(not(flecs_c_release))]
#[rustfmt::skip]
mod bindings;
#[cfg(flecs_c_release)]
#[rustfmt::skip]
#[path = "bindings_release.rs"]
mod bindings;
mod extensions;
mod mbindings;

pub use bindings::*;
pub use mbindings::*;
