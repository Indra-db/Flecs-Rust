use std::{
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

use crate::core::*;
use crate::sys;

pub struct EventBuilder<'a> {
    /// non-owning world reference
    pub world: WorldRef<'a>,
    pub(crate) desc: sys::ecs_event_desc_t,
    pub(crate) ids: TypeT,
    pub(crate) ids_array: [IdT; sys::FLECS_EVENT_DESC_MAX as usize],
}

impl<'a> EventBuilder<'a> {
    /// Create a new (untyped) `EventBuilder`
    ///
    /// # Safety
    /// Caller must ensure either that `event` represents a ZST
    /// or the event data is set to point to the appropriate type
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the `EventBuilder` in
    /// * `event` - The event to create the `EventBuilder` for
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::event_builder_base`
    #[doc(alias = "event_builder_base::event_builder_base")]
    pub unsafe fn new(world: impl IntoWorld<'a>, event: impl IntoEntity) -> Self {
        let mut obj = Self {
            world: world.world(),
            desc: Default::default(),
            ids: Default::default(),
            ids_array: Default::default(),
        };
        obj.desc.event = event.get_id();
        obj
    }
}

impl<'a> EventBuilderImpl<'a> for EventBuilder<'a> {
    type BuiltType = *mut c_void;

    fn get_data(&mut self) -> &mut EventBuilder<'a> {
        self
    }

    /// Set the event data for the event
    ///
    /// # Arguments
    ///
    /// * `data` - The data to set for the event which is type-erased of type `*mut c_void`
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::ctx`
    #[doc(alias = "event_builder_base::ctx")]
    fn set_event_data(&mut self, data: Self::BuiltType) -> &mut Self {
        self.desc.param = data as *mut c_void;
        self
    }
}

/// A strongly-typed interface wrapper around `EventBuilder` for constructing events with specific data.
///
/// # Type parameters
///
/// * `T` - The type of the event data to set for the event, which must implement `EventData` and `ComponentId`.
/// `EventData` is a trait used to mark components compatible with events to be used as event data.
/// Ensures the use of appropriate data types for events, enhancing type safety and data integrity.
/// This design aims to prevent the utilization of incompatible components as event data,
/// thereby ensuring greater explicitness and correctness in event handling.
pub struct EventBuilderTyped<'a, T: ComponentId> {
    pub(crate) builder: EventBuilder<'a>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: ComponentId> EventBuilderTyped<'a, T> {
    /// Create a new typed `EventBuilder`
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the `EventBuilder` in
    /// * `event` - The event to create the `EventBuilder` for
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_typed::event_builder_typed`
    #[doc(alias = "event_builder_typed::event_builder_typed")]
    pub fn new(world: impl IntoWorld<'a>) -> Self {
        Self {
            builder: unsafe { EventBuilder::new(world.world(), T::get_id(world)) },
            _phantom: std::marker::PhantomData,
        }
    }
}

/// The `Deref` trait is implemented to allow `EventBuilderTyped` instances to be treated as
/// references to `EventBuilder`. This enables the use of `EventBuilder` methods directly on
/// `EventBuilderTyped` instances.
impl<'a, T: ComponentId> Deref for EventBuilderTyped<'a, T> {
    type Target = EventBuilder<'a>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

/// See `Deref` trait for more information.
impl<'a, T: ComponentId> DerefMut for EventBuilderTyped<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

impl<'a, T: ComponentId> EventBuilderImpl<'a> for EventBuilderTyped<'a, T>
where
    T: ComponentId,
{
    type BuiltType = &'a T;

    fn get_data(&mut self) -> &mut EventBuilder<'a> {
        &mut self.builder
    }
    /// Set the event data for the event
    ///
    /// # Arguments
    ///
    /// * `data` - The data to set for the event which is specific to the type `T`
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_typed::ctx`
    #[doc(alias = "event_builder_typed::ctx")]
    fn set_event_data(&mut self, data: Self::BuiltType) -> &mut Self {
        self.desc.param = data as *const T as *mut c_void;
        self
    }
}
