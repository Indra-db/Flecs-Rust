/// addon meta, flecs reflection framework
#[cfg(feature = "flecs_meta")]
pub mod meta {
    pub mod declarations;
    pub mod opaque;
    pub use declarations::*;
    pub use opaque::*;
}
