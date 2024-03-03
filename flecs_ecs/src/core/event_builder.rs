use std::{
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

use crate::{
    core::{
        c_binding::bindings::{ecs_emit, ecs_get_world},
    },
};

use super::{
    c_binding::{
        bindings::{ecs_event_desc_t, FLECS_EVENT_DESC_MAX},
        ecs_enqueue,
    },
    c_types::{EntityT, IdT, TableT, TypeT},
    component_registration::CachedComponentData,
    ecs_pair,
    event::{EventBuilderImpl, EventData},
    world::World,
};

pub struct EventBuilder {
    /// non-owning world reference
    world: World,
    desc: ecs_event_desc_t,
    ids: TypeT,
    ids_array: [IdT; FLECS_EVENT_DESC_MAX as usize],
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
    pub fn new(world: &World, event: EntityT) -> Self {
        let mut obj = Self {
            world: world.clone(),
            desc: Default::default(),
            ids: Default::default(),
            ids_array: Default::default(),
        };
        obj.desc.event = event;
        obj
    }

    /// Add component to emit for the event
    ///
    /// # Type parameters
    ///
    /// * `C` - The component to add to the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::id`
    #[doc(alias = "event_builder_base::id")]
    pub fn add_component_to_emit<C>(&mut self) -> &mut Self
    where
        C: CachedComponentData,
    {
        self.ids.array = self.ids_array.as_mut_ptr();
        unsafe {
            *self.ids.array.add(self.ids.count as usize) = C::get_id(self.world.raw_world);
        }
        self.ids.count += 1;
        self
    }

    /// Add component id to emit for the event
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component to add to the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::id`
    #[doc(alias = "event_builder_base::id")]
    pub fn add_component_id_to_emit(&mut self, id: IdT) -> &mut Self {
        self.ids.array = self.ids_array.as_mut_ptr();
        unsafe {
            *self.ids.array.add(self.ids.count as usize) = id;
        }
        self.ids.count += 1;
        self
    }

    /// Add a pair of components to emit for the event
    ///
    /// # Type parameters
    ///
    /// * `C1` - The first component to add to the event
    /// * `C2` - The second component to add to the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::id`
    #[doc(alias = "event_builder_base::id")]
    pub fn add_pair_to_emit<C1, C2>(&mut self) -> &mut Self
    where
        C1: CachedComponentData,
        C2: CachedComponentData,
    {
        self.add_component_id_to_emit(ecs_pair(
            C1::get_id(self.world.raw_world),
            C2::get_id(self.world.raw_world),
        ))
    }

    /// Add a pair of component ids to emit for the event
    ///
    /// # Arguments
    ///
    /// * `first` - The id of the first component to add to the event
    /// * `second` - The id of the second component to add to the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::id`
    #[doc(alias = "event_builder_base::id")]
    pub fn add_pair_ids_to_emit(&mut self, first: IdT, second: IdT) -> &mut Self {
        self.add_component_id_to_emit(ecs_pair(first, second))
    }

    /// Add a pair of components to emit for the event
    ///
    /// # Type parameters
    ///
    /// * `First` - The first component to add to the event
    ///
    /// # Arguments
    ///
    /// * `second` - The id of the second component to add to the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::id`
    #[doc(alias = "event_builder_base::id")]
    pub fn add_pair_second_id_to_emit<First>(&mut self, second: IdT) -> &mut Self
    where
        First: CachedComponentData,
    {
        self.add_component_id_to_emit(ecs_pair(First::get_id(self.world.raw_world), second))
    }

    /// Set the entity to emit for the event
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to emit for the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::entity`
    #[doc(alias = "event_builder_base::entity")]
    pub fn set_entity_to_emit(&mut self, entity: EntityT) -> &mut Self {
        self.desc.entity = entity;
        self
    }

    /// Set the table to emit for the event
    ///
    /// # Arguments
    ///
    /// * `table` - The table to emit for the event
    /// * `offset` - The offset tof the table to emit for the event
    /// * `count` - The count of the table to emit for the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::table`
    #[doc(alias = "event_builder_base::table")]
    pub fn set_table_to_emit(&mut self, table: *mut TableT, offset: i32, count: i32) -> &mut Self {
        self.desc.table = table;
        self.desc.offset = offset;
        self.desc.count = count;
        self
    }

    /// Emit the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::emit`
    #[doc(alias = "event_builder_base::emit")]
    pub fn emit(&mut self) {
        self.ids.array = self.ids_array.as_mut_ptr();
        self.desc.ids = &self.ids;
        self.desc.observable =
            unsafe { ecs_get_world(self.world.raw_world as *const c_void) } as *mut c_void;
        unsafe { ecs_emit(self.world.raw_world, &mut self.desc) };
    }

    pub fn enqueue(&mut self) {
        self.ids.array = self.ids_array.as_mut_ptr();
        self.desc.ids = &self.ids;
        self.desc.observable =
            unsafe { ecs_get_world(self.world.raw_world as *const c_void) } as *mut c_void;
        unsafe {
            ecs_enqueue(
                self.world.raw_world,
                &mut self.desc as *mut ecs_event_desc_t,
            );
        };
    }
}

impl EventBuilderImpl for EventBuilder {
    type BuiltType = *mut c_void;
    type ConstBuiltType = *const c_void;

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
/// * `T` - The type of the event data to set for the event, which must implement `EventData` and `CachedComponentData`.
/// `EventData` is a trait used to mark components compatible with events to be used as event data.
/// Ensures the use of appropriate data types for events, enhancing type safety and data integrity.
/// This design aims to prevent the utilization of incompatible components as event data,
/// thereby ensuring greater explicitness and correctness in event handling.
pub struct EventBuilderTyped<'a, T: EventData + CachedComponentData> {
    builder: EventBuilder,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T: EventData + CachedComponentData> EventBuilderTyped<'a, T> {
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
    pub fn new(world: &World, event: EntityT) -> Self {
        Self {
            builder: EventBuilder::new(world, event),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// The `Deref` trait is implemented to allow `EventBuilderTyped` instances to be treated as
/// references to `EventBuilder`. This enables the use of `EventBuilder` methods directly on
/// `EventBuilderTyped` instances.
impl<'a, T: EventData + CachedComponentData> Deref for EventBuilderTyped<'a, T> {
    type Target = EventBuilder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

/// See `Deref` trait for more information.
impl<'a, T: EventData + CachedComponentData> DerefMut for EventBuilderTyped<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

impl<'a, T: EventData + CachedComponentData> EventBuilderImpl for EventBuilderTyped<'a, T>
where
    T: EventData + CachedComponentData,
{
    type BuiltType = &'a mut T;
    type ConstBuiltType = &'a T;

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
