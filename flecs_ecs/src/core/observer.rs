use std::{ops::Deref, os::raw::c_void};

use super::{
    c_binding::bindings::{
        ecs_get_observer_ctx, ecs_observer_desc_t, ecs_observer_init, ecs_observer_t, ecs_os_api,
    },
    c_types::{Poly, ECS_OBSERVER},
    entity::Entity,
    filter::Filter,
    world::World,
};

#[derive(Clone)]
pub struct Observer {
    pub entity: Entity,
    world: World,
}

impl Deref for Observer {
    type Target = Entity;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl Observer {
    //TODO in query ect desc is a pointer, does it need to be?
    pub fn new(world: &World, mut desc: ecs_observer_desc_t, is_instanced: bool) -> Self {
        if !desc.filter.instanced {
            desc.filter.instanced = is_instanced;
        }

        let id = unsafe { ecs_observer_init(world.raw_world, &desc) };
        let entity = Entity::new_from_existing_raw(world.raw_world, id);

        unsafe {
            if !desc.filter.terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func(desc.filter.terms_buffer as *mut _)
                }
            }
        }

        Self {
            entity,
            world: world.clone(),
        }
    }

    pub fn new_from_existing(world: &World, observer_entity: Entity) -> Self {
        Self {
            world: world.clone(),
            entity: observer_entity,
        }
    }

    pub fn set_context(&mut self, context: *mut c_void) {
        let desc: ecs_observer_desc_t = ecs_observer_desc_t {
            entity: self.raw_id,
            ctx: context,
            ..Default::default()
        };

        unsafe {
            ecs_observer_init(self.world.raw_world, &desc);
        }
    }

    pub fn get_context(&self) -> *mut c_void {
        unsafe { ecs_get_observer_ctx(self.world.raw_world, self.raw_id) }
    }

    pub fn query(&mut self) -> Filter<()> {
        let poly: *const Poly = self.get_target_for_pair_as_first::<Poly>(ECS_OBSERVER);
        let obj: *mut ecs_observer_t = unsafe { (*poly).poly as *mut ecs_observer_t };
        let world: World = self.get_as_world();
        Filter::<()>::new_ownership(&self.world, unsafe { &mut (*obj).filter })
    }
}
