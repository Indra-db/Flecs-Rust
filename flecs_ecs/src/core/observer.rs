use std::{ops::Deref, os::raw::c_void};

use crate::sys::{
    ecs_observer_desc_t, ecs_observer_get_ctx, ecs_observer_init, ecs_observer_t, ecs_os_api,
};

use super::{
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
    //TODO in query etc desc is a pointer, does it need to be?
    /// Create a new observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::observer`
    #[doc(alias = "observer::observer")]
    pub fn new(world: &World, mut desc: ecs_observer_desc_t, is_instanced: bool) -> Self {
        if !desc.filter.instanced {
            desc.filter.instanced = is_instanced;
        }

        let id = unsafe { ecs_observer_init(world.raw_world, &desc) };
        let entity = Entity::new_from_existing_raw(world.raw_world, id);

        unsafe {
            if !desc.filter.terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func(desc.filter.terms_buffer as *mut _);
                }
            }
        }

        Self {
            entity,
            world: world.clone(),
        }
    }

    /// Wrap an existing observer entity in an observer object
    pub fn new_from_existing(world: &World, observer_entity: Entity) -> Self {
        Self {
            world: world.clone(),
            entity: observer_entity,
        }
    }

    /// Set the context for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::ctx`
    #[doc(alias = "observer::ctx")]
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

    /// Get the context for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::ctx`
    #[doc(alias = "observer::ctx")]
    pub fn get_context(&self) -> *mut c_void {
        unsafe { ecs_observer_get_ctx(self.world.raw_world, self.raw_id) }
    }

    /// Get the filter for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::query`
    #[doc(alias = "observer::query")]
    pub fn query(&mut self) -> Filter<()> {
        let poly: *const Poly = self.get_target_for_pair_first::<Poly>(ECS_OBSERVER);
        let obj: *mut ecs_observer_t = unsafe { (*poly).poly as *mut ecs_observer_t };
        Filter::<()>::new_ownership(&self.world, unsafe { &mut (*obj).filter })
    }
}
