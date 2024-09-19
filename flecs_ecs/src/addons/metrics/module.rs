use flecs_ecs_derive::Component;

use crate::{core::World, prelude::Module};

#[derive(Clone, Copy, Component, Default)]
pub struct MetricsModule;

impl Module for MetricsModule {
    fn module(world: &World) {
        world.module::<MetricsModule>("::flecs::metrics");
        unsafe { flecs_ecs_sys::FlecsMetricsImport(world.ptr_mut()) };
    }
}
