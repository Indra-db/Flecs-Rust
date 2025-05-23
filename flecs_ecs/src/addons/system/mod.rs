//! Systems are a query + function that can be ran manually or by a pipeline.
//!
//! The system module allows for creating and running systems. A system is a
//! query in combination with a callback function. In addition systems have
//! support for time management, scheduling via pipeline and can be monitored by the stats addon.

mod system_builder;
mod system_runner_fluent;
pub use system_builder::*;
pub use system_runner_fluent::*;

use core::ops::DerefMut;
use core::{ffi::c_void, ops::Deref, ptr::NonNull};

use crate::core::*;
use crate::sys;

/// Systems are a query + function that can be ran manually or by a pipeline.
#[derive(Clone, Copy)]
pub struct System<'a> {
    pub(crate) entity: EntityView<'a>,
}

impl<'a> Deref for System<'a> {
    type Target = EntityView<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl DerefMut for System<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<'a> WorldProvider<'a> for System<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
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
    pub fn new(world: impl WorldProvider<'a>, desc: sys::ecs_system_desc_t) -> Self {
        let id = unsafe { sys::ecs_system_init(world.world_ptr_mut(), &desc) };
        let entity = EntityView::new_from(world.world(), id);

        Self { entity }
    }

    /// Wrap an existing system entity in a system object
    ///
    /// # Arguments
    ///
    /// * `world` - The world the system is in.
    /// * `system_entity` - The entity of the system.
    pub fn new_from_existing(system_entity: EntityView<'a>) -> Self {
        Self {
            entity: system_entity,
        }
    }

    /// Set the context for the system
    ///
    /// # Arguments
    ///
    /// * `context` - The context to set.
    pub fn set_context(&mut self, context: *mut c_void) {
        let desc: sys::ecs_system_desc_t = sys::ecs_system_desc_t {
            entity: *self.id(),
            ctx: context,
            ..Default::default()
        };

        unsafe {
            sys::ecs_system_init(self.world.world_ptr_mut(), &desc);
        }
    }

    /// Get the context for the system
    pub fn context(&self) -> *mut c_void {
        unsafe { (*sys::ecs_system_get(self.world.world_ptr(), *self.id())).ctx }
    }

    /// Get the underlying query for the system
    pub fn query(&self) -> Query<()> {
        let query = unsafe {
            NonNull::new_unchecked((*sys::ecs_system_get(self.world.world_ptr(), *self.id())).query)
        };
        unsafe { Query::<()>::new_from(query) }
    }

    /// Run the system
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time delta.
    /// * `param` - A user-defined parameter to pass to the system
    #[inline]
    pub fn run_dt_param(&self, delta_time: FTime, param: *mut c_void) -> SystemRunnerFluent {
        SystemRunnerFluent::new(self.world.real_world(), *self.id(), 0, 0, delta_time, param)
    }

    /// Run the system
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time delta.
    #[inline]
    pub fn run_dt(&self, delta_time: FTime) -> SystemRunnerFluent {
        self.run_dt_param(delta_time, core::ptr::null_mut())
    }

    /// Run the system
    #[inline]
    pub fn run(&self) -> SystemRunnerFluent {
        self.run_dt_param(0.0, core::ptr::null_mut())
    }

    /// Run the system worker
    ///
    /// # Arguments
    ///
    /// * `stage_current` - The current stage.
    /// * `stage_count` - The total number of stages.
    /// * `delta_time` - The time delta.
    /// * `param` - An optional parameter to pass to the system.
    pub fn run_worker(
        &self,
        stage_current: i32,
        stage_count: i32,
        delta_time: FTime,
        param: *mut c_void,
    ) -> SystemRunnerFluent {
        SystemRunnerFluent::new(
            self.world,
            *self.id(),
            stage_current,
            stage_count,
            delta_time,
            param,
        )
    }
}
