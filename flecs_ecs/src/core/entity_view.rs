// Standard Library imports
use std::{
    ffi::{c_void, CStr},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr,
};

use flecs_ecs_sys::{
    ecs_field_src, ecs_get_target_for_id, ecs_iter_action_t, ecs_observer_desc_t,
    ecs_observer_init, ecs_table_lock, ecs_table_unlock,
};

// Module imports from within the current crate
use crate::{
    core::FlecsErrorCode,
    ecs_assert,
    sys::{
        ecs_clone, ecs_filter_desc_t, ecs_filter_init, ecs_filter_iter, ecs_filter_next,
        ecs_filter_t, ecs_get_depth, ecs_get_id, ecs_get_name, ecs_get_path_w_sep, ecs_get_symbol,
        ecs_get_table, ecs_get_target, ecs_get_type, ecs_get_world, ecs_has_id, ecs_is_alive,
        ecs_is_enabled_id, ecs_is_valid, ecs_iter_t, ecs_lookup_path_w_sep, ecs_new_id,
        ecs_oper_kind_t_EcsOptional, ecs_owns_id, ecs_record_find, ecs_record_t, ecs_search_offset,
        ecs_table_get_type, ecs_table_t, ecs_term_t, EcsAny, EcsChildOf, EcsDisabled, EcsIsEntity,
        EcsPrefab, EcsUnion, EcsWildcard, ECS_FILTER_INIT,
    },
};

