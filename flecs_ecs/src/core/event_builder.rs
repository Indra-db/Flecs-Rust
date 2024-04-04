use std::{
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

use crate::sys::{ecs_event_desc_t, FLECS_EVENT_DESC_MAX};

use super::{
    c_types::{IdT, TypeT},
    component_registration::ComponentId,
    event::{EventBuilderImpl, EventData},
    world::World,
    IntoEntityId,
};

pub struct EventBuilder {
    /// non-owning world reference
    pub world: World,
    pub(crate) desc: ecs_event_desc_t,
    pub(crate) ids: TypeT,
    pub(crate) ids_array: [IdT; FLECS_EVENT_DESC_MAX as usize],
}

impl EventBuilder {
    /// Create a new (untyped) `EventBuilder`
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
    pub fn new(world: &World, event: impl IntoEntityId) -> Self {
        let mut obj = Self {
            world: world.clone(),
            desc: Default::default(),
            ids: Default::default(),
            ids_array: Default::default(),
        };
        obj.desc.event = event.get_id();
        obj
    }
}

impl EventBuilderImpl for EventBuilder {
    type BuiltType = *mut c_void;
    type ConstBuiltType = *const c_void;

    fn get_data(&mut self) -> &mut EventBuilder {
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

    /// Set the const event data for the event
    ///
    /// # Arguments
    ///
    /// * `data` - The data to set for the event which is type-erased of type `*const c_void`
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::ctx`
    #[doc(alias = "event_builder_base::ctx")]
    fn set_event_data_const(&mut self, data: Self::ConstBuiltType) -> &mut Self {
        self.desc.const_param = data as *const c_void;
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
pub struct EventBuilderTyped<'a, T: EventData + ComponentId> {
    pub(crate) builder: EventBuilder,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T: EventData + ComponentId> EventBuilderTyped<'a, T> {
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
    pub fn new(world: &World, event: impl IntoEntityId) -> Self {
        Self {
            builder: EventBuilder::new(world, event),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// The `Deref` trait is implemented to allow `EventBuilderTyped` instances to be treated as
/// references to `EventBuilder`. This enables the use of `EventBuilder` methods directly on
/// `EventBuilderTyped` instances.
impl<'a, T: EventData + ComponentId> Deref for EventBuilderTyped<'a, T> {
    type Target = EventBuilder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

/// See `Deref` trait for more information.
impl<'a, T: EventData + ComponentId> DerefMut for EventBuilderTyped<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

impl<'a, T: EventData + ComponentId> EventBuilderImpl for EventBuilderTyped<'a, T>
where
    T: EventData + ComponentId,
{
    type BuiltType = &'a mut T;
    type ConstBuiltType = &'a T;

    fn get_data(&mut self) -> &mut EventBuilder {
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
        self.desc.param = data as *mut T as *mut c_void;
        self
    }

    fn set_event_data_const(&mut self, data: Self::ConstBuiltType) -> &mut Self {
        self.desc.const_param = data as *const T as *const c_void;
        self
    }
}
