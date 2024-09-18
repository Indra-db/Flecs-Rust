mod types;
pub use types::*;

use super::module::Module;
use crate::core::World;
use flecs_ecs_derive::Component;

#[derive(Clone, Copy, Component, Default)]
pub struct UnitsModule;

impl Module for UnitsModule {
    fn module(world: &World) {
        world.module::<UnitsModule>("::flecs::units");
        unsafe { flecs_ecs_sys::FlecsUnitsImport(world.ptr_mut()) };
    }
}