// Super module imports
use super::{
    archetype::Archetype,
    c_types::{EntityT, IdT, TypeT, WorldT, SEPARATOR},
    component_registration::{CachedComponentData, ComponentType, Enum, Struct},
    ecs_add_pair, ecs_has_pair, ecs_pair, ecs_pair_first, ecs_pair_second, ecs_record_to_row,
    entity::Entity,
    enum_type::CachedEnumData,
    id::Id,
    table::{Table, TableRange},
    world::World,
    EmptyComponent, EventBuilderImpl, EventData, IterT, NotEmptyComponent,
    ObserverEntityBindingCtx, ECS_ANY, ECS_CHILD_OF,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityView {
    pub id: Id,
}

impl Deref for EntityView {
    type Target = Id;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl DerefMut for EntityView {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}

impl EntityView {
    /// Wrap an existing entity id.
    /// # Arguments
    /// * `world` - The world the entity belongs to.
    /// * `id` - The entity id.
    ///
    /// # Safety
    ///
    /// if the world is passed as None, it's not safe to use this entity for operations on it
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::entity_view`
    #[doc(alias = "entity_view::entity_view")]
    pub fn new(world: Option<&World>, id: IdT) -> Self {
        if let Some(world) = world {
            Self {
                id: Id::new_from_existing(world.raw_world, id),
            }
        } else {
            Self {
                id: Id::new_id_only(id),
            }
        }
    }

    /// Wrap an existing entity id.
    /// # Arguments
    /// * `world` - The world the entity belongs to as void*.
    /// * `id` - The entity id.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::entity_view`
    #[doc(alias = "entity_view::entity_view")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_from_existing_with_poly_world(world: *mut c_void, id: IdT) -> Self {
        unsafe {
            Self {
                id: Id::new_from_existing(
                    if world.is_null() {
                        std::ptr::null_mut()
                    } else {
                        ecs_get_world(world) as *mut WorldT
                    },
                    id,
                ),
            }
        }
    }

    /// Wrap an existing entity id.
    /// # Arguments
    /// * `world` - The world the entity belongs to.
    /// * `id` - The entity id.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::entity_view`
    #[doc(alias = "entity_view::entity_view")]
    pub(crate) fn new_from_existing(world: *mut WorldT, id: IdT) -> Self {
        Self {
            id: Id::new_from_existing(world, id),
        }
    }

    /// Explicit conversion from `EntityT` to `EntityView`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::entity_view`
    #[doc(alias = "entity_view::entity_view")]
    pub const fn new_id_only(id: EntityT) -> Self {
        Self {
            id: Id::new_id_only(id),
        }
    }

    /// checks if entity is valid
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::is_valid`
    #[doc(alias = "entity_view::is_valid")]
    pub fn is_valid(&self) -> bool {
        !self.world.is_null() && unsafe { ecs_is_valid(self.world, self.raw_id) }
    }

    /// Checks if entity is alive.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::is_alive`
    #[doc(alias = "entity_view::is_alive")]
    pub fn is_alive(&self) -> bool {
        !self.world.is_null() && unsafe { ecs_is_alive(self.world, self.raw_id) }
    }

    /// Returns the entity name.
    ///
    /// if the entity has no name, this will return an empty string
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    #[doc(alias = "entity_view::name")]
    pub fn get_name(&self) -> &'static str {
        let name_ptr = unsafe { ecs_get_name(self.world, self.raw_id) };

        if name_ptr.is_null() {
            return "";
        }

        unsafe { CStr::from_ptr(name_ptr).to_str().unwrap_or("") }
    }

    /// Returns the entity name.
    ///
    /// if the entity has no name, this will return none
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    #[doc(alias = "entity_view::name")]
    pub fn get_name_optional(&self) -> Option<&'static str> {
        let name_ptr = unsafe { ecs_get_name(self.world, self.raw_id) };

        if name_ptr.is_null() {
            return None;
        }

        Some(unsafe { CStr::from_ptr(name_ptr).to_str().unwrap_or("") })
    }

    //TODO check if we need this -> can we use get_symbol from CachedComponentData?
    /// Returns the entity symbol.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::symbol`
    #[doc(alias = "entity_view::symbol")]
    pub fn get_symbol(&self) -> &'static CStr {
        unsafe { CStr::from_ptr(ecs_get_symbol(self.world, self.raw_id)) }
    }

    /// Return the hierarchical entity path.
    /// # Note
    /// if you're using the default separator "::" you can use `get_hierarchy_path_default`
    /// which does no extra heap allocations to communicate with C
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path`
    #[doc(alias = "entity_view::path")]
    pub fn get_hierarchy_path_w_sep(&self, sep: &CStr, init_sep: &CStr) -> Option<String> {
        self.get_hierarchy_path_from_parent_id_w_sep(0, sep, init_sep)
    }

    /// Return the hierarchical entity path using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path`
    #[doc(alias = "entity_view::path")]
    pub fn get_hierarchy_path(&self) -> Option<String> {
        self.get_hierarchy_path_from_parent_id(0)
    }

    /// Return the hierarchical entity path relative to a parent.
    ///
    /// if you're using the default separator "::" you can use `get_hierarchy_path_default`
    /// which does no extra heap allocations to communicate with C
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path_from`
    #[doc(alias = "entity_view::path_from")]
    pub fn get_hierarchy_path_from_parent_id_w_sep(
        &self,
        parent: EntityT,
        sep: &CStr,
        init_sep: &CStr,
    ) -> Option<String> {
        let raw_ptr = if sep == init_sep {
            unsafe {
                ecs_get_path_w_sep(self.world, parent, self.raw_id, sep.as_ptr(), sep.as_ptr())
            }
        } else {
            unsafe {
                ecs_get_path_w_sep(
                    self.world,
                    parent,
                    self.raw_id,
                    sep.as_ptr(),
                    init_sep.as_ptr(),
                )
            }
        };

        if raw_ptr.is_null() {
            return None;
        }

        let len = unsafe { CStr::from_ptr(raw_ptr).to_bytes().len() };

        // Convert the C string to a Rust String without any new heap allocation.
        // The String will de-allocate the C string when it goes out of scope.
        Some(unsafe {
            String::from_utf8_unchecked(Vec::from_raw_parts(raw_ptr as *mut u8, len, len))
        })
    }

    /// Return the hierarchical entity path relative to a parent id using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path_from`
    #[doc(alias = "entity_view::path_from")]
    pub fn get_hierarchy_path_from_parent_id(&self, parent: EntityT) -> Option<String> {
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

            let len = CStr::from_ptr(raw_ptr).to_bytes().len();

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
    /// if you're using the default separator "::" you can use `get_hierarchy_path_default`
    /// which does no extra heap allocations to communicate with C
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path_from`
    #[doc(alias = "entity_view::path_from")]
    pub fn get_hierarchy_path_from_parent_type_w_sep<T: CachedComponentData>(
        &self,
        sep: &CStr,
        init_sep: &CStr,
    ) -> Option<String> {
        self.get_hierarchy_path_from_parent_id_w_sep(T::get_id(self.world), sep, init_sep)
    }

    /// Return the hierarchical entity path relative to a parent type using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path_from`
    #[doc(alias = "entity_view::path_from")]
    pub fn get_hierarchy_path_from_parent_type<T: CachedComponentData>(&self) -> Option<String> {
        self.get_hierarchy_path_from_parent_id(T::get_id(self.world))
    }

    /// Checks if the entity is enabled.
    ///
    /// # Returns
    ///
    /// True if the entity is enabled, false if disabled.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled(&self) -> bool {
        unsafe { !ecs_has_id(self.world, self.raw_id, EcsDisabled) }
    }

    /// get the entity's archetype
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::type`
    #[doc(alias = "entity_view::type")]
    #[inline(always)]
    pub fn get_archetype(&self) -> Archetype {
        Archetype::new(self.world, unsafe { ecs_get_type(self.world, self.raw_id) })
    }

    /// get the entity's type/table
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::table`
    #[doc(alias = "entity_view::table")]
    #[inline(always)]
    pub fn get_table(&self) -> Table {
        Table::new(&self.get_world(), unsafe {
            ecs_get_table(self.world, self.raw_id)
        })
    }

    /// Get table range for the entity.
    /// # Returns
    /// Returns a range with the entity's row as offset and count set to 1. If
    /// the entity is not stored in a table, the function returns a range with
    /// count 0.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::range`
    #[doc(alias = "entity_view::range")]
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

    /// Iterate over component ids of an entity.
    ///
    /// # Arguments
    /// * `func` - The closure invoked for each matching ID. Must match the signature `FnMut(Id)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::each`
    #[doc(alias = "entity_view::each")]
    pub fn for_each_component<F>(&self, mut func: F)
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

            // Union object is not stored in type, so handle separately
            if unsafe { ecs_pair_first(id) == EcsUnion } {
                let ent = Id::new_world_pair(self.world, ecs_pair_second(id), unsafe {
                    ecs_get_target(self.world, self.raw_id, ecs_pair_second(self.raw_id), 0)
                });

                func(ent);
            }
        }
    }

    /// Iterates over matching pair IDs of an entity.
    ///
    /// # Arguments
    ///
    /// * `first` - The first ID to match against.
    /// * `second` - The second ID to match against.
    /// * `func` - The closure invoked for each matching ID. Must match the signature `FnMut(Id)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::each`
    #[doc(alias = "entity_view::each")]
    pub fn for_each_matching_pair<F>(&self, pred: IdT, obj: IdT, mut func: F)
    where
        F: FnMut(Id),
    {
        // this is safe because we are only reading the world
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
            let ent = Id::new_from_existing(self.world, unsafe { *(ids.add(cur as usize)) });
            func(ent);
            cur += 1;
        }
    }

    /// Iterate over targets for a given relationship.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship for which to iterate the targets.
    /// * `func` - The closure invoked for each target. Must match the signature `FnMut(Entity)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::each`
    #[doc(alias = "entity_view::each")]
    pub fn for_each_target_in_relationship_by_entity<F>(
        &self,
        relationship: EntityView,
        mut func: F,
    ) where
        F: FnMut(Entity),
    {
        self.for_each_matching_pair(relationship.raw_id, unsafe { EcsWildcard }, |id| {
            let obj = id.second();
            func(obj);
        });
    }

    /// Iterate over targets for a given relationship.
    ///
    /// # Type Parameters
    ///
    /// * `Relationship` - The relationship for which to iterate the targets.
    ///
    /// # Arguments
    ///
    /// * `func` - The function invoked for each target.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::each`
    #[doc(alias = "entity_view::each")]
    pub fn for_each_target_in_relationship<T, F>(&self, func: F)
    where
        T: CachedComponentData,
        F: FnMut(Entity),
    {
        self.for_each_target_in_relationship_by_entity(
            EntityView::new_id_only(T::get_id(self.world)),
            func,
        );
    }

    /// Iterate children for entity
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    pub fn for_each_children_by_relationship_id<F>(&self, relationship: EntityT, mut func: F)
    where
        F: FnMut(Entity),
    {
        // When the entity is a wildcard, this would attempt to query for all
        //entities with (ChildOf, *) or (ChildOf, _) instead of querying for
        //the children of the wildcard entity.
        if unsafe { self.raw_id == EcsWildcard || self.raw_id == EcsAny } {
            // this is correct, wildcard entities don't have children
            return;
        }

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
                        //TODO should investigate if this is correct
                        let id = it.entities.add(i);
                        let ent = Entity::new_from_existing_raw(self.world, *id);
                        func(ent);
                    }
                }
            }
        }
    }

    /// Iterate children for entity
    ///
    /// # Arguments
    ///
    /// * T - The relationship to follow
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    pub fn for_each_children_by_relationship<T, F>(&self, func: F)
    where
        T: CachedComponentData,
        F: FnMut(Entity),
    {
        self.for_each_children_by_relationship_id(T::get_id(self.world), func);
    }

    /// Iterate children for entity
    /// This operation follows the `ChildOf` relationship.
    /// # Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    pub fn for_each_child_of<F>(&self, func: F)
    where
        F: FnMut(Entity),
    {
        self.for_each_children_by_relationship_id(unsafe { EcsChildOf }, func);
    }

    /// Get (struct) Component from entity
    /// use `.unwrap()` or `.unwrap_unchecked()` or `get_unchecked()` if you're sure the entity has the component
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to get
    ///
    /// # Returns
    ///
    /// * Option<&T> - The component, None if the entity does not have the component
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get<T: CachedComponentData + ComponentType<Struct>>(
        &self,
    ) -> Option<&T::UnderlyingType> {
        let component_id = T::get_id(self.world);
        unsafe {
            (ecs_get_id(self.world, self.raw_id, component_id) as *const T::UnderlyingType).as_ref()
        }
    }

    /// Get (struct) Component from entity unchecked
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to get
    ///
    /// # Returns
    ///
    /// * &T - The component
    ///
    /// # Safety
    ///
    /// if the entity does not have the component, this will cause a panic
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub unsafe fn get_unchecked<T: CachedComponentData + ComponentType<Struct>>(
        &self,
    ) -> &T::UnderlyingType {
        let component_id = T::get_id(self.world);
        let ptr = ecs_get_id(self.world, self.raw_id, component_id) as *const T::UnderlyingType;
        ecs_assert!(
            !ptr.is_null(),
            FlecsErrorCode::InternalError,
            "missing component {}",
            T::get_symbol_name()
        );
        &*ptr
    }

    /// Get enum constant.
    /// Use `.unwrap()` or `.unwrap_unchecked()` or `.get_enum_unchecked` if you're sure the entity has the component
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum component type which to get the constant
    ///
    /// # Returns
    ///
    /// * Option<&T> - The enum component, None if the entity does not have the component
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_enum<T: CachedComponentData + ComponentType<Enum>>(
        &self,
    ) -> Option<&T::UnderlyingType> {
        let component_id: IdT = T::get_id(self.world);
        let target: IdT = unsafe { ecs_get_target(self.world, self.raw_id, component_id, 0) };

        if target == 0 {
            // if there is no matching pair for (r,*), try just r
            unsafe {
                (ecs_get_id(self.world, self.raw_id, component_id) as *const T::UnderlyingType)
                    .as_ref()
            }
        } else {
            // get constant value from constant entity
            let constant_value =
                unsafe { ecs_get_id(self.world, target, component_id) as *const T::UnderlyingType };

            ecs_assert!(
                !constant_value.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );

            unsafe { constant_value.as_ref() }
        }
    }

    /// Get enum constant from entity unchecked
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to get
    ///
    /// # Returns
    ///
    /// * &T - The component
    ///
    /// # Safety
    ///
    /// if the entity does not have the component, this will cause a panic
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub unsafe fn get_enum_unchecked<T: CachedComponentData + ComponentType<Enum>>(
        &self,
    ) -> &T::UnderlyingType {
        let component_id: IdT = T::get_id(self.world);
        let target: IdT = ecs_get_target(self.world, self.raw_id, component_id, 0);

        if target == 0 {
            // if there is no matching pair for (r,*), try just r
            let ptr = ecs_get_id(self.world, self.raw_id, component_id) as *const T::UnderlyingType;
            ecs_assert!(
                !ptr.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );
            &*ptr
        } else {
            // get constant value from constant entity
            let constant_value =
                ecs_get_id(self.world, target, component_id) as *const T::UnderlyingType;
            ecs_assert!(
                !constant_value.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );
            &*constant_value
        }
    }

    /// Get component value as untyped pointer
    ///
    /// # Arguments
    ///
    /// * `component_id` - The component to get
    ///
    /// # Returns
    ///
    /// * `*const c_void` - Pointer to the component value, nullptr if the entity does not have the component
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_untyped(&self, component_id: IdT) -> *const c_void {
        unsafe { ecs_get_id(self.world, self.raw_id, component_id) }
    }

    /// get a pair as untyped pointer
    /// This operation gets the value for a pair from the entity. If neither the
    /// first nor the second part of the pair are components, the operation
    /// will fail.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair
    /// * `second` - The second element of the pair
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_untyped_pair(&self, first: EntityT, second: EntityT) -> *const c_void {
        unsafe { ecs_get_id(self.world, self.raw_id, ecs_pair(first, second)) }
    }

    /// Get target for a given pair.
    ///
    /// This operation returns the target for a given pair. The optional
    /// index can be used to iterate through targets, in case the entity `get_has`
    /// multiple instances for the same relationship.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `index` - The index (0 for the first instance of the relationship).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target")]
    pub fn get_target_from_component<First: CachedComponentData>(&self, index: i32) -> Entity {
        Entity::new_from_existing_raw(self.world, unsafe {
            ecs_get_target(self.world, self.raw_id, First::get_id(self.world), index)
        })
    }

    /// Get target for a given pair.
    ///
    /// This operation returns the target for a given pair. The optional
    /// index can be used to iterate through targets, in case the entity `get_has`
    /// multiple instances for the same relationship.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair for which to retrieve the target.
    /// * `index` - The index (0 for the first instance of the relationship).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target")]
    pub fn get_target_from_entity(&self, first: &Entity, index: i32) -> Entity {
        Entity::new_from_existing_raw(self.world, unsafe {
            ecs_get_target(self.world, self.raw_id, first.raw_id, index)
        })
    }

    /// Get the target of a pair for a given relationship id.
    ///
    /// This operation returns the first entity that has the provided id by following
    /// the specified relationship. If the entity itself has the id then entity will
    /// be returned. If the id cannot be found on the entity or by following the
    /// relationship, the operation will return 0.
    ///
    /// This operation can be used to lookup, for example, which prefab is providing
    /// a component by specifying the `IsA` pair:
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// // Is Position provided by the entity or one of its base entities?
    /// get_target_by_relationship_and_component_id(world, EcsIsA, T::get_id<Position>(world))
    /// ```
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow.
    /// * `id` - The id to lookup.
    ///
    /// # Returns
    ///
    /// * The entity for which the target `get_has` been found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target_for`
    #[doc(alias = "entity_view::target_for")]
    pub fn get_target_for_id(&self, relationship: &Entity, component_id: IdT) -> Entity {
        Entity::new_from_existing_raw(self.world, unsafe {
            ecs_get_target_for_id(self.world, self.raw_id, relationship.raw_id, component_id)
        })
    }

    /// Get the target for a given component and relationship.
    ///
    /// This function is a convenient wrapper around `get_target_by_relationship_and_component_id`,
    /// allowing callers to provide a type and automatically deriving the component id.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to use for deriving the id.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow.
    ///
    /// # Returns
    ///
    /// * The entity for which the target `get_has` been found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target")]
    #[inline(always)]
    pub fn get_target_for<T: CachedComponentData>(&self, relationship: &Entity) -> Entity {
        self.get_target_for_id(relationship, T::get_id(self.world))
    }

    /// Get the target for a given pair of components and a relationship.
    ///
    /// This function extends `get_target`, allowing callers to provide two component types.
    /// It retrieves the target entity for the combined pair of those component ids.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first component type to use for deriving the id.
    /// * `Second` - The second component type to use for deriving the id.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow.
    ///
    /// # Returns
    ///
    /// * The entity for which the target `get_has` been found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target")]
    #[inline(always)]
    pub fn get_target_for_pair<First: CachedComponentData, Second: CachedComponentData>(
        &self,
        relationship: &Entity,
    ) -> Entity {
        self.get_target_for_id(
            relationship,
            ecs_pair(First::get_id(self.world), Second::get_id(self.world)),
        )
    }
    // TODO this needs a better name and documentation, the rest of the cpp functions still have to be done as well
    // TODO, I removed the second template parameter and changed the fn parameter second to entityT, check validity
    /// Get the target for a given pair of components and a relationship.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first component type to use for deriving the id.
    ///
    /// # Arguments
    ///
    /// * `second` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// * The entity for which the target has been found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target")]
    pub fn get_target_for_pair_first<First: CachedComponentData>(
        &self,
        second: EntityT,
    ) -> *const First {
        let comp_id = First::get_id(self.world);
        ecs_assert!(
            //this is safe because the previous line guarantees registration
            unsafe { First::get_size_unchecked() != 0 },
            FlecsErrorCode::InvalidParameter,
            "First element is size 0"
        );
        unsafe { ecs_get_id(self.world, comp_id, ecs_pair(comp_id, second)) as *const First }
    }

    /// Get the depth for the given relationship.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship for which to get the depth.
    ///
    /// # Returns
    ///
    /// * The depth of the relationship.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::depth`
    #[doc(alias = "entity_view::depth")]
    #[inline(always)]
    pub fn get_depth_by_id(&self, relationship: EntityT) -> i32 {
        unsafe { ecs_get_depth(self.world, self.raw_id, relationship) }
    }

    /// Retrieves the depth for a specified relationship.
    ///
    /// This function is a convenient wrapper around `get_depth_by_id`, allowing callers
    /// to provide a type and automatically deriving the relationship id.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The relationship type to use for deriving the id.
    ///
    /// # Returns
    ///
    /// * The depth of the relationship.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::depth`
    #[doc(alias = "entity_view::depth")]
    #[inline(always)]
    pub fn get_depth<T: CachedComponentData>(&self) -> i32 {
        self.get_depth_by_id(T::get_id(self.world))
    }

    /// Retrieves the parent of the entity.
    ///
    /// This function is shorthand for getting the target using the `EcsChildOf` relationship.
    ///
    /// # Returns
    ///
    /// * The parent of the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::parent`
    #[doc(alias = "entity_view::parent")]
    #[inline(always)]
    pub fn get_parent(&self) -> Entity {
        self.get_target_from_entity(&ECS_CHILD_OF.into(), 0)
    }

    /// Lookup an entity by name.
    ///
    /// Lookup an entity in the scope of this entity. The provided path may
    /// contain double colons as scope separators, for example: "`Foo::Bar`".
    ///
    /// # Arguments
    ///
    /// * `path` - The name of the entity to lookup.
    /// * `search_path` - Whether to search the entire path or just the current scope.
    ///
    /// # Returns
    ///
    /// The found entity, or `Entity::null` if no entity matched.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    pub fn lookup_entity_by_name(&self, path: &CStr, search_path: bool) -> Option<Entity> {
        ecs_assert!(
            self.raw_id != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid lookup from null handle"
        );
        let id = unsafe {
            ecs_lookup_path_w_sep(
                self.world,
                self.raw_id,
                path.as_ptr(),
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                search_path,
            )
        };

        if id == 0 {
            None
        } else {
            Some(Entity::new_from_existing_raw(self.world, id))
        }
    }

    /// Check if entity has the provided entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided entity, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    #[inline(always)]
    pub fn has_id(&self, entity: IdT) -> bool {
        unsafe { ecs_has_id(self.world, self.raw_id, entity) }
    }

    /// Check if entity has the provided struct component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to check.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has<T: CachedComponentData + ComponentType<Struct>>(&self) -> bool {
        unsafe { ecs_has_id(self.world, self.raw_id, T::get_id(self.world)) }
    }

    /// Check if entity has the provided enum component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to check.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has_enum<T: CachedComponentData + ComponentType<Enum>>(&self) -> bool {
        let component_id: IdT = T::get_id(self.world);
        ecs_has_pair(self.world, self.raw_id, component_id, unsafe {
            EcsWildcard
        })
    }

    /// Check if entity has the provided enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `constant` - The enum constant to check.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided constant, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has_enum_constant<T>(&self, constant: T) -> bool
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        let component_id: IdT = T::get_id(self.world);
        // Safety: we know the enum fields are registered because of the previous T::get_id call
        let enum_constant_entity_id: IdT = constant.get_entity_id_from_enum_field(self.world);
        ecs_has_pair(
            self.world,
            self.raw_id,
            component_id,
            enum_constant_entity_id,
        )
    }

    /// Check if entity has the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The first element of the pair.
    /// * `U` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has_pair<T: CachedComponentData, U: CachedComponentData>(&self) -> bool {
        ecs_has_pair(
            self.world,
            self.raw_id,
            T::get_id(self.world),
            U::get_id(self.world),
        )
    }

    /// Check if entity has the provided pair.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair.
    /// * `second` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has_pair_ids(&self, first: IdT, second: IdT) -> bool {
        ecs_has_pair(self.world, self.raw_id, first, second)
    }

    /// Check if entity has the provided pair with an enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The first element of the pair.
    /// * `U` - The second element of the pair as an enum constant.
    ///
    /// # Arguments
    ///
    /// * `constant` - The enum constant.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
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

    /// Check if the entity owns the provided entity.
    /// An entity is owned if it is not shared from a base entity.
    ///
    /// # Arguments
    /// - `entity_id`: The entity to check.
    ///
    /// # Returns
    /// - `true` if the entity owns the provided entity, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn is_owner_of_id(&self, entity_id: IdT) -> bool {
        unsafe { ecs_owns_id(self.world, self.raw_id, entity_id) }
    }

    /// Check if the entity owns the provided entity.
    ///
    /// # Arguments
    /// - `entity`: The entity to check.
    ///
    /// # Returns
    /// - `true` if the entity owns the provided entity, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn is_owner_of_entity(&self, entity: Entity) -> bool {
        unsafe { ecs_owns_id(self.world, self.raw_id, entity.raw_id) }
    }

    /// Check if the entity owns the provided component.
    /// A component is owned if it is not shared from a base entity.
    ///
    /// # Type Parameters
    /// - `T`: The component to check.
    ///
    /// # Returns
    /// - `true` if the entity owns the provided component, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn is_owner_of<T: CachedComponentData>(&self) -> bool {
        unsafe { ecs_owns_id(self.world, self.raw_id, T::get_id(self.world)) }
    }

    /// Check if the entity owns the provided pair.
    ///
    /// # Arguments
    /// - `first`: The first element of the pair.
    /// - `second`: The second element of the pair.
    ///
    /// # Returns
    /// - `true` if the entity owns the provided pair, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn is_owner_of_pair_ids(&self, first: IdT, second: IdT) -> bool {
        unsafe { ecs_owns_id(self.world, self.raw_id, ecs_pair(first, second)) }
    }

    /// Check if the entity owns the provided pair.
    ///
    /// # Type Parameters
    /// - `T`: The first element of the pair.
    /// - `U`: The second element of the pair.
    ///
    /// # Returns
    /// - `true` if the entity owns the provided pair, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn is_owner_of_pair<T: CachedComponentData, U: CachedComponentData>(&self) -> bool {
        unsafe {
            ecs_owns_id(
                self.world,
                self.raw_id,
                ecs_pair(T::get_id(self.world), U::get_id(self.world)),
            )
        }
    }

    /// Test if id is enabled.
    ///
    /// # Arguments
    /// - `id`: The id to test.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_id(&self, id: IdT) -> bool {
        unsafe { ecs_is_enabled_id(self.world, self.raw_id, id) }
    }

    /// Test if component is enabled.
    ///
    /// # Type Parameters
    /// - `T`: The component to test.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_component<T: CachedComponentData>(&self) -> bool {
        unsafe { ecs_is_enabled_id(self.world, self.raw_id, T::get_id(self.world)) }
    }

    /// Test if pair is enabled.
    ///
    /// # Arguments
    /// - `first`: The first element of the pair.
    /// - `second`: The second element of the pair.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_pair_ids(&self, first: IdT, second: IdT) -> bool {
        unsafe { ecs_is_enabled_id(self.world, self.raw_id, ecs_pair(first, second)) }
    }

    /// Test if pair is enabled.
    ///
    /// # Type Parameters
    /// - `T`: The first element of the pair.
    /// - `U`: The second element of the pair.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_pair<T: CachedComponentData, U: CachedComponentData>(&self) -> bool {
        self.is_enabled_pair_ids(T::get_id(self.world), U::get_id(self.world))
    }

    /// Test if pair is enabled.
    ///
    /// # Type Parameters
    /// - `T`: The first element of the pair.
    ///
    /// # Arguments
    /// - `second`: The second element of the pair.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_pair_first<T: CachedComponentData>(&self, second: IdT) -> bool {
        self.is_enabled_pair_ids(T::get_id(self.world), second)
    }

    /// Test if pair is enabled.
    ///
    /// # Type Parameters
    /// - `T`: The second element of the pair.
    ///
    /// # Arguments
    /// - `first`: The second element of the pair.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_pair_second<U: CachedComponentData>(&self, first: IdT) -> bool {
        self.is_enabled_pair_ids(first, U::get_id(self.world))
    }

    /// Clones the current entity to a new or specified entity.
    ///
    /// This function creates a clone of the current entity. If `dest_id` is provided
    /// (i.e., not zero), it will clone the current entity to the entity with `dest_id`.
    /// If `dest_id` is zero, it will create a new entity and clone the current entity
    /// to the newly created entity.
    ///
    /// If `copy_value` is set to `true`, the value of the current entity is also copied to
    /// the destination entity. Otherwise, only the entity's structure is cloned without copying the value.
    ///
    /// # Arguments
    /// - `copy_value`: A boolean indicating whether to copy the entity's value to the destination entity.
    /// - `dest_id`: The identifier of the destination entity. If zero, a new entity is created.
    ///
    /// # Returns
    /// - An `Entity` object representing the destination entity.
    ///
    /// ## Safety
    /// This function makes use of `unsafe` operations to interact with the underlying ECS.
    /// Ensure that the provided `dest_id` is valid or zero
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::clone`
    #[doc(alias = "entity_view::clone")]
    #[inline(always)]
    pub fn clone(&self, copy_value: bool, mut dest_id: EntityT) -> Entity {
        if dest_id == 0 {
            dest_id = unsafe { ecs_new_id(self.world) };
        }

        let dest_entity = Entity::new_from_existing_raw(self.world, dest_id);
        unsafe { ecs_clone(self.world, dest_id, self.raw_id, copy_value) };
        dest_entity
    }

    /// Returns a mutable entity handle for the current stage.
    ///
    /// When an entity handle created from the world is used while the world is
    /// in staged mode, it will only allow for readonly operations since
    /// structural changes are not allowed on the world while in staged mode.
    ///
    /// To perform mutations on the entity, this operation provides a handle to the
    /// entity that uses the stage instead of the actual world.
    ///
    /// Note that staged entity handles should never be stored persistently, in
    /// components or elsewhere. An entity handle should always point to the
    /// main world.
    ///
    /// Also note that this operation is not necessary when doing mutations on an
    /// entity outside of a system. It is allowed to perform entity operations
    /// directly on the world, as long as the world is not in staged mode.
    ///
    /// # Arguments
    /// - `stage`: The current stage.
    ///
    /// # Returns
    /// - An entity handle that allows for mutations in the current stage.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::mut`
    #[doc(alias = "entity_view::mut")]
    pub fn get_mutable_handle_for_stage(&self, stage: &World) -> Entity {
        ecs_assert!(
            !stage.is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use readonly world/stage to create mutable handle"
        );

        Entity::new_from_existing_raw(stage.raw_world, self.raw_id)
    }

    /// Returns a mutable entity handle for the current stage from another entity.
    ///
    /// This operation allows for the construction of a mutable entity handle
    /// from another entity. This is useful in `each` functions, which only
    /// provide a handle to the entity being iterated over.
    ///
    /// # Arguments
    /// - `entity`: Another mutable entity.
    ///
    /// # Returns
    /// - An entity handle that allows for mutations in the current stage.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::mut`
    #[doc(alias = "entity_view::mut")]
    pub fn get_mutable_handle_from_entity(&self, entity: &EntityView) -> Entity {
        ecs_assert!(
            !entity.get_world().is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use entity created for readonly world/stage to create mutable handle"
        );

        Entity::new_from_existing_raw(entity.world, self.raw_id)
    }

    //might not be needed, in the original c++ impl it was used in the get_mut functions.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::set_stage`
    #[doc(alias = "entity_view::set_stage")]
    #[doc(hidden)]
    fn set_stage(&self, stage: *mut WorldT) -> Entity {
        Entity::new_from_existing_raw(stage, self.raw_id)
    }
}

