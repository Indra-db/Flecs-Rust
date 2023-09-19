use std::{
    ffi::{c_void, CStr, CString},
    mem::MaybeUninit,
    sync::OnceLock,
};

use libc::strlen;

use crate::{
    core::{c_binding::bindings::ecs_get_world, utility::errors::FlecsErrorCode},
    ecs_assert,
};

use super::{
    archetype::Archetype,
    c_binding::bindings::{
        ecs_add_id, ecs_clear, ecs_delete, ecs_filter_desc_t, ecs_filter_init, ecs_filter_iter,
        ecs_filter_next, ecs_filter_t, ecs_get_id, ecs_get_name, ecs_get_path_w_sep,
        ecs_get_symbol, ecs_get_table, ecs_get_target, ecs_get_type, ecs_has_id, ecs_is_alive,
        ecs_is_valid, ecs_iter_t, ecs_oper_kind_t_EcsOptional, ecs_record_find, ecs_record_t,
        ecs_search_offset, ecs_table_get_type, ecs_table_t, ecs_term_t, EcsAny, EcsChildOf,
        EcsDisabled, EcsIsEntity, EcsPrefab, EcsUnion, EcsWildcard, ECS_FILTER_INIT,
    },
    c_types::*,
    component::{CachedComponentData, ComponentType, Enum, NotEmptyComponent, Struct},
    id::Id,
    table::{Table, TableRange},
    utility::functions::{ecs_pair, ecs_pair_first, ecs_pair_second, ecs_record_to_row},
    world::World,
};

static SEPARATOR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"::\0") };

#[derive(Default, Debug, Clone, Copy)]
pub struct Entity {
    pub id: Id,
}

impl Entity {
    /// Wrap an existing entity id.
    /// # Arguments
    /// * `world` - The world the entity belongs to.
    /// * `id` - The entity id.
    pub fn new(world: *mut WorldT, id: EntityT) -> Self {
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

    // Explicit conversion from flecs::entity_t to Entity
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

    /// checks if entity is valid
    pub fn get_is_valid(&self) -> bool {
        !self.id.world.is_null() && unsafe { ecs_is_valid(self.id.world, self.id.id) }
    }

    /// Checks if entity is alive.
    pub fn get_is_alive(&self) -> bool {
        !self.id.world.is_null() && unsafe { ecs_is_alive(self.id.world, self.id.id) }
    }

    /// Returns the entity name.
    pub fn get_name(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(ecs_get_name(self.id.world, self.id.id))
                .to_str()
                .unwrap_or("")
        }
    }

