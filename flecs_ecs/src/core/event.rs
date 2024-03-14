use std::ffi::c_void;

use flecs_ecs_sys::{ecs_emit, ecs_enqueue, ecs_event_desc_t, ecs_get_world};

use super::{ecs_pair, CachedComponentData, Entity, EventBuilder, IdT, TableT};

/// Trait to mark component structs as `EventData` to be used in `EventBuilderTyped`.
/// This is used to set the event data for the event to be emitted
/// this is to ensure that the event data is of the correct type and the component is meant to be used with `EventBuilderTyped`
pub trait EventData {}

/// Event builder trait to implement '`set_event_data`' for untyped and typed `EventBuilder`
pub trait EventBuilderImpl {
    type BuiltType;
    type ConstBuiltType;

    fn get_data(&mut self) -> &mut EventBuilder;
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
    fn add_type_to_emit<C>(&mut self) -> &mut Self
    where
        C: CachedComponentData,
    {
        let data = self.get_data();
        let ids = &mut data.ids;
        let ids_array = &mut data.ids_array;
        ids.array = ids_array.as_mut_ptr();
        unsafe {
            *ids.array.add(ids.count as usize) = C::get_id(data.world.raw_world);
        }
        ids.count += 1;
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
    fn add_id_to_emit(&mut self, id: IdT) -> &mut Self {
        let data = self.get_data();
        let ids = &mut data.ids;
        let ids_array = &mut data.ids_array;
        ids.array = ids_array.as_mut_ptr();
        unsafe {
            *ids.array.add(ids.count as usize) = id;
        }
        ids.count += 1;
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
    fn add_pair_to_emit<C1, C2>(&mut self) -> &mut Self
    where
        C1: CachedComponentData,
        C2: CachedComponentData,
    {
        let world = self.get_data().world.raw_world;
        self.add_id_to_emit(ecs_pair(C1::get_id(world), C2::get_id(world)))
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
    fn add_pair_ids_to_emit(&mut self, first: IdT, second: IdT) -> &mut Self {
        self.add_id_to_emit(ecs_pair(first, second))
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
    fn add_pair_second_id_to_emit<First>(&mut self, second: IdT) -> &mut Self
    where
        First: CachedComponentData,
    {
        let world = self.get_data().world.raw_world;
        self.add_id_to_emit(ecs_pair(First::get_id(world), second))
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
    fn set_entity_to_emit(&mut self, entity: &Entity) -> &mut Self {
        let desc = &mut self.get_data().desc;
        desc.entity = entity.raw_id;
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
    fn set_table_to_emit(&mut self, table: *mut TableT, offset: i32, count: i32) -> &mut Self {
        let desc = &mut self.get_data().desc;
        desc.table = table;
        desc.offset = offset;
        desc.count = count;
        self
    }

    /// Emit the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::emit`
    #[doc(alias = "event_builder_base::emit")]
    fn emit(&mut self) {
        let data = self.get_data();
        let ids = &mut data.ids;
        let ids_array = &mut data.ids_array;
        let desc = &mut data.desc;
        let world = data.world.raw_world;
        ids.array = ids_array.as_mut_ptr();
        desc.ids = ids;
        desc.observable = unsafe { ecs_get_world(world as *const c_void) } as *mut c_void;
        unsafe { ecs_emit(world, desc) };
    }

    fn enqueue(&mut self) {
        let data = self.get_data();
        let ids = &mut data.ids;
        let ids_array = &mut data.ids_array;
        let desc = &mut data.desc;
        let world = data.world.raw_world;
        ids.array = ids_array.as_mut_ptr();
        desc.ids = ids;
        desc.observable = unsafe { ecs_get_world(world as *const c_void) } as *mut c_void;
        unsafe {
            ecs_enqueue(world, desc as *mut ecs_event_desc_t);
        };
    }

    fn set_event_data(&mut self, data: Self::BuiltType) -> &mut Self;
    fn set_event_data_const(&mut self, data: Self::ConstBuiltType) -> &mut Self;
}