// Event mixin
impl EntityView {
    /// Emit event for entity
    ///
    /// # Arguments
    ///
    /// * event - the event to emit
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::emit`
    #[doc(alias = "entity_view::emit")]
    pub fn emit_id(&self, event: &Entity) {
        self.get_world()
            .event_id(event)
            .set_entity_to_emit(&self.entity())
            .emit();
    }

    /// Emit event for entity
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to emit. Type must be empty.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::emit`
    #[doc(alias = "entity_view::emit")]
    pub fn emit<T: EventData + EmptyComponent>(&self) {
        self.emit_id(&T::UnderlyingType::get_id(self.world).into());
    }

    /// Emit event with payload for entity.
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to emit. Type must contain data (not empty struct).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::emit`
    #[doc(alias = "entity_view::emit")]
    pub fn emit_payload<T: EventData + NotEmptyComponent>(&self, payload: &mut T) {
        self.get_world()
            .event::<T>()
            .set_entity_to_emit(&self.entity())
            .set_event_data(payload)
            .emit();
    }

    /// Enqueue event for entity.
    ///
    /// # Arguments
    ///
    /// * event - the event to enqueue
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    pub fn enqueue_id(&self, event: &Entity) {
        self.get_world()
            .event_id(event)
            .set_entity_to_emit(&self.entity())
            .enqueue();
    }

    /// Enqueue event for entity
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to enqueue. Type must be empty.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    pub fn enqueue<T: EventData + EmptyComponent>(&self) {
        self.enqueue_id(&T::UnderlyingType::get_id(self.world).into());
    }

    /// enqueue event with payload for entity.
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to enqueue. Type must contain data (not empty struct).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    pub fn enqueue_payload<T: EventData + NotEmptyComponent>(&self, payload: &mut T) {
        self.get_world()
            .event::<T>()
            .set_entity_to_emit(&self.entity())
            .set_event_data(payload)
            .enqueue();
    }
}

// Event/Observe mixin
impl EntityView {
    /// Register the callback for the entity observer for empty events.
    ///
    /// The "empty" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func()`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe<C>(&self, func: impl FnMut()) -> &Self
    where
        C: EventData + CachedComponentData + EmptyComponent,
    {
        self.observe_impl::<C, _>(func)
    }

