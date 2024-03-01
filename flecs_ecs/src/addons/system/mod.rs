pub mod system_builder;
pub mod system_runner_fluent;

use std::{ffi::CStr, ops::Deref, os::raw::c_void};

pub use system_builder::*;
pub use system_runner_fluent::*;

use crate::core::{
    c_binding::{
        ecs_get_system_ctx, ecs_os_api, ecs_system_desc_t, ecs_system_get_query, ecs_system_init,
    },
    Entity, FTime, Query, TickSource, World,
};

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
        let desc: ecs_system_desc_t = ecs_system_desc_t {
            entity: self.raw_id,
            ctx: context,
            ..Default::default()
        };

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
