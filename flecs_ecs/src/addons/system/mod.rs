/// addon system, flecs system framework
#[cfg(feature = "flecs_system")]
pub mod system {
    pub mod system;
    pub mod system_builder;
    pub use system::*;
    pub use system_builder::*;
}
