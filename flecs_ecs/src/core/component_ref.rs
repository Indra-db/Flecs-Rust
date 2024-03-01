use std::{marker::PhantomData, os::raw::c_void};

use crate::{
    core::{
        c_binding::bindings::{ecs_get_world, ecs_ref_get_id, ecs_ref_init_id},
        FlecsErrorCode,
    },
    ecs_assert,
};

use super::{
    c_types::{EntityT, IdT, RefT, WorldT},
    component_registration::CachedComponentData,
    entity::Entity,
};

/// A reference to a component from a specific entity.
pub struct Ref<T: CachedComponentData> {
    world: *mut WorldT,
    component_ref: RefT,
    _marker: PhantomData<T>,
}

impl<T: CachedComponentData> Ref<T> {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(mut world: *mut WorldT, entity: EntityT, mut id: IdT) -> Self {
        // the world we were called with may be a stage; convert it to a world
        // here if that is the case
        world = if !world.is_null() {
            unsafe { ecs_get_world(world as *const c_void) as *mut WorldT }
        } else {
            std::ptr::null_mut()
        };

        if id == 0 {
            id = T::get_id(world);
        }

        ecs_assert!(T::get_size(world) != 0, FlecsErrorCode::InvalidParameter);

        let component_ref = unsafe { ecs_ref_init_id(world, entity, id) };

        Ref {
            world,
            component_ref,
            _marker: PhantomData,
        }
    }

    pub fn get(&mut self) -> *mut T {
        unsafe {
            ecs_ref_get_id(self.world, &mut self.component_ref, self.component_ref.id) as *mut T
        }
    }

    pub fn entity(&self) -> Entity {
        Entity::new_from_existing_raw(self.world, self.component_ref.entity)
    }
}
