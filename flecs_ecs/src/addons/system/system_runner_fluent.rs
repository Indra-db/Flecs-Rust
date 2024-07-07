//! System module implementation
//!
use std::os::raw::c_void;

use crate::core::*;
use crate::sys;

pub struct SystemRunnerFluent<'a> {
    stage: WorldRef<'a>,
    id: sys::ecs_entity_t,
    stage_current: i32,
    stage_count: i32,
    offset: i32,
    limit: i32,
    delta_time: FTime,
    param: *mut c_void,
}

impl<'a> SystemRunnerFluent<'a> {
    /// Create a new system runner fluent interface
    pub fn new(
        world: impl WorldProvider<'a>,
        id: impl Into<Entity>,
        stage_current: i32,
        stage_count: i32,
        delta_time: FTime,
        param: *mut c_void,
    ) -> Self {
        Self {
            stage: world.world(),
            id: *id.into(),
            stage_current,
            stage_count,
            offset: 0,
            limit: 0,
            delta_time,
            param,
        }
    }

    /// Set the offset
    pub fn set_offset(&mut self, offset: i32) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Set the limit
    pub fn set_limit(&mut self, limit: i32) -> &mut Self {
        self.limit = limit;
        self
    }

    /// Set the stage
    pub fn set_stage(&mut self, stage: impl WorldProvider<'a>) -> &mut Self {
        self.stage = stage.world();
        self
    }
}

impl<'a> Drop for SystemRunnerFluent<'a> {
    fn drop(&mut self) {
        if self.stage_count != 0 {
            unsafe {
                sys::ecs_run_worker(
                    self.stage.world_ptr_mut(),
                    self.id,
                    self.stage_current,
                    self.stage_count,
                    self.delta_time,
                    self.param,
                );
            }
        } else {
            unsafe {
                sys::ecs_run(
                    self.stage.world_ptr_mut(),
                    self.id,
                    self.delta_time,
                    self.param,
                );
            }
        }
    }
}
