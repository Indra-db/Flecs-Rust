//! Observers are systems that react to events. Observers let applications register callbacks for ECS events.
use std::ops::DerefMut;
use std::ptr::NonNull;
use std::{ops::Deref, os::raw::c_void};

use crate::core::*;
use crate::sys;

/// Observers are systems that react to events.
/// Observers let applications register callbacks for ECS events.
///
/// These are typically constructed via [`World::observer()`].
#[derive(Clone, Copy)]
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

impl<'a> DerefMut for Observer<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<'a> WorldProvider<'a> for Observer<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> Observer<'a> {
    /// Create a new observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::observer`
    #[doc(alias = "observer::observer")]
    pub(crate) fn new(world: impl WorldProvider<'a>, desc: sys::ecs_observer_desc_t) -> Self {
        for event in desc.events {
            if event == flecs::OnAdd::ID {
                for term in desc.query.terms {
                    if (term.first.id | term.id | term.second.id | term.src.id) == 0 {
                        break;
                    }

                    if term.inout != sys::ecs_inout_kind_t_EcsInOutFilter as i16
                        && term.inout != sys::ecs_inout_kind_t_EcsInOutNone as i16
                    {
                        panic!(
                            r#"
                            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                            Cannot create typed observers with the `OnAdd` event or when `InOut` is set to `In` or `InOut`. 
                            This situation occurs when using `&` or `&mut` with `.with`.
                        
                            Accessing the uninitialized value of a component is undefined behavior in Rust.
                            Instead, use `.with::<T>` to add the component you want to observe, without passing the type directly.
                        
                            For example:
                            ```
                            .observer::<flecs::OnAdd, &Position>()
                            ```
                            should be written as:
                            ```
                            .observer::<flecs::OnAdd, ()>()
                            .with::<Position>() // Note: no `&` or `&mut` here! 
                            ```
                        
                            Invalid signatures include:
                            ```
                            .observer::<flecs::OnAdd, &T>()
                            .observer::<flecs::OnAdd, &mut T>()
                            .observer::<flecs::OnAdd, ()>().with::<&T>()
                            .observer::<flecs::OnAdd, ()>().with::<&mut T>()
                            ```
                            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                            "#
                        );
                    }
                }
            }
        }
        let id = unsafe { sys::ecs_observer_init(world.world_ptr_mut(), &desc) };
        let entity = EntityView::new_from(world.world(), id);

        Self { entity }
    }

    /// Wrap an existing observer entity in an observer object
    pub(crate) fn new_from_existing(observer_entity: EntityView<'a>) -> Self {
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
        unsafe { (*sys::ecs_observer_get(self.world.world_ptr_mut(), *self.id)).ctx }
    }

    /// Get the query for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::query`
    #[doc(alias = "observer::query")]
    pub fn query(&mut self) -> Query<()> {
        unsafe {
            Query::<()>::new_from(NonNull::new_unchecked(
                (*sys::ecs_observer_get(self.world_ptr(), *self.entity.id())).query,
            ))
        }
    }

    /// Get the observer's entity
    ///
    /// # See also
    ///
    /// * C++ API: `observer::entity`
    #[doc(alias = "observer::entity")]
    pub fn entity(&self) -> EntityView<'a> {
        self.entity
    }
}
