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

//todo!() should implement copy?
#[derive(Clone)]
pub struct System {
    pub entity: Entity,
    world: World,
}

impl Deref for System {
    type Target = Entity;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl System {
    //todo!() in query ect desc is a pointer, does it need to be?
    pub fn new(world: &World, mut desc: ecs_system_desc_t, is_instanced: bool) -> Self {
        if !desc.query.filter.instanced {
            desc.query.filter.instanced = is_instanced;
        }

        let id = unsafe { ecs_system_init(world.raw_world, &desc) };
        let entity = Entity::new_from_existing_raw(world.raw_world, id);

        unsafe {
            if !desc.query.filter.terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func(desc.query.filter.terms_buffer as *mut _)
                }
            }
        }

        Self {
            entity,
            world: world.clone(),
        }
    }

    pub fn new_from_existing(world: &World, system_entity: Entity) -> Self {
        Self {
            world: world.clone(),
            entity: system_entity,
        }
    }

    pub(crate) fn system_init(world: &World) {
        world.component_named::<TickSource>(
            CStr::from_bytes_with_nul(b"flecs::system::TickSource\0").unwrap(),
        );
    }

    pub fn set_context(&mut self, context: *mut c_void) {
        let mut desc: ecs_system_desc_t = Default::default();
        desc.entity = self.raw_id;
        desc.ctx = context;
        unsafe {
            ecs_system_init(self.world.raw_world, &desc);
        }
    }

    pub fn get_context(&self) -> *mut c_void {
        unsafe { ecs_get_system_ctx(self.world.raw_world, self.raw_id) }
    }

    pub fn query(&mut self) -> Query<()> {
        Query::<()>::new_ownership(&self.world, unsafe {
            ecs_system_get_query(self.world.raw_world, self.raw_id)
        })
    }

    pub fn run(&self, delta_time: FTime, param: *mut c_void) -> SystemRunnerFluent {
        SystemRunnerFluent::new(&self.world, self.raw_id, 0, 0, delta_time, param)
    }

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
