use std::{
    ops::{Deref, DerefMut},
    os::raw::c_void,
    ptr::NonNull,
};

use crate::core::*;
use crate::sys;

pub struct EventBuilderUntyped<'a> {
    /// non-owning world reference
    pub world: WorldRef<'a>,
    pub(crate) desc: sys::ecs_event_desc_t,
    pub(crate) ids: TypeT,
    pub(crate) ids_array: [IdT; sys::FLECS_EVENT_DESC_MAX as usize],
}

impl<'a> EventBuilderUntyped<'a> {
    /// Create a new (untyped) `EventBuilderUntyped`
    ///
    /// # Safety
    /// Caller must ensure either that `event` represents a ZST
    /// or the event data is set to point to the appropriate type
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the `EventBuilderUntyped` in
    /// * `event` - The event to create the `EventBuilderUntyped` for
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::event_builder_base`
    #[doc(alias = "event_builder_base::event_builder_base")]
    pub fn new(world: impl IntoWorld<'a>, event: impl Into<Entity>) -> Self {
        let mut obj = Self {
            world: world.world(),
            desc: Default::default(),
            ids: Default::default(),
            ids_array: Default::default(),
        };
        obj.desc.event = *event.into();
        obj
    }
}

impl<'a> EventBuilderImpl<'a> for EventBuilderUntyped<'a> {
    type Data = NonNull<u8>;

    fn get_data(&mut self) -> &mut EventBuilderUntyped<'a> {
        self
    }

    /// Set the event data for the event
    ///
    /// # Arguments
    ///
    /// * `data` - The data to set for the event which is type-erased
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::ctx`
    #[doc(alias = "event_builder_base::ctx")]
    fn set_const_event_data(&mut self, data: Self::Data) -> &mut Self {
        self.desc.const_param = data.as_ptr() as *const c_void;
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
    fn set_event_data(&mut self, data: Self::Data) -> &mut Self {
        self.desc.param = data.as_ptr() as *mut c_void;
        self
    }
}

/// A strongly-typed interface wrapper around `EventBuilderUntyped` for constructing events with specific data.
///
/// # Type parameters
///
/// * `T` - The type of the event data to set for the event, which must implement `ComponentId`.
///
/// Ensures the use of appropriate data types for events, enhancing type safety and data integrity.
/// This design aims to prevent the utilization of incompatible components as event data,
/// thereby ensuring greater explicitness and correctness in event handling.
pub struct EventBuilder<'a, T: ComponentId> {
    pub(crate) builder: EventBuilderUntyped<'a>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: ComponentId> EventBuilder<'a, T> {
    /// Create a new typed `EventBuilderUntyped`
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the `EventBuilderUntyped` in
    /// * `event` - The event to create the `EventBuilderUntyped` for
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_typed::event_builder_typed`
    #[doc(alias = "event_builder_typed::event_builder_typed")]
    pub fn new(world: impl IntoWorld<'a>) -> Self {
        Self {
            builder: EventBuilderUntyped::new(world.world(), T::get_id(world)),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// The `Deref` trait is implemented to allow `EventBuilder` instances to be treated as
/// references to `EventBuilderUntyped`. This enables the use of `EventBuilderUntyped` methods directly on
/// `EventBuilder` instances.
impl<'a, T: ComponentId> Deref for EventBuilder<'a, T> {
    type Target = EventBuilderUntyped<'a>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

/// See `Deref` trait for more information.
impl<'a, T: ComponentId> DerefMut for EventBuilder<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

impl<'a, T: ComponentId> EventBuilderImpl<'a> for EventBuilder<'a, T>
where
    T: ComponentId,
{
    type Data = T;

    fn get_data(&mut self) -> &mut EventBuilderUntyped<'a> {
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
    fn set_const_event_data(&mut self, data: Self::Data) -> &mut Self {
        self.desc.const_param = Box::leak(Box::new(data)) as *const T as *const c_void;
        self
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
    fn set_event_data(&mut self, data: Self::Data) -> &mut Self {
        self.desc.param = Box::leak(Box::new(data)) as *mut T as *mut c_void;
        self
    }
}
