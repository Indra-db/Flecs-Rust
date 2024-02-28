use std::{ops::Deref, os::raw::c_void};

use crate::core::{
    c_binding::bindings::{
        ecs_get_system_ctx, ecs_os_api, ecs_system_desc_t, ecs_system_get_query, ecs_system_init,
    },
    entity::Entity,
    query::Query,
    world::World,
};

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
        //todo!() this code can be rustified, ask

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
}
