use entity_view::entity_view_helper::*;

use super::entity_id::EntityId;
use crate::core::*;

pub trait EventEntityView<'w>: EntityId + WorldProvider<'w> + Sized {
    /// Emit event for entity.
    ///
    /// # Safety
    /// Caller must ensure that any type associated with `event` is a ZST
    ///
    /// # Arguments
    ///
    /// * event - the event to emit
    ///
    /// # See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue_id()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_view::emit`
    #[doc(alias = "entity_view::emit")]
    unsafe fn emit_id(self, event: impl Into<Entity>) {
        self.world()
            .event_id(event)
            .entity(self.entity_id())
            .emit(&());
    }

    /// Emit event with an immutable payload for entity.
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to emit.
    ///
    /// # See also
    ///
    /// * [`EntityView::emit_id()`]
    /// * [`EntityView::enqueue_id()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_view::emit`
    #[doc(alias = "entity_view::emit")]
    fn emit<T: ComponentId>(self, event: &T) {
        self.world().event().entity(self.entity_id()).emit(event);
    }

    /// Enqueue event for entity.
    ///
    /// # Safety
    ///
    ///
    /// # Arguments
    ///
    /// * event - the event to enqueue
    ///
    /// # See also
    ///
    /// * [`EntityView::emit_id()`]
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    unsafe fn enqueue_id(self, event: impl Into<Entity>) {
        self.world()
            .event_id(event)
            .entity(self.entity_id())
            .enqueue(());
    }

    /// Enqueue event for entity.
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to enqueue.
    ///
    /// # Usage:
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// # let world = World::new();
    /// # let entity = world.entity();
    /// #[derive(Component)]
    /// struct Resize {
    ///     width: i32,
    ///     height: i32,
    /// }
    ///
    /// world.defer(|| {
    ///     entity.enqueue(Resize{width: 10, height: 20});
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::emit_id()`]
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue_id()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    fn enqueue<T: ComponentId>(self, event: T) {
        self.world().event().entity(self.entity_id()).enqueue(event);
    }

    /// Register the callback for the entity observer for empty events.
    ///
    /// The "empty" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func()`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe_entity()`]
    /// * [`EntityView::observe_payload_entity()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    fn observe<C>(self, func: impl FnMut() + 'static) -> Self
    where
        C: ComponentId + TagComponent,
    {
        observe_impl::<C, _>(self.world(), self.entity_id(), func);
        self
    }

    /// Register the callback for the entity observer for empty events with entity parameter.
    ///
    /// The `empty_entity` iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut EntityView)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload_entity()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    fn observe_entity<C>(self, func: impl FnMut(&mut EntityView) + 'static) -> Self
    where
        C: ComponentId + TagComponent,
    {
        observe_entity_impl::<C, _>(self.world(), self.entity_id(), func);
        self
    }

    /// Register the callback for the entity observer for `payload` events.
    ///
    /// The "payload" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut EventData)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe_entity()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload_entity()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    fn observe_payload<C>(self, func: impl FnMut(&C) + 'static) -> Self
    where
        C: ComponentId + DataComponent,
    {
        observe_payload_impl::<C, _>(self.world(), self.entity_id(), func);
        self
    }

    /// Register the callback for the entity observer for an event with payload and entity parameter.
    ///
    /// The "payload" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut EntityView, &mut EventData)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe_entity()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    fn observe_payload_entity<C>(self, func: impl FnMut(&mut EntityView, &C) + 'static) -> Self
    where
        C: ComponentId + DataComponent,
    {
        observe_payload_entity_impl::<C, _>(self.world(), self.entity_id(), func);
        self
    }
}
