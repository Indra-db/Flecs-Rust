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
#[rustfmt::skip]
mod bindings;
mod extensions;
mod mbindings;

pub use bindings::*;
pub use mbindings::*;
