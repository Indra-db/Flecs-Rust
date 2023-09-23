use std::{
    ffi::{c_void, CStr, CString},
    mem::MaybeUninit,
    ops::Deref,
    sync::OnceLock,
};

use flecs_ecs_bridge_derive::Component;
use libc::strlen;

use crate::{
    core::{
        c_binding::bindings::ecs_get_world,
        data_structures::pair::{PairT, PairTT},
        utility::errors::FlecsErrorCode,
    },
    ecs_assert,
};

use super::{
    archetype::Archetype,
    c_binding::bindings::{
        ecs_add_id, ecs_clear, ecs_clone, ecs_delete, ecs_filter_desc_t, ecs_filter_init,
        ecs_filter_iter, ecs_filter_next, ecs_filter_t, ecs_get_depth, ecs_get_id, ecs_get_name,
        ecs_get_path_w_sep, ecs_get_symbol, ecs_get_table, ecs_get_target, ecs_get_type,
        ecs_has_id, ecs_is_alive, ecs_is_enabled_id, ecs_is_valid, ecs_iter_t,
        ecs_lookup_path_w_sep, ecs_new_id, ecs_oper_kind_t_EcsOptional, ecs_owns_id,
        ecs_record_find, ecs_record_t, ecs_search_offset, ecs_table_get_type, ecs_table_t,
        ecs_term_t, EcsAny, EcsChildOf, EcsDisabled, EcsIsEntity, EcsPrefab, EcsUnion, EcsWildcard,
        ECS_FILTER_INIT,
    },
    c_types::*,
    component::{CachedComponentData, ComponentType, Enum, NotEmptyComponent, Struct},
    enum_type::CachedEnumData,
    id::Id,
    table::{Table, TableRange},
    utility::functions::{
        ecs_has_pair, ecs_pair, ecs_pair_first, ecs_pair_second, ecs_record_to_row,
    },
    world::World,
};

static SEPARATOR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"::\0") };

#[derive(Default, Debug, Clone, Copy)]
pub struct Entity {
    pub id: Id,
}

impl Deref for Entity {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl Entity {
    pub fn new(world: *mut WorldT, id: IdT) -> Self {
        unsafe {
            Self {
                id: Id::new(
                    if world.is_null() {
                        std::ptr::null_mut()
                    } else {
                        ecs_get_world(world as *mut c_void) as *mut WorldT
                    },
                    id,
                ),
            }
        }
    }

    pub const fn new_only_id(id: EntityT) -> Self {
        Self {
            id: Id::new_only_id(id),
        }
    }

    pub fn new_only_world(world: *mut WorldT) -> Self {
        Self {
            id: Id::new_only_world(world),
        }
    }

    pub fn get_is_valid(&self) -> bool {
        !self.world.is_null() && unsafe { ecs_is_valid(self.world, self.raw_id) }
    }

    pub fn get_is_alive(&self) -> bool {
        !self.world.is_null() && unsafe { ecs_is_alive(self.world, self.raw_id) }
    }

    pub fn get_name(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(ecs_get_name(self.world, self.raw_id))
                .to_str()
                .unwrap_or("")
        }
    }