    fn observe_impl<C, Func>(&self, func: Func) -> &Self
    where
        Func: FnMut(),
        C: EventData + CachedComponentData,
    {
        let new_binding_ctx = Box::<super::ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.empty = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_empty = Some(Self::on_free_empty);

        Self::entity_observer_create(
            self.world,
            C::UnderlyingType::get_id(self.world),
            self.raw_id,
            binding_ctx,
            Some(Self::run_empty::<Func> as unsafe extern "C" fn(_)),
        );
        self
    }

    /// Register the callback for the entity observer for empty events with entity parameter.
    ///
    /// The `empty_entity` iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut Entity)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe_entity<C>(&self, func: impl FnMut(&mut Entity)) -> &Self
    where
        C: EventData + CachedComponentData + EmptyComponent,
    {
        self.observe_entity_impl::<C, _>(func)
    }

    fn observe_entity_impl<C, Func>(&self, func: Func) -> &Self
    where
        Func: FnMut(&mut Entity),
        C: EventData + CachedComponentData,
    {
        let new_binding_ctx = Box::<super::ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.empty_entity = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_empty_entity = Some(Self::on_free_empty_entity);

        Self::entity_observer_create(
            self.world,
            C::UnderlyingType::get_id(self.world),
            self.raw_id,
            binding_ctx,
            Some(Self::run_empty_entity::<Func> as unsafe extern "C" fn(_)),
        );
        self
    }

