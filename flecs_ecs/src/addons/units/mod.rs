mod types;
pub use types::*;

use super::module::Module;
use crate::core::World;
use flecs_ecs_derive::Component;

#[derive(Clone, Copy, Component, Default)]
pub struct Units;

impl Module for Units {
    fn module(world: &World) {
        unsafe { flecs_ecs_sys::FlecsUnitsImport(world.ptr_mut()) };
    }
}
