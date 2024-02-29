use std::{ffi::CStr, ops::Deref, os::raw::c_void};

use crate::core::{
    c_binding::bindings::{
        ecs_get_system_ctx, ecs_os_api, ecs_run_w_filter, ecs_run_worker, ecs_system_desc_t,
        ecs_system_get_query, ecs_system_init,
    },
    c_types::{EntityT, TickSource},
    entity::Entity,
    id::Id,
    query::Query,
    utility::types::FTime,
    world::World,
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
