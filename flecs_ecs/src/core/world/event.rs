use super::*;

/// Event mixin implementation
impl World {
    /// Create a new event builder (untyped) from entity id which represents an event
    ///
    /// # Safety
    /// Caller must ensure that `event` is a ZST or that a pointer to the associated type is set on the builder
    ///
    /// # Arguments
    ///
    /// * `event` - The event id
    ///
    /// # Returns
    ///
    /// A new (untyped) event builder.
    ///
    /// # See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`World::event()`]
    pub unsafe fn event_id(&self, event: impl Into<Entity>) -> EventBuilder<'_, ()> {
        EventBuilder::<()>::new_untyped(self, event)
    }

    /// Create a new event.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The event type.
    ///
    /// # Returns
    ///
    /// A new (typed) event builder.
    ///
    /// # See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`World::event_id()`]
    pub fn event<T: ComponentId>(&self) -> EventBuilder<'_, T> {
        EventBuilder::<T>::new(self)
    }
}
