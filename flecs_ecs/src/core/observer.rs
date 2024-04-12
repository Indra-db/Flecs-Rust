use std::{ops::Deref, os::raw::c_void, ptr::NonNull};

use crate::core::*;
use crate::sys;

#[derive(Clone)]
pub struct Observer<'a> {
    pub entity: EntityView<'a>,
    world: WorldRef<'a>,
}

impl<'a> Deref for Observer<'a> {
    type Target = EntityView<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a> Observer<'a> {
    //TODO in query etc desc is a pointer, does it need to be?
    /// Create a new observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::observer`
    #[doc(alias = "observer::observer")]
    pub fn new(
        world: impl IntoWorld<'a>,
        mut desc: sys::ecs_observer_desc_t,
        is_instanced: bool,
    ) -> Self {
        if !desc.filter.instanced {
            desc.filter.instanced = is_instanced;
        }

        let id = unsafe { sys::ecs_observer_init(world.world_ptr_mut(), &desc) };
        let entity = EntityView::new_from(world.world(), id);

        unsafe {
            if !desc.filter.terms_buffer.is_null() {
                if let Some(free_func) = sys::ecs_os_api.free_ {
                    free_func(desc.filter.terms_buffer as *mut _);
                }
            }
        }

        Self {
            entity,
            world: world.world(),
        }
    }

    /// Wrap an existing observer entity in an observer object
    pub fn new_from_existing(world: impl IntoWorld<'a>, observer_entity: EntityView<'a>) -> Self {
        Self {
            world: world.world(),
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
        let desc: sys::ecs_observer_desc_t = sys::ecs_observer_desc_t {
            entity: *self.id,
            ctx: context,
            ..Default::default()
        };

        unsafe {
            sys::ecs_observer_init(self.world.world_ptr_mut(), &desc);
        }
    }

    /// Get the context for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::ctx`
    #[doc(alias = "observer::ctx")]
    pub fn context(&self) -> *mut c_void {
        unsafe { sys::ecs_observer_get_ctx(self.world.world_ptr_mut(), *self.id) }
    }

    /// Get the filter for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::query`
    #[doc(alias = "observer::query")]
    pub fn query(&mut self) -> Filter<()> {
        let poly: *const Poly = self.target_for_pair_first::<Poly>(ECS_OBSERVER);
        let obj: *mut sys::ecs_observer_t = unsafe { (*poly).poly as *mut sys::ecs_observer_t };
        unsafe {
            Filter::<()>::new_ownership(self.world, NonNull::new_unchecked(&mut (*obj).filter))
        }
    }
}
