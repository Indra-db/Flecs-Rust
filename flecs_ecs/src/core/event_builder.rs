use std::{
    default,
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

use crate::{
    core::{
        c_binding::bindings::{ecs_emit, ecs_get_world, ECS_INVALID_PARAMETER},
        utility::functions::ecs_record_to_row,
    },
    ecs_assert,
};

use super::{
    c_binding::bindings::{ecs_event_desc_t, ecs_record_find, ecs_record_t, FLECS_EVENT_DESC_MAX},
    c_types::{EntityT, IdT, TableT, TypeT, WorldT},
    component_registration::CachedComponentData,
    event::{EventBuilderImpl, EventData},
    utility::functions::ecs_pair,
    world::World,
};

pub struct EventBuilder {
    world: World,
    desc: ecs_event_desc_t,
    ids: TypeT,
    ids_array: [IdT; FLECS_EVENT_DESC_MAX as usize],
}

impl EventBuilder {
    pub fn new(world: *mut WorldT, event: EntityT) -> Self {
        let mut obj = Self {
            world: World::new_from_world(world),
            desc: Default::default(),
            ids: Default::default(),
            ids_array: Default::default(),
        };
        obj.desc.event = event;
        obj
    }

    pub fn add_component_to_emit<C>(&mut self) -> &mut Self
    where
        C: CachedComponentData,
    {
        self.ids.array = self.ids_array.as_mut_ptr();
        unsafe {
            *self.ids.array.add(self.ids.count as usize) = C::get_id(self.world.raw_world);
        }
        self.ids.count = self.ids.count + 1;
        self
    }

    pub fn add_component_id_to_emit(&mut self, id: IdT) -> &mut Self {
        self.ids.array = self.ids_array.as_mut_ptr();
        unsafe {
            *self.ids.array.add(self.ids.count as usize) = id;
        }
        self.ids.count = self.ids.count + 1;
        self
    }

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

    pub fn add_pair_ids_to_emit(&mut self, first: IdT, second: IdT) -> &mut Self {
        self.add_component_id_to_emit(ecs_pair(first, second))
    }

    pub fn add_pair_second_id_to_emit<First>(&mut self, second: IdT) -> &mut Self
    where
        First: CachedComponentData,
    {
        self.add_component_id_to_emit(ecs_pair(First::get_id(self.world.raw_world), second))
    }

    pub fn set_entity_to_emit(&mut self, entity: EntityT) -> &mut Self {
        let record: *mut ecs_record_t = unsafe { ecs_record_find(self.world.raw_world, entity) };

        // can't emit for empty entity
        ecs_assert!(
            record != std::ptr::null_mut(),
            ECS_INVALID_PARAMETER,
            "Can't emit for empty record"
        );
        ecs_assert!(
            unsafe { !(*record).table.is_null() },
            ECS_INVALID_PARAMETER,
            "Can't emit for empty record tble"
        );

        self.desc.table = unsafe { (*record).table };
        self.desc.offset = ecs_record_to_row(unsafe { (*record).row });
        self.desc.count = 1;
        self
    }

    pub fn set_table_to_emit(&mut self, table: *mut TableT, offset: i32, count: i32) -> &mut Self {
        self.desc.table = table;
        self.desc.offset = offset;
        self.desc.count = count;
        self
    }

    pub fn emit(&mut self) {
        ecs_assert!(self.ids.count > 0, ECS_INVALID_PARAMETER, "No ids to emit");
        ecs_assert!(
            !self.desc.table.is_null(),
            ECS_INVALID_PARAMETER,
            "No table to emit"
        );

        self.ids.array = self.ids_array.as_mut_ptr();
        self.desc.ids = &self.ids;
        self.desc.observable =
            unsafe { ecs_get_world(self.world.raw_world as *const c_void) } as *mut c_void;
        unsafe { ecs_emit(self.world.raw_world, &mut self.desc) };
    }
}

impl EventBuilderImpl for EventBuilder {
    type BuiltType = *const c_void;

    fn set_event_data(&mut self, data: Self::BuiltType) -> &mut Self {
        self.desc.param = data as *const c_void;
        self
    }
}

pub struct EventBuilderTyped<'a, T: EventData + CachedComponentData> {
    builder: EventBuilder,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T: EventData + CachedComponentData> EventBuilderTyped<'a, T> {
    pub fn new(world: *mut WorldT, event: EntityT) -> Self {
        Self {
            builder: EventBuilder::new(world, event),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T: EventData + CachedComponentData> Deref for EventBuilderTyped<'a, T> {
    type Target = EventBuilder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl<'a, T: EventData + CachedComponentData> DerefMut for EventBuilderTyped<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

impl<'a, T: EventData + CachedComponentData> EventBuilderImpl for EventBuilderTyped<'a, T>
where
    T: EventData + CachedComponentData,
{
    type BuiltType = &'a T;

    fn set_event_data(&mut self, data: Self::BuiltType) -> &mut Self {
        self.desc.param = data as *const T as *const c_void;
        self
    }
}
