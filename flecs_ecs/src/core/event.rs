use std::ffi::c_void;

use crate::core::*;
use crate::sys;

/// Event builder trait to implement '`set_event_data`' for untyped and typed `EventBuilderUntyped`
pub trait EventBuilderImpl<'a> {
    type Data;

    fn get_data(&mut self) -> &mut EventBuilderUntyped<'a>;

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
    fn add_id(&mut self, id: impl IntoId) -> &mut Self {
        let id = *id.into();
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
    fn add_first_to_emit<First>(&mut self, second: impl Into<Entity>) -> &mut Self
    where
        First: ComponentId,
    {
        let world = self.get_data().world;
        self.add_id(ecs_pair(First::get_id(world), *second.into()))
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
    fn set_entity_to_emit(&mut self, entity: impl Into<Entity>) -> &mut Self {
        let desc = &mut self.get_data().desc;
        desc.entity = *entity.into();
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
        unsafe { sys::ecs_emit(world.world_ptr_mut(), desc) };
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
            sys::ecs_enqueue(world.world_ptr_mut(), desc as *mut sys::ecs_event_desc_t);
        };
    }

    fn set_event_data(&mut self, data: Self::Data) -> &mut Self;
    fn set_const_event_data(&mut self, data: Self::Data) -> &mut Self;
}
