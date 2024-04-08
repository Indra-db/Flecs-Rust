//! Refs are a fast mechanism for referring to a specific entity/component

use std::{marker::PhantomData, os::raw::c_void};

use super::{
    c_types::{IdT, RefT, WorldT},
    component_registration::ComponentId,
    entity::Entity,
    IntoEntityId, IntoWorld, WorldRef,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;
use crate::{
    ecs_assert,
    sys::{ecs_get_world, ecs_ref_get_id, ecs_ref_init_id},
};

/// A reference to a component from a specific entity.
/// Refs are a fast mechanism for referring to a specific entity/component
pub struct Ref<'a, T: ComponentId> {
    world: WorldRef<'a>,
    component_ref: RefT,
    _marker: PhantomData<T>,
}

impl<'a, T: ComponentId> Ref<'a, T> {
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
    pub fn new(world: impl IntoWorld<'a>, entity: impl IntoEntityId, mut id: IdT) -> Self {
        let mut world_ptr = world.world_ptr_mut();
        // the world we were called with may be a stage; convert it to a world
        // here if that is the case
        world_ptr = if !world_ptr.is_null() {
            unsafe { ecs_get_world(world_ptr as *const c_void) as *mut WorldT }
        } else {
            std::ptr::null_mut()
        };

        if id == 0 {
            id = T::get_id(world.world_ref());
        }

        ecs_assert!(
            std::mem::size_of::<T>() != 0,
            FlecsErrorCode::InvalidParameter
        );

        let component_ref = unsafe { ecs_ref_init_id(world_ptr, entity.get_id(), id) };

        Ref {
            world: world.world_ref(),
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
            &mut *(ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            ) as *mut T)
        }
    }

    /// Try to get component from ref.
    ///
    /// # See also
    ///
    /// * C++ API: `ref::try_get`
    #[doc(alias = "ref::try_get")]
    pub fn get(&mut self) -> Option<&mut T> {
        if self.world.world_ptr_mut().is_null() || self.component_ref.entity == 0 {
            return None;
        }
        Some(self.get_unchecked())
    }

    pub fn entity(&self) -> Entity {
        Entity::new_from_existing_raw(self.world, self.component_ref.entity)
    }
}
