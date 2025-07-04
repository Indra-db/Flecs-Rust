//! API for emitting events that trigger [`Observer`]s.

use core::marker::PhantomData;
use core::{alloc::Layout, ffi::c_void};

use crate::core::*;
use crate::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::{alloc::dealloc, boxed::Box};

/// A strongly-typed or dynamic interface wrapper for constructing events with specific data.
///
/// # Type parameters
///
/// * `T` - The type of the event data to set for the event, which must implement `ComponentId`.
///
/// Ensures the use of appropriate data types for events, enhancing type safety and data integrity.
/// This design aims to prevent the utilization of incompatible components as event data,
/// thereby ensuring greater explicitness and correctness in event handling.
///
/// # See also
///
/// * [`World::event()`]
/// * [`World::event_id()`]
pub struct EventBuilder<'a, T = ()> {
    pub world: WorldRef<'a>,
    pub(crate) desc: sys::ecs_event_desc_t,
    pub(crate) ids: sys::ecs_type_t,
    pub(crate) ids_array: [sys::ecs_id_t; sys::FLECS_EVENT_DESC_MAX as usize],
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: ComponentId> EventBuilder<'a, T> {
    /// Create a new typed [`EventBuilder`].
    ///
    /// # See also
    ///
    /// * [`EventBuilder::new_untyped()`]
    pub(crate) fn new(world: impl WorldProvider<'a>) -> Self {
        let mut obj = Self {
            world: world.world(),
            desc: Default::default(),
            ids: Default::default(),
            ids_array: Default::default(),
            _phantom: PhantomData,
        };
        obj.desc.event = T::UnderlyingType::entity_id(world);
        obj
    }

    /// Create a new (untyped) [`EventBuilder`].
    ///
    /// # Safety
    ///
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
    /// * [`EventBuilder::new()`]
    pub(crate) fn new_untyped(
        world: impl WorldProvider<'a>,
        event: impl Into<Entity>,
    ) -> EventBuilder<'a, ()> {
        let mut obj = EventBuilder::<'a, ()> {
            world: world.world(),
            desc: Default::default(),
            ids: Default::default(),
            ids_array: Default::default(),
            _phantom: PhantomData::<()>,
        };
        obj.desc.event = *event.into();
        obj
    }

    /// Add component id or pair to emit for the event.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component to add to the event
    pub fn add(&mut self, id: impl IntoId) -> &mut Self {
        let id = *id.into_id(self.world);
        let ids = &mut self.ids;
        let ids_array = &mut self.ids_array;
        ids.array = ids_array.as_mut_ptr();
        unsafe {
            *ids.array.add(ids.count as usize) = id;
        }
        ids.count += 1;
        self
    }

    pub fn add_enum<C: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &mut self,
        enum_value: C,
    ) -> &mut Self {
        let world = self.world;
        let rel = T::entity_id(world);
        // SAFETY: we know that the enum_value is a valid because of the T::id call
        let target = unsafe { enum_value.id_variant_unchecked(world) };
        ecs_assert!(
            target != 0,
            FlecsErrorCode::InvalidParameter,
            "Component was not found in reflection data."
        );
        self.add((rel, target))
    }

    /// Set the target entity to emit for the event.
    ///
    /// # Arguments
    ///
    /// * `entity` - The target entity to emit for the event
    pub fn entity(&mut self, entity: impl Into<Entity>) -> &mut Self {
        let desc = &mut self.desc;
        desc.entity = *entity.into();
        self
    }

    /// Set the table to emit for the event.
    ///
    /// # Arguments
    ///
    /// * `table` - The table to emit for the event
    /// * `offset` - The offset tof the table to emit for the event
    /// * `count` - The count of the table to emit for the event
    pub fn table(&mut self, table: impl IntoTable, offset: i32, count: i32) -> &mut Self {
        let desc = &mut self.desc;
        desc.table = table.table_ptr_mut();
        desc.offset = offset;
        desc.count = count;
        self
    }

    pub fn emit(&mut self, data: &T) {
        let ids = &mut self.ids;
        let ids_array = &mut self.ids_array;
        let desc = &mut self.desc;
        let world = self.world;
        ids.array = ids_array.as_mut_ptr();

        if !T::IS_TAG {
            desc.const_param = data as *const T as *const c_void;
        }

        desc.ids = ids;
        desc.observable = world.real_world().world_ptr_mut() as *mut c_void;
        unsafe { sys::ecs_emit(world.world_ptr_mut(), desc) };
    }

    pub fn enqueue(&mut self, data: T) {
        let ids = &mut self.ids;
        let ids_array = &mut self.ids_array;
        let desc = &mut self.desc;
        let world = self.world;
        ids.array = ids_array.as_mut_ptr();

        if !T::IS_TAG {
            desc.param = Box::leak(Box::new(data)) as *mut T as *mut c_void;
        }

        desc.ids = ids;
        desc.observable = world.real_world().world_ptr_mut() as *mut c_void;
        unsafe {
            sys::ecs_enqueue(world.world_ptr_mut(), desc);
            if !T::IS_TAG {
                dealloc(desc.param as *mut u8, Layout::new::<T>());
            }
        };
    }
}
