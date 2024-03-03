//! Refs are a fast mechanism for referring to a specific entity/component

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
/// Refs are a fast mechanism for referring to a specific entity/component
pub struct Ref<T: CachedComponentData> {
    world: *mut WorldT,
    component_ref: RefT,
    _marker: PhantomData<T>,
}

impl<T: CachedComponentData> Ref<T> {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]

    /// Create a new ref to a component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `entity`: the entity to reference.
    /// * `id`: the id of the component to reference.
    ///
    /// # See also
    ///
    /// * C++ API: `ref::ref`
    #[doc(alias = "ref::ref")]
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
    /// Get component from ref.
    ///
    /// # Safety
    ///
    /// This function assumes you know what you are doing and that the component is valid.
    /// Dereferences the component ref without checking.
    ///
    /// # See also
    ///
    /// * C++ API: `ref::get`
    #[doc(alias = "ref::get")]
    pub fn get_unchecked(&mut self) -> &mut T {
        unsafe {
            &mut *(ecs_ref_get_id(self.world, &mut self.component_ref, self.component_ref.id)
                as *mut T)
        }
    }

    /// Try to get component from ref.
    ///
    /// # See also
    ///
    /// * C++ API: `ref::try_get`
    #[doc(alias = "ref::try_get")]
    pub fn get(&mut self) -> Option<&mut T> {
        if self.world.is_null() || self.component_ref.entity == 0 {
            return None;
        }
        Some(self.get_unchecked())
    }

    pub fn entity(&self) -> Entity {
        Entity::new_from_existing_raw(self.world, self.component_ref.entity)
    }
}