    /// Register the callback for the entity observer for `payload` events.
    ///
    /// The "payload" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut EventData)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe_payload<C>(&self, func: impl FnMut(&mut C)) -> &Self
    where
        C: EventData + CachedComponentData + NotEmptyComponent,
    {
        self.observe_payload_impl::<C, _>(func)
    }

    fn observe_payload_impl<C, Func>(&self, func: Func) -> &Self
    where
        Func: FnMut(&mut C),
        C: EventData + CachedComponentData,
    {
        let new_binding_ctx = Box::<super::ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.payload = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_payload = Some(Self::on_free_payload::<C>);

        Self::entity_observer_create(
            self.world,
            C::UnderlyingType::get_id(self.world),
            self.raw_id,
            binding_ctx,
            Some(Self::run_payload::<C, Func> as unsafe extern "C" fn(_)),
        );
        self
    }

    /// Register the callback for the entity observer for an event with payload and entity parameter.
    ///
    /// The "payload" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut Entity, &mut EventData)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe_payload_entity<C>(&self, func: impl FnMut(&mut Entity, &mut C)) -> &Self
    where
        C: EventData + CachedComponentData + NotEmptyComponent,
    {
        self.observe_payload_entity_impl::<C, _>(func)
    }

    fn observe_payload_entity_impl<C, Func>(&self, func: Func) -> &Self
    where
        Func: FnMut(&mut Entity, &mut C),
        C: EventData + CachedComponentData,
    {
        let new_binding_ctx = Box::<super::ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.payload_entity = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_payload_entity = Some(Self::on_free_payload_entity::<C>);

        Self::entity_observer_create(
            self.world,
            C::UnderlyingType::get_id(self.world),
            self.raw_id,
            binding_ctx,
            Some(Self::run_payload_entity::<C, Func> as unsafe extern "C" fn(_)),
        );
        self
    }
}

