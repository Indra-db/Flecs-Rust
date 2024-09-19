use crate::{addons::metrics::MetricsModule, core::World, prelude::Module};
use flecs_ecs_derive::Component;

#[derive(Clone, Copy, Component, Default)]
pub struct AlertsModule;

impl Module for AlertsModule {
    fn module(world: &World) {
        world.import::<MetricsModule>();

        world.module::<AlertsModule>("::flecs::alerts");

        // Import C module
        unsafe { flecs_ecs_sys::FlecsAlertsImport(world.ptr_mut()) };
    }
}
