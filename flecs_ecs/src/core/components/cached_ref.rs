//! Refs are a fast mechanism for referring to a specific entity/component. It caches data to speedup get operations.

use core::{ffi::c_void, marker::PhantomData, ptr::NonNull};

use crate::core::*;
use crate::sys;

/// A cached reference for fast access to a component from a specific entity.
#[derive(Debug, Clone, Copy)]
pub struct CachedRef<'a, T> {
    pub(crate) world: WorldRef<'a>,
    pub(crate) component_ref: sys::ecs_ref_t,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, T> CachedRef<'a, T> {
    /// Create a new ref to a component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `entity`: the entity to reference.
    /// * `id`: the id of the component to reference.
    pub fn new(
        world: impl WorldProvider<'a>,
        entity: impl Into<Entity>,
        component: impl IntoId,
    ) -> CachedRef<'a, T> {
        let world = world.world();
        // the world we were called with may be a stage; convert it to a world
        // here if that is the case
        let world_ptr = unsafe {
            sys::ecs_get_world(world.world_ptr_mut() as *const c_void) as *mut sys::ecs_world_t
        };

        let id = *component.into_id(world);

        debug_assert!(
            id != 0,
            "Tried to create invalid `CachedRef` type. id is 0."
        );

        const {
            assert!(
                core::mem::size_of::<T>() != 0,
                "Cached Ref cannot be created for zero-sized types / tags."
            );
        }

        // TODO this is done with FLECS_DEBUG flag normally
        debug_assert!(
            {
                let type_ = unsafe { sys::ecs_get_typeid(world_ptr, id) };
                let ti = unsafe { sys::ecs_get_type_info(world_ptr, type_) };
                ti.is_null() || unsafe { (*ti).size } != 0
            },
            "Cannot create ref to empty type"
        );

        let component_ref = unsafe { sys::ecs_ref_init_id(world_ptr, *entity.into(), id) };
        assert_ne!(
            component_ref.entity, 0,
            "Tried to create invalid `CachedRef` type."
        );
        CachedRef::<T> {
            world,
            component_ref,
            _marker: PhantomData,
        }
    }

    /// Return entity associated with reference.
    pub fn entity(&self) -> EntityView<'a> {
        EntityView::new_from(self.world, self.component_ref.entity)
    }

    /// Return component associated with reference.
    pub fn component(&self) -> IdView<'a> {
        IdView::new_from_id(self.world, self.component_ref.id)
    }

    pub fn has(&mut self) -> bool {
        !unsafe {
            sys::ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            )
        }
        .is_null()
    }

    pub fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a, T: ComponentId> CachedRef<'a, T> {
    /// Try to get component from ref.
    pub fn try_get<R>(&mut self, callback: impl FnOnce(&mut T) -> R) -> Option<R> {
        NonNull::new(unsafe {
            sys::ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            ) as *mut T
        })
        .map(|mut t| unsafe { t.as_mut() })
        .map(callback)
    }

    pub fn get<R>(&mut self, callback: impl FnOnce(&mut T) -> R) -> R {
        let mut ref_comp = NonNull::new(unsafe {
            sys::ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            ) as *mut T
        })
        .expect("Component not found, use try_get if you want to handle this case");

        callback(unsafe { ref_comp.as_mut() })
    }
}

impl<'a> CachedRef<'a, core::ffi::c_void> {
    /// Try to get component from ref.
    pub fn try_get<R>(&mut self, callback: impl FnOnce(*mut core::ffi::c_void) -> R) -> Option<R> {
        NonNull::new(unsafe {
            sys::ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            )
        })
        .map(NonNull::as_ptr)
        .map(callback)
    }

    pub fn get<R>(&mut self, callback: impl FnOnce(*mut core::ffi::c_void) -> R) -> R {
        let ref_comp = NonNull::new(unsafe {
            sys::ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            )
        })
        .expect("Component not found, use try_get if you want to handle this case");

        callback(ref_comp.as_ptr())
    }

    /// Try to mutably borrow component from ref.
    pub fn try_borrow_mut(&mut self) -> Option<&mut T> {
        NonNull::new(unsafe {
            sys::ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            ) as *mut T
        })
        .map(|mut t| unsafe { t.as_mut() })
    }

    /// Mutably borrow component from ref.
    ///
    /// # Panics
    ///
    /// Panics if the the ref does not refer to a component.
    pub fn borrow_mut(&mut self) -> &mut T {
        NonNull::new(unsafe {
            sys::ecs_ref_get_id(
                self.world.world_ptr_mut(),
                &mut self.component_ref,
                self.component_ref.id,
            ) as *mut T
        })
        .map(|mut t| unsafe { t.as_mut() })
        .expect("Component not found, use try_get if you want to handle this case")
    }
}
