use std::os::raw::c_void;

use crate::{
    core::{c_types::EntityT, world::World, FTime, IntoWorld},
    sys::{ecs_run_w_filter, ecs_run_worker},
};

pub struct SystemRunnerFluent<'a> {
    stage: &'a World,
    id: EntityT,
    stage_current: i32,
    stage_count: i32,
    offset: i32,
    limit: i32,
    delta_time: FTime,
    param: *mut c_void,
}

impl<'a> SystemRunnerFluent<'a> {
    pub fn new(
        world: &'a World,
        id: EntityT,
        stage_current: i32,
        stage_count: i32,
        delta_time: FTime,
        param: *mut c_void,
    ) -> Self {
        Self {
            stage: world,
            id,
            stage_current,
            stage_count,
            offset: 0,
            limit: 0,
            delta_time,
            param,
        }
    }

    pub fn offset(&mut self, offset: i32) -> &mut Self {
        self.offset = offset;
        self
    }

    pub fn limit(&mut self, limit: i32) -> &mut Self {
        self.limit = limit;
        self
    }

    pub fn stage(&mut self, stage: &'a World) -> &mut Self {
        self.stage = stage;
        self
    }
}

impl<'a> Drop for SystemRunnerFluent<'a> {
    fn drop(&mut self) {
        if self.stage_count != 0 {
            unsafe {
                ecs_run_worker(
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
                ecs_run_w_filter(
                    self.stage.world_ptr_mut(),
                    self.id,
                    self.delta_time,
                    self.offset,
                    self.limit,
                    self.param,
                );
            }
        }
    }
}