// entity observer creation
impl EntityView {
    pub(crate) fn entity_observer_create(
        world: *mut WorldT,
        event: EntityT,
        entity: EntityT,
        binding_ctx: *mut ObserverEntityBindingCtx,
        callback: ecs_iter_action_t,
    ) {
        let mut desc = ecs_observer_desc_t::default();
        desc.events[0] = event;
        desc.filter.terms[0].id = ECS_ANY;
        desc.filter.terms[0].src.id = entity;
        desc.callback = callback;
        desc.binding_ctx = binding_ctx as *mut c_void;
        desc.binding_ctx_free = Some(Self::binding_entity_ctx_drop);

        let observer = unsafe { ecs_observer_init(world, &desc) };
        ecs_add_pair(world, observer, ECS_CHILD_OF, entity);
    }

    /// Callback of the observe functionality
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator which gets passed in from `C`
    ///
    /// # See also
    ///
    /// * C++ API: `entity_observer_delegate::invoke`
    #[doc(alias = "entity_observer_delegate::invoke")]
    pub(crate) unsafe extern "C" fn run_empty<Func>(iter: *mut IterT)
    where
        Func: FnMut(),
    {
        let ctx: *mut ObserverEntityBindingCtx = (*iter).binding_ctx as *mut _;
        let empty = (*ctx).empty.unwrap();
        let empty = &mut *(empty as *mut Func);
        let iter_count = (*iter).count as usize;

        ecs_table_lock((*iter).world, (*iter).table);

        for _i in 0..iter_count {
            empty();
        }

        ecs_table_unlock((*iter).world, (*iter).table);
    }

