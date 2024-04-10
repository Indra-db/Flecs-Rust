//! Refs are a fast mechanism for referring to a specific entity/component

use crate::core::*;
use crate::sys;
use std::{marker::PhantomData, os::raw::c_void, ptr::NonNull};

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
    pub fn new(world: impl IntoWorld<'a>, entity: impl IntoEntity, mut id: IdT) -> Self {
        // the world we were called with may be a stage; convert it to a world
        // here if that is the case
        let world_ptr =
            unsafe { sys::ecs_get_world(world.world_ptr_mut() as *const c_void) as *mut WorldT };

        if id == 0 {
            id = T::get_id(world);
        }

        ecs_assert!(
            std::mem::size_of::<T>() != 0,
            FlecsErrorCode::InvalidParameter
        );

        let component_ref = unsafe { sys::ecs_ref_init_id(world_ptr, entity.get_id(), id) };
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
            sys::ecs_ref_get_id(
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

    pub fn entity(&self) -> EntityView<'a> {
        EntityView::new_from(self.world, self.component_ref.entity)
    }
}
