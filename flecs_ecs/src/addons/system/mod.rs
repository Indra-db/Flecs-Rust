/// addon system, flecs system framework
#[cfg(feature = "flecs_system")]
pub mod system {
    pub mod fsystem;
    pub mod system_builder;
    pub use fsystem::*;
    pub use system_builder::*;
}
