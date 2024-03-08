use std::os::raw::c_void;

use crate::{
    core::{c_types::EntityT, world::World, FTime},
    sys::{ecs_run_w_filter, ecs_run_worker},
};

pub struct SystemRunnerFluent {
    stage: World,
    id: EntityT,
    stage_current: i32,
    stage_count: i32,
    offset: i32,
    limit: i32,
    delta_time: FTime,
    param: *mut c_void,
}

impl SystemRunnerFluent {
    pub fn new(
        world: &World,
        id: EntityT,
        stage_current: i32,
        stage_count: i32,
        delta_time: FTime,
        param: *mut c_void,
    ) -> Self {
        Self {
            stage: world.clone(),
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

    pub fn stage(&mut self, stage: &mut World) -> &mut Self {
        self.stage = stage.clone();
        self
    }
}

impl Drop for SystemRunnerFluent {
    fn drop(&mut self) {
        if self.stage_count != 0 {
            unsafe {
                ecs_run_worker(
                    self.stage.raw_world,
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
                    self.stage.raw_world,
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