    //TODO check if we need this -> can we use get_symbol from CachedComponentData?
    /// Returns the entity symbol.
    pub fn get_symbol(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(ecs_get_symbol(self.id.world, self.id.id))
                .to_str()
                .unwrap_or("")
        }
    }

    /// Return the hierarchical entity path.
    /// # Note
    /// if you're using the default separator "::" you can use get_hierachy_path_default
    /// which does no extra heap allocations to communicate with C
    pub fn get_hierachy_path(&self, sep: &str, init_sep: &str) -> Option<String> {
        self.get_hierachy_path_from_parent_id(0, sep, init_sep)
    }

    /// Return the hierarchical entity path using the default separator "::".
    pub fn get_hierachy_path_default(&self) -> Option<String> {
        self.get_hierachy_path_from_parent_id_default(0)
    }

    /// Return the hierarchical entity path relative to a parent.
    /// # Note
    /// if you're using the default separator "::" you can use get_hierachy_path_default
    /// which does no extra heap allocations to communicate with C
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
                    self.id.world,
                    parent,
                    self.id.id,
                    c_sep.as_ptr(),
                    c_sep.as_ptr(),
                )
            }
        } else {
            let c_init_sep = CString::new(init_sep).unwrap();
            unsafe {
                ecs_get_path_w_sep(
                    self.id.world,
                    parent,
                    self.id.id,
                    c_sep.as_ptr(),
                    c_init_sep.as_ptr(),
                )
            }
        };

        if raw_ptr.is_null() {
            return None;
        }

        let len = unsafe { strlen(raw_ptr) } as usize;

        // Convert the C string to a Rust String without any new heap allocation.
        // The String will de-allocate the C string when it goes out of scope.
        Some(unsafe {
            String::from_utf8_unchecked(Vec::from_raw_parts(raw_ptr as *mut u8, len, len))
        })
    }

    /// Return the hierarchical entity path relative to a parent id using the default separator "::".
    pub fn get_hierachy_path_from_parent_id_default(&self, parent: EntityT) -> Option<String> {
        unsafe {
            let raw_ptr = ecs_get_path_w_sep(
                self.id.world,
                parent,
                self.id.id,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
            );

            if raw_ptr.is_null() {
                return None;
            }

            let len = strlen(raw_ptr) as usize;

            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            Some(String::from_utf8_unchecked(Vec::from_raw_parts(
                raw_ptr as *mut u8,
                len,
                len,
            )))
        }
    }

    /// Return the hierarchical entity path relative to a parent type.
    /// # Note
    /// if you're using the default separator "::" you can use get_hierachy_path_default
    /// which does no extra heap allocations to communicate with C
    pub fn get_hierachy_path_from_parent_type<T: CachedComponentData>(
        &self,
        sep: &str,
        init_sep: &str,
    ) -> Option<String> {
        self.get_hierachy_path_from_parent_id(T::get_id(self.id.world), sep, init_sep)
    }

    /// Return the hierarchical entity path relative to a parent type using the default separator "::".
    pub fn get_hierachy_path_from_parent_type_default<T: CachedComponentData>(
        &self,
    ) -> Option<String> {
        self.get_hierachy_path_from_parent_id_default(T::get_id(self.id.world))
    }

    pub fn get_is_enabled(&self) -> bool {
        unsafe { !ecs_has_id(self.id.world, self.id.id, EcsDisabled) }
    }

    /// get the entity's archetype
    #[inline(always)]
    pub fn get_archetype(&self) -> Archetype {
        Archetype::new(self.id.world, unsafe {
            ecs_get_type(self.id.world, self.id.id)
        })
    }

    /// get the entity's table
    #[inline(always)]
    pub fn get_table(&self) -> Table {
        Table::new(self.id.world, unsafe {
            ecs_get_table(self.id.world, self.id.id)
        })
    }

    /// Get table range for the entity.
    /// ### Returns
    /// Returns a range with the entity's row as offset and count set to 1. If
    /// the entity is not stored in a table, the function returns a range with
    /// count 0.
    #[inline]
    pub fn get_table_range(&self) -> TableRange {
        let ecs_record: *mut ecs_record_t = unsafe { ecs_record_find(self.id.world, self.id.id) };
        if !ecs_record.is_null() {
            unsafe {
                TableRange::new_raw(
                    self.id.world,
                    (*ecs_record).table,
                    ecs_record_to_row((*ecs_record).row),
                    1,
                )
            }
        } else {
            TableRange::default()
        }
    }

    /// Iterate over component ids of an entity.
    ///
    /// ### Arguments
    /// * `func` - The closure invoked for each matching ID. Must match the signature `FnMut(Id)`.
    fn for_each_component<F>(&self, mut func: F)
    where
        F: FnMut(Id),
    {
        let type_ptr = unsafe { ecs_get_type(self.id.world, self.id.id) };

        if type_ptr.is_null() {
            return;
        }

        let type_ref: &TypeT = unsafe { &*type_ptr };
        let ids = type_ref.array;
        let count = type_ref.count;

        for i in 0..count as usize {
            let id: IdT = unsafe { *ids.add(i) };
            let ent = Id {
                world: self.id.world,
                id,
            };
            func(ent);

            // Union object is not stored in type, so handle separately
            if unsafe { ecs_pair_first(id) == EcsUnion } {
                let ent = Id::new_world_pair(self.id.world, ecs_pair_second(id), unsafe {
                    ecs_get_target(self.id.world, self.id.id, ecs_pair_second(self.id.id), 0)
                });

                func(ent);
            }
        }
    }

    /// Iterates over matching pair IDs of an entity.
    ///
    /// ### Arguments
    ///
    /// * `first` - The first ID to match against.
    /// * `second` - The second ID to match against.
    /// * `func` - The closure invoked for each matching ID. Must match the signature `FnMut(Id)`.
    fn for_each_matching_pair<F>(&self, pred: IdT, obj: IdT, mut func: F)
    where
        F: FnMut(Id),
    {
        // this is safe because we are only reading the world
        let real_world = unsafe { ecs_get_world(self.id.world as *const c_void) as *mut WorldT };

        let table: *mut ecs_table_t = unsafe { ecs_get_table(self.id.world, self.id.id) };

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
            let ent = Id::new(self.id.world, unsafe { *(ids.add(cur as usize)) });
            func(ent);
            cur += 1;
        }
    }

    /// Iterate over targets for a given relationship.
    ///
    /// ### Arguments
    ///
    /// * `relationship` - The relationship for which to iterate the targets.
    /// * `func` - The closure invoked for each target. Must match the signature `FnMut(Entity)`.
    pub fn for_each_target_in_relationship_by_entity<F>(&self, relationship: Entity, mut func: F)
    where
        F: FnMut(Entity),
    {
        self.for_each_matching_pair(relationship.id.id, unsafe { EcsWildcard }, |id| {
            let obj = id.second();
            func(obj);
        });
    }

    /// Iterate over targets for a given relationship.
    ///
    /// ### Type Parameters
    ///
    /// * `Relationship` - The relationship for which to iterate the targets.
    ///
    /// ### Arguments
    ///
    /// * `func` - The function invoked for each target.
    pub fn for_each_target_in_relationship<T, F>(&self, func: F)
    where
        T: CachedComponentData,
        F: FnMut(Entity),
    {
        self.for_each_target_in_relationship_by_entity(
            Entity::new_only_id(T::get_id(self.id.world)),
            func,
        );
    }

    /// Iterate children for entity
    ///
    /// ### Arguments
    ///
    /// * `relationship` - The relationship to follow
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    pub fn for_each_children<F>(&self, relationship: EntityT, mut func: F)
    where
        F: FnMut(Entity),
    {
        // When the entity is a wildcard, this would attempt to query for all
        //entities with (ChildOf, *) or (ChildOf, _) instead of querying for
        //the children of the wildcard entity.
        if unsafe { self.id.id == EcsWildcard || self.id.id == EcsAny } {
            // this is correct, wildcard entities don't have children
            return;
        }

        let world: World = World::new_from_world(self.id.world);

        let mut terms: [ecs_term_t; 2] = unsafe { MaybeUninit::zeroed().assume_init() };

        let mut filter: ecs_filter_t = unsafe { ECS_FILTER_INIT };
        filter.terms = terms.as_mut_ptr();
        filter.term_count = 2;

        let mut desc: ecs_filter_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
        desc.terms[0].first.id = relationship;
        desc.terms[0].second.id = self.id.id;
        unsafe {
            desc.terms[0].second.flags = EcsIsEntity;
            desc.terms[1].id = EcsPrefab;
            desc.terms[1].oper = ecs_oper_kind_t_EcsOptional;
        }
        desc.storage = &mut filter;

        if !unsafe { ecs_filter_init(self.id.world, &desc) }.is_null() {
            let mut it: ecs_iter_t = unsafe { ecs_filter_iter(self.id.world, &filter) };
            while unsafe { ecs_filter_next(&mut it) } {
                todo!("yet to implement");
            }
        }
    }

    /// Iterate children for entity
    ///
    /// ### Arguments
    ///
    /// * T - The relationship to follow
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    pub fn for_each_children1<T, F>(&self, mut func: F)
    where
        T: CachedComponentData,
        F: FnMut(Entity),
    {
        self.for_each_children(T::get_id(self.id.world), func);
    }

    /// Iterate children for entity
    /// This operation follows the ChildOf relationship.
    /// ### Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    pub fn for_each_children2<F>(&self, mut func: F)
    where
        F: FnMut(Entity),
    {
        self.for_each_children(unsafe { EcsChildOf }, func);
    }

    /// Get (struct) Component from entity
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The component type to get
    ///
    /// ### Returns
    ///
    /// * `*const T` - The enum component, nullptr if the entity does not have the component
    pub fn get<T: CachedComponentData + ComponentType<Struct>>(&self) -> *const T {
        let component_id = T::get_id(self.id.world);
        unsafe { (ecs_get_id(self.id.world, self.id.id, component_id) as *const T) }
    }

    /// Get (enum) Component from entity
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The enum component type to get
    ///
    /// ### Returns
    ///
    /// * `*const T` - The enum component, nullptr if the entity does not have the component
    pub fn get1<T: CachedComponentData + ComponentType<Enum>>(&self) -> *const T {
        let component_id: IdT = T::get_id(self.id.world);
        let target: IdT = unsafe { ecs_get_target(self.id.world, self.id.id, component_id, 0) };

        if target == 0 {
            unsafe { ecs_get_id(self.id.world, self.id.id, component_id) as *const T }
        } else {
            // get constant value from constant entity
            let constant_value =
                unsafe { ecs_get_id(self.id.world, target, component_id) as *const T };

            ecs_assert!(
                !constant_value.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );

            constant_value
        }
    }
    //
    //
    //
    /*
    temp placed seperately
    */

    pub fn add_component<T: CachedComponentData>(self) -> Self {
        let component_id = T::get_id(self.id.world);
        unsafe { ecs_add_id(self.id.world, self.id.id, component_id) }
        self
    }

    pub fn destruct(self) {
        unsafe { ecs_delete(self.id.world, self.id.id) }
    }

    pub fn clear(&self) {
        unsafe { ecs_clear(self.id.world, self.id.id) }
    }
}