    pub fn get_symbol(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(ecs_get_symbol(self.world, self.raw_id))
                .to_str()
                .unwrap_or("")
        }
    }

    pub fn get_hierachy_path(&self, sep: &str, init_sep: &str) -> Option<String> {
        self.get_hierachy_path_from_parent_id(0, sep, init_sep)
    }

    pub fn get_hierachy_path_default(&self) -> Option<String> {
        self.get_hierachy_path_from_parent_id_default(0)
    }

    pub fn get_hierachy_path_from_parent_id(
        &self,
        parent: EntityT,
        sep: &str,
        init_sep: &str,
    ) -> Option<String> {
        let c_sep = CString::new(sep).unwrap();
        let raw_ptr = if sep == init_sep {
            unsafe {
                ecs_get_path_w_sep(
                    self.world,
                    parent,
                    self.raw_id,
                    c_sep.as_ptr(),
                    c_sep.as_ptr(),
                )
            }
        } else {
            let c_init_sep = CString::new(init_sep).unwrap();
            unsafe {
                ecs_get_path_w_sep(
                    self.world,
                    parent,
                    self.raw_id,
                    c_sep.as_ptr(),
                    c_init_sep.as_ptr(),
                )
            }
        };

        if raw_ptr.is_null() {
            return None;
        }

        let len = unsafe { strlen(raw_ptr) } as usize;

        Some(unsafe {
            String::from_utf8_unchecked(Vec::from_raw_parts(raw_ptr as *mut u8, len, len))
        })
    }

    pub fn get_hierachy_path_from_parent_id_default(&self, parent: EntityT) -> Option<String> {
        unsafe {
            let raw_ptr = ecs_get_path_w_sep(
                self.world,
                parent,
                self.raw_id,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
            );

            if raw_ptr.is_null() {
                return None;
            }

            let len = strlen(raw_ptr) as usize;

            Some(String::from_utf8_unchecked(Vec::from_raw_parts(
                raw_ptr as *mut u8,
                len,
                len,
            )))
        }
    }

    pub fn get_hierachy_path_from_parent_type<T: CachedComponentData>(
        &self,
        sep: &str,
        init_sep: &str,
    ) -> Option<String> {
        self.get_hierachy_path_from_parent_id(T::get_id(self.world), sep, init_sep)
    }

    pub fn get_hierachy_path_from_parent_type_default<T: CachedComponentData>(
        &self,
    ) -> Option<String> {
        self.get_hierachy_path_from_parent_id_default(T::get_id(self.world))
    }

    pub fn get_is_enabled(&self) -> bool {
        unsafe { !ecs_has_id(self.world, self.raw_id, EcsDisabled) }
    }

    #[inline(always)]
    pub fn get_archetype(&self) -> Archetype {
        Archetype::new(self.world, unsafe { ecs_get_type(self.world, self.raw_id) })
    }

    #[inline(always)]
    pub fn get_table(&self) -> Table {
        Table::new(self.world, unsafe {
            ecs_get_table(self.world, self.raw_id)
        })
    }

    #[inline]
    pub fn get_table_range(&self) -> TableRange {
        let ecs_record: *mut ecs_record_t = unsafe { ecs_record_find(self.world, self.raw_id) };
        if !ecs_record.is_null() {
            unsafe {
                TableRange::new_raw(
                    self.world,
                    (*ecs_record).table,
                    ecs_record_to_row((*ecs_record).row),
                    1,
                )
            }
        } else {
            TableRange::default()
        }
    }

    fn for_each_component<F>(&self, mut func: F)
    where
        F: FnMut(Id),
    {
        let type_ptr = unsafe { ecs_get_type(self.world, self.raw_id) };

        if type_ptr.is_null() {
            return;
        }

        let type_ref: &TypeT = unsafe { &*type_ptr };
        let ids = type_ref.array;
        let count = type_ref.count;

        for i in 0..count as usize {
            let id: IdT = unsafe { *ids.add(i) };
            let ent = Id {
                world: self.world,
                raw_id: id,
            };
            func(ent);

            if unsafe { ecs_pair_first(id) == EcsUnion } {
                let ent = Id::new_world_pair(self.world, ecs_pair_second(id), unsafe {
                    ecs_get_target(self.world, self.raw_id, ecs_pair_second(self.raw_id), 0)
                });

                func(ent);
            }
        }
    }

    fn for_each_matching_pair<F>(&self, pred: IdT, obj: IdT, mut func: F)
    where
        F: FnMut(Id),
    {
        let real_world = unsafe { ecs_get_world(self.world as *const c_void) as *mut WorldT };

        let table: *mut ecs_table_t = unsafe { ecs_get_table(self.world, self.raw_id) };

        if table.is_null() {
            return;
        }

        let table_type = unsafe { ecs_table_get_type(table) };
        if table_type.is_null() {
            return;
        }

        let mut pattern: IdT = pred;
        if obj != 0 {
            pattern = ecs_pair(pred, obj);
        }

        let mut cur: i32 = 0;
        let ids: *mut IdT = unsafe { (*table_type).array };
        let id_out: *mut IdT = &mut 0;

        while -1 != unsafe { ecs_search_offset(real_world, table, cur, pattern, id_out) } {
            let ent = Id::new(self.world, unsafe { *(ids.add(cur as usize)) });
            func(ent);
            cur += 1;
        }
    }

    pub fn for_each_target_in_relationship_by_entity<F>(&self, relationship: Entity, mut func: F)
    where
        F: FnMut(Entity),
    {
        self.for_each_matching_pair(relationship.id.raw_id, unsafe { EcsWildcard }, |id| {
            let obj = id.second();
            func(obj);
        });
    }

    pub fn for_each_target_in_relationship<T, F>(&self, func: F)
    where
        T: CachedComponentData,
        F: FnMut(Entity),
    {
        self.for_each_target_in_relationship_by_entity(
            Entity::new_only_id(T::get_id(self.world)),
            func,
        );
    }

    pub fn for_each_children<F>(&self, relationship: EntityT, mut func: F)
    where
        F: FnMut(Entity),
    {
        if unsafe { self.raw_id == EcsWildcard || self.raw_id == EcsAny } {
            return;
        }

        let world: World = World::new_from_world(self.world);

        let mut terms: [ecs_term_t; 2] = unsafe { MaybeUninit::zeroed().assume_init() };

        let mut filter: ecs_filter_t = unsafe { ECS_FILTER_INIT };
        filter.terms = terms.as_mut_ptr();
        filter.term_count = 2;

        let mut desc: ecs_filter_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
        desc.terms[0].first.id = relationship;
        desc.terms[0].second.id = self.raw_id;
        unsafe {
            desc.terms[0].second.flags = EcsIsEntity;
            desc.terms[1].id = EcsPrefab;
            desc.terms[1].oper = ecs_oper_kind_t_EcsOptional;
        }
        desc.storage = &mut filter;

        if !unsafe { ecs_filter_init(self.world, &desc) }.is_null() {
            let mut it: ecs_iter_t = unsafe { ecs_filter_iter(self.world, &filter) };
            while unsafe { ecs_filter_next(&mut it) } {
                for i in 0..it.count as usize {
                    unsafe {
                        let id = it.entities.add(i);
                        let ent = Entity::new(self.world, *id);
                        func(ent);
                    }
                }
            }
        }
    }

    pub fn for_each_children1<T, F>(&self, mut func: F)
    where
        T: CachedComponentData,
        F: FnMut(Entity),
    {
        self.for_each_children(T::get_id(self.world), func);
    }

    pub fn for_each_children2<F>(&self, mut func: F)
    where
        F: FnMut(Entity),
    {
        self.for_each_children(unsafe { EcsChildOf }, func);
    }

    pub fn get_component<T: CachedComponentData + ComponentType<Struct>>(&self) -> *const T {
        let component_id = T::get_id(self.world);
        unsafe { (ecs_get_id(self.world, self.raw_id, component_id) as *const T) }
    }

    pub fn get_enum_component<T: CachedComponentData + ComponentType<Enum>>(&self) -> *const T {
        let component_id: IdT = T::get_id(self.world);
        let target: IdT = unsafe { ecs_get_target(self.world, self.raw_id, component_id, 0) };

        if target == 0 {
            unsafe { ecs_get_id(self.world, self.raw_id, component_id) as *const T }
        } else {
            let constant_value =
                unsafe { ecs_get_id(self.world, target, component_id) as *const T };

            ecs_assert!(
                !constant_value.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );

            constant_value
        }
    }

    pub fn get_component_by_id(&self, component_id: IdT) -> *const c_void {
        unsafe { ecs_get_id(self.world, self.raw_id, component_id) as *const c_void }
    }

    pub fn get_pair_untyped(&self, first: EntityT, second: EntityT) -> *const c_void {
        unsafe { ecs_get_id(self.world, self.raw_id, ecs_pair(first, second)) as *const c_void }
    }

    pub fn get_target_from_component<First: CachedComponentData>(&self, index: i32) -> Entity {
        Entity::new(self.world, unsafe {
            ecs_get_target(self.world, self.raw_id, First::get_id(self.world), index)
        })
    }

    pub fn get_target_from_entity(&self, first: EntityT, index: i32) -> Entity {
        Entity::new(self.world, unsafe {
            ecs_get_target(self.world, self.raw_id, first, index)
        })
    }

    pub fn get_target_by_component_id(&self, relationship: EntityT, component_id: IdT) -> Entity {
        Entity::new(self.world, unsafe {
            ecs_get_target(self.world, self.raw_id, relationship, component_id as i32)
        })
    }

    #[inline(always)]
    pub fn get_target_for_component<T: CachedComponentData>(
        &self,
        relationship: EntityT,
    ) -> Entity {
        self.get_target_by_component_id(relationship, T::get_id(self.world))
    }

    #[inline(always)]
    pub fn get_target_for_pair<First: CachedComponentData, Second: CachedComponentData>(
        &self,
        relationship: EntityT,
    ) -> Entity {
        self.get_target_by_component_id(
            relationship,
            ecs_pair(First::get_id(self.world), Second::get_id(self.world)),
        )
    }

    #[inline(always)]
    pub fn get_depth_by_id(&self, relationship: EntityT) -> i32 {
        unsafe { ecs_get_depth(self.world, self.raw_id, relationship) }
    }

    #[inline(always)]
    pub fn get_depth<T: CachedComponentData>(&self) -> i32 {
        self.get_depth_by_id(T::get_id(self.world))
    }

    #[inline(always)]
    pub fn parent(&self) -> Entity {
        self.get_target_from_entity(unsafe { EcsChildOf }, 0)
    }

    #[inline(always)]
    pub fn lookup_entity_by_name(&self, path: &str) -> Entity {
        ecs_assert!(
            self.raw_id != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid lookup from null handle"
        );
        let c_path = CString::new(path).unwrap();
        Entity::new(self.world, unsafe {
            ecs_lookup_path_w_sep(
                self.world,
                self.raw_id,
                c_path.as_ptr(),
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                false,
            )
        })
    }

    #[inline(always)]
    pub fn has_entity(&self, entity: IdT) -> bool {
        unsafe { ecs_has_id(self.world, self.raw_id, entity) }
    }

    pub fn has_struct_component<T: CachedComponentData + ComponentType<Struct>>(&self) -> bool {
        unsafe { ecs_has_id(self.world, self.raw_id, T::get_id(self.world)) }
    }

    pub fn has_enum_component<T: CachedComponentData + ComponentType<Enum>>(&self) -> bool {
        let component_id: IdT = T::get_id(self.world);
        ecs_has_pair(self.world, self.raw_id, component_id, unsafe {
            EcsWildcard
        })
    }

    pub fn has_enum_constant<T>(&self, constant: T) -> bool
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        let component_id: IdT = T::get_id(self.world);
        let enum_constant_entity_id: IdT = constant.get_entity_id_from_enum_field(self.world);
        ecs_has_pair(
            self.world,
            self.raw_id,
            component_id,
            enum_constant_entity_id,
        )
    }

    pub fn has_pair<T: CachedComponentData, U: CachedComponentData>(&self) -> bool {
        ecs_has_pair(
            self.world,
            self.raw_id,
            T::get_id(self.world),
            U::get_id(self.world),
        )
    }

    pub fn has_pair_by_id(&self, first: IdT, second: IdT) -> bool {
        ecs_has_pair(self.world, self.raw_id, first, second)
    }

    pub fn has_pair_with_enum_constant<
        T: CachedComponentData,
        U: CachedComponentData + CachedEnumData,
    >(
        &self,
        constant: U,
    ) -> bool {
        let component_id: IdT = T::get_id(self.world);
        let enum_constant_entity_id: IdT = constant.get_entity_id_from_enum_field(self.world);

        ecs_has_pair(
            self.world,
            self.raw_id,
            component_id,
            enum_constant_entity_id,
        )
    }

    pub fn get_is_entity_owner_of_id(&self, entity_id: IdT) -> bool {
        unsafe { ecs_owns_id(self.world, self.raw_id, entity_id) }
    }

    pub fn get_is_entity_owner_of_entity(&self, entity: Entity) -> bool {
        unsafe { ecs_owns_id(self.world, self.raw_id, entity.id.raw_id) }
    }

    pub fn get_is_entity_owner_of<T: CachedComponentData>(&self) -> bool {
        unsafe { ecs_owns_id(self.world, self.raw_id, T::get_id(self.world)) }
    }

    pub fn get_is_entity_owner_of_pair_ids(&self, first: IdT, second: IdT) -> bool {
        unsafe { ecs_owns_id(self.world, self.raw_id, ecs_pair(first, second)) }
    }

    pub fn get_is_entity_owner_of_pair<T: CachedComponentData, U: CachedComponentData>(
        &self,
    ) -> bool {
        unsafe {
            ecs_owns_id(
                self.world,
                self.raw_id,
                ecs_pair(T::get_id(self.world), U::get_id(self.world)),
            )
        }
    }

    pub fn get_is_id_enabled(&self, id: IdT) -> bool {
        unsafe { ecs_is_enabled_id(self.world, self.raw_id, id) }
    }

    pub fn get_is_component_enabled<T: CachedComponentData>(&self) -> bool {
        unsafe { ecs_is_enabled_id(self.world, self.raw_id, T::get_id(self.world)) }
    }

    pub fn get_is_pair_ids_enabled(&self, first: IdT, second: IdT) -> bool {
        unsafe { ecs_is_enabled_id(self.world, self.raw_id, ecs_pair(first, second)) }
    }

    pub fn get_is_pair_enabled<T: CachedComponentData, U: CachedComponentData>(&self) -> bool {
        unsafe {
            ecs_is_enabled_id(
                self.world,
                self.raw_id,
                ecs_pair(T::get_id(self.world), U::get_id(self.world)),
            )
        }
    }

    #[inline(always)]
    pub fn clone(&self, copy_value: bool, mut dest_id: EntityT) -> Entity {
        if dest_id == 0 {
            dest_id = unsafe { ecs_new_id(self.world) };
        }

        let dest_entity = Entity::new(self.world, dest_id);
        unsafe { ecs_clone(self.world, dest_id, self.raw_id, copy_value) };
        dest_entity
    }

    pub fn get_mutable_handle_for_stage(&self, stage: &World) -> Entity {
        ecs_assert!(
            !stage.is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use readonly world/stage to create mutable handle"
        );

        Entity::new(stage.world, self.raw_id)
    }

    pub fn get_mutable_handle_from_entity(&self, entity: &Entity) -> Entity {
        ecs_assert!(
            !entity.id.get_as_world().is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use entity created for readonly world/stage to create mutable handle"
        );

        Entity::new(entity.id.world, self.raw_id)
    }

    fn set_stage(&self, stage: *mut WorldT) -> Entity {
        Entity::new(stage, self.raw_id)
    }

    pub fn add_component<T: CachedComponentData>(self) -> Self {
        let component_id = T::get_id(self.world);
        unsafe { ecs_add_id(self.world, self.raw_id, component_id) }
        self
    }

    pub fn destruct(self) {
        unsafe { ecs_delete(self.world, self.raw_id) }
    }

    pub fn clear(&self) {
        unsafe { ecs_clear(self.world, self.raw_id) }
    }
}