    /// Callback of the observe functionality
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator which gets passed in from `C`
    ///
    /// # See also
    ///
    /// * C++ API: `entity_observer_delegate::invoke`
    #[doc(alias = "entity_observer_delegate::invoke")]
    pub(crate) unsafe extern "C" fn run_empty_entity<Func>(iter: *mut IterT)
    where
        Func: FnMut(&mut Entity),
    {
        let ctx: *mut ObserverEntityBindingCtx = (*iter).binding_ctx as *mut _;
        let empty = (*ctx).empty_entity.unwrap();
        let empty = &mut *(empty as *mut Func);
        let iter_count = (*iter).count as usize;

        ecs_table_lock((*iter).world, (*iter).table);

        for _i in 0..iter_count {
            empty(&mut Entity::new_from_existing_raw(
                (*iter).world,
                ecs_field_src(iter, 1),
            ));
        }

        ecs_table_unlock((*iter).world, (*iter).table);
    }

    /// Callback of the observe functionality
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator which gets passed in from `C`
    ///
    /// # See also
    ///
    /// * C++ API: `entity_payload_observer_delegate::invoke`
    #[doc(alias = "entity_payload_observer_delegate::invoke")]
    pub(crate) unsafe extern "C" fn run_payload<C, Func>(iter: *mut IterT)
    where
        Func: FnMut(&mut C),
    {
        let ctx: *mut ObserverEntityBindingCtx = (*iter).binding_ctx as *mut _;
        let empty = (*ctx).payload.unwrap();
        let empty = &mut *(empty as *mut Func);
        let iter_count = (*iter).count as usize;

        ecs_table_lock((*iter).world, (*iter).table);

        for _i in 0..iter_count {
            let data = (*iter).param as *mut C;
            let data_ref = &mut *data;
            empty(data_ref);
        }

        ecs_table_unlock((*iter).world, (*iter).table);
    }

