use std::ptr::NonNull;
use std::{ops::Deref, os::raw::c_void};

use crate::core::*;
use crate::sys;

/// `ObserverBuilder` is used to configure and build Observers.
/// Observers are systems that react to events.
/// Observers let applications register callbacks for ECS events.
#[derive(Clone)]
pub struct Observer<'a> {
    entity: EntityView<'a>,
}

impl<'a> Deref for Observer<'a> {
    type Target = EntityView<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a> Observer<'a> {
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
        if desc.query.flags & sys::EcsQueryIsInstanced == 0 {
            ecs_bit_cond(
                &mut desc.query.flags,
                sys::EcsQueryIsInstanced,
                is_instanced,
            );
        }

        let id = unsafe { sys::ecs_observer_init(world.world_ptr_mut(), &desc) };
        let entity = EntityView::new_from(world.world(), id);

        Self { entity }
    }

    /// Wrap an existing observer entity in an observer object
    pub fn new_from_existing(observer_entity: EntityView<'a>) -> Self {
        Self {
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

    /// Get the query for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::query`
    #[doc(alias = "observer::query")]
    pub fn query(&mut self) -> Query<()> {
        unsafe {
            Query::<()>::new_from(NonNull::new_unchecked(sys::ecs_observer_get_query(
                self.world_ptr(),
                *self.entity.id(),
            )
                as *mut sys::ecs_query_t))
        }
    }
}
