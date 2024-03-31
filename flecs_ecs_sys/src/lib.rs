#[cfg(all(feature = "force_build_release", feature = "force_build_debug"))]
compile_error!(
    "Features 'force_build_release' and 'force_build_debug' cannot be enabled at the same time."
);

pub mod bindings;
pub mod extensions;
pub mod mbindings;

pub use bindings::*;
pub use mbindings::*;
