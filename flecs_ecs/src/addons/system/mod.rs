//! Systems are a query + function that can be ran manually or by a pipeline.
//!
//! The system module allows for creating and running systems. A system is a
//! query in combination with a callback function. In addition systems have
//! support for time management, scheduling via pipeline and can be monitored by the stats addon.

mod system_builder;
mod system_runner_fluent;

use std::{ffi::CStr, ops::Deref, os::raw::c_void};

pub use system_builder::*;
pub use system_runner_fluent::*;

use crate::{
    core::{Entity, FTime, Query, TickSource, World},
    sys::{
        ecs_os_api, ecs_system_desc_t, ecs_system_get_ctx, ecs_system_get_query, ecs_system_init,
    },
};

#[derive(Clone)]
pub struct System<'a> {
    pub entity: Entity<'a>,
    world: &'a World,
}

impl<'a> Deref for System<'a> {
    type Target = Entity<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a> System<'a> {
    //todo!() in query etc desc is a pointer, does it need to be?
    /// Create a new system
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the system in.
    /// * `desc` - The system description.
    ///
    /// # See also
    ///
    /// * C++ API: `system::system`
    #[doc(alias = "system::system")]
    pub fn new(world: &'a World, mut desc: ecs_system_desc_t, is_instanced: bool) -> Self {
        if !desc.query.filter.instanced {
            desc.query.filter.instanced = is_instanced;
        }

        let id = unsafe { ecs_system_init(world.raw_world, &desc) };
        let entity = Entity::new_from_existing(world, id);

        unsafe {
            if !desc.query.filter.terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func(desc.query.filter.terms_buffer as *mut _);
                }
            }
        }

        Self { entity, world }
    }

    /// Wrap an existing system entity in a system object
    ///
    /// # Arguments
    ///
    /// * `world` - The world the system is in.
    /// * `system_entity` - The entity of the system.
    ///
    /// # See also
    ///
    /// * C++ API: `system::system`
    #[doc(alias = "system::system")]
    pub fn new_from_existing(world: &'a World, system_entity: Entity<'a>) -> Self {
        Self {
            world,
            entity: system_entity,
        }
    }

    /// Initialize the system module and register the `TickSource` component
    ///
    /// # Arguments
    ///
    /// * `world` - The world to initialize the system module in.
    ///
    /// # See also
    ///
    /// * C++ API: `system::system_init`
    #[doc(alias = "system::system_init")]
    pub(crate) fn system_init(world: &World) {
        world.component_named::<TickSource>(
            CStr::from_bytes_with_nul(b"flecs::system::TickSource\0").unwrap(),
        );
    }

    /// Set the context for the system
    ///
    /// # Arguments
    ///
    /// * `context` - The context to set.
    ///
    /// # See also
    ///
    /// * C++ API: `system::ctx`
    #[doc(alias = "system::ctx")]
    pub fn set_context(&mut self, context: *mut c_void) {
        let desc: ecs_system_desc_t = ecs_system_desc_t {
            entity: self.raw_id,
            ctx: context,
            ..Default::default()
        };

        unsafe {
            ecs_system_init(self.world.raw_world, &desc);
        }
    }

    /// Get the context for the system
    ///
    /// # See also
    ///
    /// * C++ API: `system::ctx`
    #[doc(alias = "system::ctx")]
    pub fn context(&self) -> *mut c_void {
        unsafe { ecs_system_get_ctx(self.world.raw_world, self.raw_id) }
    }

    /// Get the underlying query for the system
    ///
    /// # See also
    ///
    /// * C++ API: `system::query`
    #[doc(alias = "system::query")]
    pub fn query(&mut self) -> Query<()> {
        Query::<()>::new_ownership(&self.world, unsafe {
            ecs_system_get_query(self.world.raw_world, self.raw_id)
        })
    }

    /// Run the system
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time delta.
    /// * `param` - A user-defined parameter to pass to the system
    ///
    /// # See also
    ///
    /// * C++ API: `system::run`
    #[doc(alias = "system::run")]
    #[inline]
    pub fn run_dt_param(&self, delta_time: FTime, param: *mut c_void) -> SystemRunnerFluent {
        SystemRunnerFluent::new(&self.world, self.raw_id, 0, 0, delta_time, param)
    }

    /// Run the system
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time delta.
    ///
    /// # See also
    ///
    /// * C++ API: `system::run`
    #[doc(alias = "system::run")]
    #[inline]
    pub fn run_dt(&self, delta_time: FTime) -> SystemRunnerFluent {
        self.run_dt_param(delta_time, std::ptr::null_mut())
    }

    /// Run the system
    ///
    /// # See also
    ///
    /// * C++ API: `system::run`
    #[doc(alias = "system::run")]
    #[inline]
    pub fn run(&self) -> SystemRunnerFluent {
        self.run_dt_param(0.0, std::ptr::null_mut())
    }

    /// Run the system worker
    ///
    /// # Arguments
    ///
    /// * `stage_current` - The current stage.
    /// * `stage_count` - The total number of stages.
    /// * `delta_time` - The time delta.
    /// * `param` - An optional parameter to pass to the system.
    ///
    /// # See also
    ///
    /// * C++ API: `system::run_worker`
    #[doc(alias = "system::run_worker")]
    pub fn run_worker(
        &self,
        stage_current: i32,
        stage_count: i32,
        delta_time: FTime,
        param: *mut c_void,
    ) -> SystemRunnerFluent {
        SystemRunnerFluent::new(
            &self.world,
            self.raw_id,
            stage_current,
            stage_count,
            delta_time,
            param,
        )
    }
}