    /// Callback of the observe functionality
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator which gets passed in from `C`
    ///
    /// # See also
    ///
    /// * C++ API: `entity_payload_observer_delegate::invoke`
    #[doc(alias = "entity_payload_observer_delegate::invoke")]
    pub(crate) unsafe extern "C" fn run_payload_entity<C, Func>(iter: *mut IterT)
    where
        Func: FnMut(&mut Entity, &mut C),
    {
        let ctx: *mut ObserverEntityBindingCtx = (*iter).binding_ctx as *mut _;
        let empty = (*ctx).payload_entity.unwrap();
        let empty = &mut *(empty as *mut Func);
        let iter_count = (*iter).count as usize;

        ecs_table_lock((*iter).world, (*iter).table);

        for _i in 0..iter_count {
            let data = (*iter).param as *mut C;
            let data_ref = &mut *data;
            empty(
                &mut Entity::new_from_existing_raw((*iter).world, ecs_field_src(iter, 1)),
                data_ref,
            );
        }

        ecs_table_unlock((*iter).world, (*iter).table);
    }

    /// Callback to free the memory of the `empty` callback
    pub(crate) extern "C" fn on_free_empty(ptr: *mut c_void) {
        let ptr_func: *mut fn() = ptr as *mut fn();
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Callback to free the memory of the `empty_entity` callback
    pub(crate) extern "C" fn on_free_empty_entity(ptr: *mut c_void) {
        let ptr_func: *mut fn(&mut Entity) = ptr as *mut fn(&mut Entity);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Callback to free the memory of the `payload` callback
    pub(crate) extern "C" fn on_free_payload<C>(ptr: *mut c_void) {
        let ptr_func: *mut fn(&mut C) = ptr as *mut fn(&mut C);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Callback to free the memory of the `payload_entity` callback
    pub(crate) extern "C" fn on_free_payload_entity<C>(ptr: *mut c_void) {
        let ptr_func: *mut fn(&mut Entity, &mut C) = ptr as *mut fn(&mut Entity, &mut C);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Executes the drop for the system binding context, meant to be used as a callback
    pub(crate) extern "C" fn binding_entity_ctx_drop(ptr: *mut c_void) {
        let ptr_struct: *mut ObserverEntityBindingCtx = ptr as *mut ObserverEntityBindingCtx;
        unsafe {
            ptr::drop_in_place(ptr_struct);
        }
    }
}
