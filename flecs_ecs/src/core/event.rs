use std::ffi::c_void;

use flecs_ecs_sys::{ecs_emit, ecs_enqueue, ecs_event_desc_t};

use super::{
    ecs_pair, ComponentId, EventBuilder, IntoComponentId, IntoEntityId, IntoEntityIdExt, IntoTable,
    IntoWorld,
};

/// Event builder trait to implement '`set_event_data`' for untyped and typed `EventBuilder`
pub trait EventBuilderImpl<'a> {
    type BuiltType;

    fn get_data(&mut self) -> &mut EventBuilder<'a>;

    /// Add component id or pair to emit for the event
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component to add to the event
    ///
    /// # See also
    ///
    /// * C++ API: `event_builder_base::id`
    #[doc(alias = "event_builder_base::id")]
    fn add_id(&mut self, id: impl IntoEntityIdExt) -> &mut Self {
        let id = id.get_id();
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
    fn add<T>(&mut self) -> &mut Self
    where
        T: IntoComponentId,
    {
        let world = self.get_data().world;
        self.add_id(T::get_id(world))
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
    fn add_pair_first_to_emit<First>(&mut self, second: impl IntoEntityId) -> &mut Self
    where
        First: ComponentId,
    {
        let world = self.get_data().world;
        self.add_id(ecs_pair(First::get_id(world), second))
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
    fn set_entity_to_emit(&mut self, entity: impl IntoEntityId) -> &mut Self {
        let desc = &mut self.get_data().desc;
        desc.entity = entity.get_id();
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
    fn set_table_to_emit(&mut self, table: impl IntoTable, offset: i32, count: i32) -> &mut Self {
        let desc = &mut self.get_data().desc;
        desc.table = table.table_ptr_mut();
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
        let world = data.world;
        ids.array = ids_array.as_mut_ptr();
        desc.ids = ids;
        desc.observable = world.real_world().world_ptr_mut() as *mut c_void;
        unsafe { ecs_emit(world.world_ptr_mut(), desc) };
    }

    fn enqueue(&mut self) {
        let data = self.get_data();
        let ids = &mut data.ids;
        let ids_array = &mut data.ids_array;
        let desc = &mut data.desc;
        let world = data.world;
        ids.array = ids_array.as_mut_ptr();
        desc.ids = ids;
        desc.observable = world.real_world().world_ptr_mut() as *mut c_void;
        unsafe {
            ecs_enqueue(world.world_ptr_mut(), desc as *mut ecs_event_desc_t);
        };
    }

    fn set_event_data(&mut self, data: Self::BuiltType) -> &mut Self;
}
