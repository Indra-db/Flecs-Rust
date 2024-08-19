#[cfg(feature = "flecs_module")]
pub use crate::addons::module::Module;
pub use crate::addons::*;
pub use crate::core::*;
pub use crate::macros::*;
pub use flecs_ecs_sys::EcsComponent;

#[cfg(feature = "flecs_meta")]
pub use crate::addons::meta::*;
#[cfg(feature = "flecs_meta")]
pub use crate::{component, component_ext, member, member_ext};
