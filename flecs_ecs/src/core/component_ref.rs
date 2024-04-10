//! Refs are a fast mechanism for referring to a specific entity/component

use std::{marker::PhantomData, os::raw::c_void, ptr::NonNull};

use super::{
    c_types::{IdT, RefT, WorldT},
    component_registration::ComponentId,
    entity::Entity,
    IntoEntityId, IntoWorld,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;
use crate::{
    core::WorldRef,
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
    ///
    #[doc(alias = "ref::ref")]
    pub fn new(world: impl IntoWorld<'a>, entity: impl IntoEntityId, mut id: IdT) -> Self {
        // the world we were called with may be a stage; convert it to a world
        // here if that is the case
        let world_ptr =
            unsafe { ecs_get_world(world.world_ptr_mut() as *const c_void) as *mut WorldT };

        if id == 0 {
            id = T::get_id(world);
        }

        ecs_assert!(
            std::mem::size_of::<T>() != 0,
            FlecsErrorCode::InvalidParameter
        );

        let component_ref = unsafe { ecs_ref_init_id(world_ptr, entity.get_id(), id) };
        assert_ne!(
            component_ref.entity, 0,
            "Tried to create invalid `Ref` type."
        );
        Ref {
            world: unsafe { WorldRef::from_ptr(world_ptr) },
            component_ref,
            _marker: PhantomData,
        }
    }

    /// Try to get component from ref.
    ///
    /// # See also
    ///
    /// * C++ API: `ref::try_get`
    #[doc(alias = "ref::try_get")]
    pub fn try_get(&mut self) -> Option<&mut T> {
        NonNull::new(unsafe {
            ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            ) as *mut T
        })
        .map(|mut t| unsafe { t.as_mut() })
    }

    pub fn get(&mut self) -> &mut T {
        self.try_get()
            .expect("Called Ref::get but the Ref was invalid")
    }

    pub fn entity(&self) -> Entity<'a> {
        Entity::new_from_existing(self.world, self.component_ref.entity)
    }
}
