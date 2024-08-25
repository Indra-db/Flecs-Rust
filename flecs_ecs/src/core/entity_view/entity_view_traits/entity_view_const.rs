use std::ffi::c_void;
use std::ffi::CStr;
use std::ptr::NonNull;

use entity_view::path_from_id_default_sep;
use entity_view::try_lookup_impl;

use crate::core::*;
use crate::sys;

use super::entity_id::EntityId;

pub trait EntityViewConst<'w>: WorldProvider<'w> + EntityId + Sized {
    /// Get the [`IdView`] representation of the `entity_view`.
    fn id_view(&self) -> IdView<'w> {
        IdView::new_from(self.world(), *self.entity_id())
    }

    /// checks if entity is valid
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity();
    /// let entity_rust_copy = entity; //this is a rust copy, not an entity clone, these have the same id.
    /// let entity_cloned = entity.duplicate(false); //this is an entity clone, these have different ids.
    ///
    /// assert!(entity.is_valid());
    /// assert!(entity_rust_copy.is_valid());
    /// assert!(entity_cloned.is_valid());
    ///
    /// entity.destruct(); //takes self, entity becomes not useable
    ///
    /// assert!(!entity_rust_copy.is_valid());
    /// assert!(entity_cloned.is_valid());
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::is_valid`
    #[doc(alias = "entity_view::is_valid")]
    fn is_valid(&self) -> bool {
        unsafe { sys::ecs_is_valid(self.world_ptr(), *self.entity_id()) }
    }

    /// Checks if entity is alive.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::is_alive`
    #[doc(alias = "entity_view::is_alive")]
    fn is_alive(&self) -> bool {
        unsafe { sys::ecs_is_alive(self.world_ptr(), *self.entity_id()) }
    }

    /// Returns the entity name.
    ///
    /// if the entity has no name, this will return an empty string
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    #[doc(alias = "entity_view::name")]
    fn name(self) -> String {
        self.get_name().unwrap_or("".to_string())
    }

    /// Returns the entity name.
    ///
    /// if the entity has no name, this will return none
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    #[doc(alias = "entity_view::name")]
    fn get_name(self) -> Option<String> {
        // self.get_name_cstr().and_then(|s| s.to_str().ok())
        let cstr =
            NonNull::new(
                unsafe { sys::ecs_get_name(self.world().world_ptr(), *self.entity_id()) } as *mut _,
            )
            .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) });
        cstr.and_then(|s| s.to_str().ok().map(|s| s.to_string()))
    }

    // /// Returns the entity name as a `CStr`.
    // ///
    // /// if the entity has no name, this will return an empty string
    // ///
    // /// # See also
    // ///
    // /// * C++ API: `entity_view::name`
    // pub fn name_cstr(self) -> &'a CStr {
    //     self.get_name_cstr().unwrap_or(c"")
    // }

    // /// Returns the entity name as a `CStr`.
    // ///
    // /// If the entity has no name, this will return `None`.
    // ///
    // /// # See also
    // ///
    // /// * C++ API: `entity_view::name`
    // pub fn get_name_cstr(self) -> Option<*const CStr> {
    //     NonNull::new(unsafe { sys::ecs_get_name(self.world.world_ptr(), *self.id) } as *mut _)
    //         .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) })
    // }

    // /// Returns the entity symbol.
    // ///
    // /// # See also
    // ///
    // /// * C++ API: `entity_view::symbol`
    // #[doc(alias = "entity_view::symbol")]
    // pub fn symbol_cstr(self) -> &'a CStr {
    //     unsafe { CStr::from_ptr(sys::ecs_get_symbol(self.world.world_ptr(), *self.id)) }
    // }

    /// Returns the entity symbol.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::symbol`
    #[doc(alias = "entity_view::symbol")]
    fn symbol(self) -> String {
        //self.symbol_cstr().to_str().unwrap()
        let cstr =
            unsafe { CStr::from_ptr(sys::ecs_get_symbol(self.world_ptr(), *self.entity_id())) };
        cstr.to_str()
            .ok()
            .map(|s| s.to_string())
            .unwrap_or_default()
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
    fn path_w_sep(self, sep: &str, init_sep: &str) -> Option<String> {
        self.path_from_id_w_sep(0, sep, init_sep)
    }

    /// Return the hierarchical entity path using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path`
    #[doc(alias = "entity_view::path")]
    fn path(self) -> Option<String> {
        self.path_from_id(0)
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
    fn path_from_id_w_sep(
        &self,
        parent: impl Into<Entity>,
        sep: &str,
        init_sep: &str,
    ) -> Option<String> {
        let sep = compact_str::format_compact!("{}\0", sep);
        let init_sep = compact_str::format_compact!("{}\0", init_sep);

        NonNull::new(unsafe {
            sys::ecs_get_path_w_sep(
                self.world_ptr(),
                *parent.into(),
                *self.entity_id(),
                sep.as_ptr() as *const _,
                init_sep.as_ptr() as *const _,
            )
        })
        .map(|s| unsafe {
            let len = CStr::from_ptr(s.as_ptr()).to_bytes().len();
            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            String::from_utf8_unchecked(Vec::from_raw_parts(s.as_ptr() as *mut u8, len, len))
        })
    }

    /// Return the hierarchical entity path relative to a parent id using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path_from`
    #[doc(alias = "entity_view::path_from")]
    fn path_from_id(self, parent: impl Into<Entity>) -> Option<String> {
        NonNull::new(unsafe {
            sys::ecs_get_path_w_sep(
                self.world_ptr(),
                *parent.into(),
                *self.entity_id(),
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
            )
        })
        .map(|s| unsafe {
            let len = CStr::from_ptr(s.as_ptr()).to_bytes().len();

            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            String::from_utf8_unchecked(Vec::from_raw_parts(s.as_ptr() as *mut u8, len, len))
        })
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
    fn path_from<T: ComponentId>(self) -> Option<String> {
        let world = self.world();
        path_from_id_default_sep(world.world_ptr(), self.entity_id(), T::id(world))
    }

    fn path_from_w_sep<T: ComponentId>(&self, sep: &str, init_sep: &str) -> Option<String> {
        self.path_from_id_w_sep(T::id(self.world()), sep, init_sep)
    }

    /// Return the hierarchical entity path relative to a parent type using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path_from`
    #[doc(alias = "entity_view::path_from")]
    fn hierarchy_path_from_parent_type<T: ComponentId>(self) -> Option<String> {
        let world = self.world();
        self.path_from_id(T::id(world))
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
    fn is_enabled_self(&self) -> bool {
        unsafe { !sys::ecs_has_id(self.world_ptr(), *self.entity_id(), flecs::Disabled::ID) }
    }

    /// Get the entity's archetype.
    ///
    /// # See also
    ///
    /// * [`Table::archetype()`]
    /// * C++ API: `entity_view::type`
    #[doc(alias = "entity_view::type")]
    #[inline(always)]
    fn archetype(self) -> Archetype<'w> {
        let world = self.world();
        self.table()
            .map(|t| t.archetype())
            .unwrap_or(Archetype::new(world, &[]))
    }

    /// Get the entity's type/table.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::table`
    #[doc(alias = "entity_view::table")]
    #[inline(always)]
    fn table(self) -> Option<Table<'w>> {
        NonNull::new(unsafe { sys::ecs_get_table(self.world_ptr(), *self.entity_id()) })
            .map(|t| Table::new(self.world(), t))
    }

    /// Get table range for the entity.
    ///
    /// # Returns
    ///
    /// Returns a range with the entity's row as offset and count set to 1. If
    /// the entity is not stored in a table, the function returns `None`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::range`
    #[doc(alias = "entity_view::range")]
    #[inline]
    fn range(self) -> Option<TableRange<'w>> {
        NonNull::new(unsafe { sys::ecs_record_find(self.world_ptr(), *self.entity_id()) }).map(
            |record| unsafe {
                TableRange::new_raw(
                    self.world(),
                    NonNull::new_unchecked((*record.as_ptr()).table),
                    ecs_record_to_row((*record.as_ptr()).row),
                    1,
                )
            },
        )
    }

    /// Iterate over component ids of an entity.
    ///
    /// # Arguments
    /// * `func` - The closure invoked for each matching ID. Must match the signature `FnMut(IdView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::each`
    #[doc(alias = "entity_view::each")]
    fn each_component(self, mut func: impl FnMut(IdView)) {
        let world = self.world();
        let archetype = self.archetype();
        for &id in archetype.as_slice() {
            let ent = IdView::new_from(world, id);
            func(ent);
        }
    }

    /// Iterates over matching pair IDs of an entity.
    ///
    /// # Arguments
    ///
    /// * `first` - The first ID to match against.
    /// * `second` - The second ID to match against.
    /// * `func` - The closure invoked for each matching ID. Must match the signature `FnMut(IdView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::each`
    #[doc(alias = "entity_view::each")]
    fn each_pair(
        &self,
        first: impl Into<Entity>,
        second: impl Into<Entity>,
        mut func: impl FnMut(IdView),
    ) {
        // this is safe because we are only reading the world
        let real_world = self.world().real_world();

        let Some(table) =
            NonNull::new(unsafe { sys::ecs_get_table(real_world.world_ptr(), *self.entity_id()) })
        else {
            return;
        };

        let table = Table::new(real_world, table);

        let mut pattern: sys::ecs_id_t = *first.into();
        let second_id = *second.into();
        if second_id != 0 {
            pattern = ecs_pair(pattern, second_id);
        }

        let mut cur: i32 = 0;
        let archetype = table.archetype();
        let ids = archetype.as_slice();

        while {
            cur = unsafe {
                sys::ecs_search_offset(
                    real_world.world_ptr(),
                    table.table_ptr_mut(),
                    cur,
                    pattern,
                    &mut 0,
                )
            };
            cur != -1
        } {
            let ent = IdView::new_from(self.world(), ids[cur as usize]);
            func(ent);
            cur += 1;
        }
    }

    /// Iterate over targets for a given relationship.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship for which to iterate the targets.
    /// * `func` - The closure invoked for each target. Must match the signature `FnMut(EntityView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::each`
    #[doc(alias = "entity_view::each")]
    fn each_target_id(self, relationship: impl Into<Entity>, mut func: impl FnMut(EntityView)) {
        self.each_pair(relationship.into(), ECS_WILDCARD, |id| {
            let obj = id.second_id();
            func(obj);
        });
    }

    /// Get the count of targets for a given relationship.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship for which to get the target count.
    ///
    /// # Returns
    ///
    /// The count of targets for the given relationship.
    /// If it doesn't have the relationship, this function will return `None`.
    fn target_id_count(self, relationship: impl Into<Entity>) -> Option<i32> {
        let world = self.world().real_world().ptr_mut();
        let id = ecs_pair(*relationship.into(), ECS_WILDCARD);
        let table = unsafe { sys::ecs_get_table(self.world_ptr(), *self.entity_id()) };

        let count = unsafe { sys::ecs_rust_rel_count(world, id, table) };

        if count == -1 {
            None
        } else {
            Some(count)
        }
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
    fn each_target<T>(self, func: impl FnMut(EntityView))
    where
        T: ComponentId,
    {
        let world = self.world();
        self.each_target_id(EntityView::new_from(world, T::id(world)), func);
    }

    /// Get the count of targets for a given relationship.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The relationship for which to get the target count.
    ///
    /// # Returns
    ///
    /// The count of targets for the given relationship.
    /// If it doesn't have the relationship, this function will return `None`.
    fn each_target_count<T>(self) -> Option<i32>
    where
        T: ComponentId,
    {
        let world = self.world();
        self.target_id_count(T::id(world))
    }

    /// Iterate children for entity
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    fn each_child_of_id(self, relationship: impl Into<Entity>, mut func: impl FnMut(EntityView)) {
        // When the entity is a wildcard, this would attempt to query for all
        //entities with (ChildOf, *) or (ChildOf, _) instead of querying for
        //the children of the wildcard entity.
        if self.entity_id() == flecs::Wildcard::ID || self.entity_id() == flecs::Any::ID {
            // this is correct, wildcard entities don't have children
            return;
        }

        let mut it: sys::ecs_iter_t = unsafe {
            sys::ecs_each_id(
                self.world_ptr(),
                ecs_pair(*relationship.into(), *self.entity_id()),
            )
        };
        while unsafe { sys::ecs_each_next(&mut it) } {
            for i in 0..it.count as usize {
                unsafe {
                    let id = it.entities.add(i);
                    let ent = EntityView::new_from(self.world(), *id);
                    func(ent);
                }
            }
        }
    }

    /// Iterate children for entity
    ///
    /// # Arguments
    ///
    /// * T - The relationship to follow
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    fn each_child_of<T>(self, func: impl FnMut(EntityView))
    where
        T: ComponentId,
    {
        let world = self.world();
        self.each_child_of_id(T::id(world), func);
    }

    /// Iterate children for entity
    /// This operation follows the `ChildOf` relationship.
    /// # Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    fn each_child(self, func: impl FnMut(EntityView)) {
        self.each_child_of_id(flecs::ChildOf::ID, func);
    }

    /// Clones components and/or relationship(s) from an entity and returns it.
    /// each component type must be marked `&`. This helps Rust type checker to determine if it's a relationship.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot clone single component tags with this function.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    ///
    /// # Panics
    ///
    /// - `cloned` assumes when not using `Option` wrapper, that the entity has the component.
    ///   This will panic if the entity does not have the component. If unsure, use `Option` wrapper or `try_cloned` function instead.
    ///   `try_cloned` will return a `None` tuple instead if the entity does not have the component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)] struct Tag;
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity()
    ///                   .set(Position { x: 10.0, y: 20.0 })
    ///                   .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///    
    /// let pos = entity.cloned::<&Position>();
    /// assert_eq!(pos.x, 10.0);
    ///
    /// let (vel, pos) = entity.cloned::<(Option<&Velocity>, &Position)>();
    /// assert_eq!(pos.x, 10.0);
    /// assert!(vel.is_none());
    ///
    /// let (tag_pos_rel, pos) = entity.cloned::<(&(Tag,Position), &Position)>();
    /// assert_eq!(pos.x, 10.0);
    /// assert_eq!(tag_pos_rel.x, 30.0);
    /// ```
    fn cloned<T: ClonedTuple>(self) -> T::TupleType<'w> {
        let record = unsafe { sys::ecs_record_find(self.world_ptr(), *self.entity_id()) };

        if unsafe { (*record).table.is_null() } {
            panic!("Entity does not have any components");
        }

        let tuple_data = T::create_ptrs::<true>(self.world(), record);
        tuple_data.get_tuple()
    }

    /// Clones components and/or relationship(s) from an entity and returns an `Option`.
    /// `None` if the entity does not have all components that are not marked `Option`, otherwise `Some(tuple)`.
    /// each component type must be marked `&`. This helps Rust type checker to determine if it's a relationship.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot clone single component tags with this function.
    /// - You can only clone relationships with a payload, so where one is not a tag / not a zst.
    ///
    /// # Returns
    ///
    /// - `Some(tuple)` if the entity has all components, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)] struct Tag;
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity()
    ///                   .set(Position { x: 10.0, y: 20.0 })
    ///                   .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///    
    /// let pos = entity.try_cloned::<&Position>();
    /// assert!(pos.is_some());
    /// assert_eq!(pos.unwrap().x, 10.0);
    ///
    /// let (vel, pos) = entity.try_cloned::<(Option<&Velocity>, &Position)>().unwrap();
    /// assert_eq!(pos.x, 10.0);
    /// assert!(vel.is_none());
    ///
    /// let (tag_pos_rel, pos) = entity.try_cloned::<(&(Tag,Position), &Position)>().unwrap();
    /// assert_eq!(pos.x, 10.0);
    /// assert_eq!(tag_pos_rel.x, 30.0);
    /// ```
    fn try_cloned<T: ClonedTuple>(self) -> Option<T::TupleType<'w>> {
        let record = unsafe { sys::ecs_record_find(self.world_ptr(), *self.entity_id()) };

        if unsafe { (*record).table.is_null() } {
            return None;
        }

        let tuple_data = T::create_ptrs::<false>(self.world(), record);
        //todo we can maybe early return if we don't yet if doesn't have all. Same for try_get
        let has_all_components = tuple_data.has_all_components();

        if has_all_components {
            Some(tuple_data.get_tuple())
        } else {
            None
        }
    }

    /// Get component value or pair as untyped pointer
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
    fn get_untyped(self, component_id: impl IntoId) -> *const c_void {
        unsafe { sys::ecs_get_id(self.world_ptr(), *self.entity_id(), *component_id.into()) }
    }
    /// Get mutable component value or pair (untyped).
    /// This operation returns a mutable ref to the component. If a base entity had
    /// the component, it will be overridden, and the value of the base component
    /// will be copied to the entity before this function returns.
    ///
    /// # Arguments
    ///
    /// * `comp`: The component to get.
    ///
    /// # Returns
    ///
    /// Pointer to the component value.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get_mut`
    #[doc(alias = "entity_view::get_mut")]
    fn get_untyped_mut(self, id: impl IntoId) -> *mut c_void {
        unsafe { sys::ecs_get_mut_id(self.world_ptr(), *self.entity_id(), *id.into()) }
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
    fn target<First: ComponentId>(&self, index: i32) -> Option<EntityView<'w>> {
        let id = unsafe {
            sys::ecs_get_target(
                self.world_ptr(),
                *self.entity_id(),
                First::id(self.world()),
                index,
            )
        };
        if id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world(), id))
        }
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
    fn target_id(&self, first: impl Into<Entity>, index: i32) -> Option<EntityView<'w>> {
        let id = unsafe {
            sys::ecs_get_target(self.world_ptr(), *self.entity_id(), *first.into(), index)
        };
        if id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world(), id))
        }
    }

    /// Get the target of a pair for a given relationship id.
    ///
    /// This operation returns the first entity that has the provided id by following
    /// the specified relationship. If the entity itself has the id then entity will
    /// be returned. If the id cannot be found on the entity or by following the
    /// relationship, the operation will return 0.
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
    /// * [`EntityView::target_for()`]
    /// * C++ API: `entity_view::target_for`
    #[doc(alias = "entity_view::target_for")]
    fn target_for_id(
        &self,
        relationship: impl Into<Entity>,
        component_id: impl IntoId,
    ) -> Option<EntityView<'w>> {
        let id = unsafe {
            sys::ecs_get_target_for_id(
                self.world_ptr(),
                *self.entity_id(),
                *relationship.into(),
                *component_id.into(),
            )
        };
        if id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world(), id))
        }
    }

    /// Get the target for a given component and relationship.
    ///
    /// This function is a convenient wrapper around `get_target_by_relationship_and_component_id`,
    /// allowing callers to provide a type and automatically deriving the component id.
    ///
    /// This operation can be used to lookup, for example, which prefab is providing
    /// a component by specifying the `IsA` pair:
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// # let world = World::new();
    /// # let entity = world.entity();
    /// # #[derive(Component)]
    /// # struct Position(f32, f32, f32);
    /// // Is Position provided by the entity or one of its base entities?
    /// let e = entity.target_for::<Position>(flecs::IsA::ID);
    /// ```
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
    /// * [`EntityView::target_for_id()`]
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target_for")]
    #[inline(always)]
    fn target_for<T: ComponentOrPairId>(
        self,
        relationship: impl Into<Entity>,
    ) -> Option<EntityView<'w>> {
        self.target_for_id(relationship, T::get_id(self.world()))
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
    fn depth_id(self, relationship: impl Into<Entity>) -> i32 {
        unsafe { sys::ecs_get_depth(self.world_ptr(), *self.entity_id(), *relationship.into()) }
    }

    /// Retrieves the depth for a specified relationship.
    ///
    /// This function is a convenient wrapper around `get_depth_id`, allowing callers
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
    fn depth<T: ComponentId>(self) -> i32 {
        let world = self.world();
        self.depth_id(T::id(world))
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
    fn parent(self) -> Option<EntityView<'w>> {
        self.target_id(ECS_CHILD_OF, 0)
    }

    /// Lookup an entity by name.
    /// The entity is searched recursively recursively traversing
    /// up the tree until found.
    ///
    /// The provided path may contain double colons as scope separators,
    /// for example: "`Foo::Bar`".
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise `None`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    fn try_lookup_recursive(&self, name: &str) -> Option<EntityView<'w>> {
        try_lookup_impl(self.world(), self.entity_id(), name, true)
    }

    /// Lookup an entity by name, only in the current scope of the entity.
    ///
    /// Lookup an entity in the scope of this entity. The provided path may
    /// contain double colons as scope separators, for example: "`Foo::Bar`".
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise `None`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    fn try_lookup(&self, name: &str) -> Option<EntityView<'w>> {
        try_lookup_impl(self.world(), self.entity_id(), name, false)
    }

    /// Lookup an entity by name.
    /// The entity is searched recursively recursively traversing
    /// up the tree until found.
    ///
    /// The provided path may contain double colons as scope separators,
    /// for example: "`Foo::Bar`".
    ///
    /// # Safety
    ///
    /// This function can return an entity with id 0 if the entity is not found.
    /// Ensure that the entity exists before using it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity, entity id will be 0 if not found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    fn lookup_recursive(&self, name: &str) -> EntityView<'w> {
        self.try_lookup_recursive(name)
            .expect("Entity not found, when unsure, use try_lookup_recursive")
    }

    /// Lookup an entity by name, only in the current scope of the entity.
    ///
    /// Lookup an entity in the scope of this entity. The provided path may
    /// contain double colons as scope separators, for example: "`Foo::Bar`".
    ///
    /// # Safety
    ///
    /// This function can return an entity with id 0 if the entity is not found.
    /// Ensure that the entity exists before using it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity, entity id will be 0 if not found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    fn lookup(&self, name: &str) -> EntityView<'w> {
        self.try_lookup(name)
            .expect("Entity not found, when unsure, use try_lookup")
    }

    /// Test if an entity has an id.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check.
    ///
    /// # Returns
    ///
    /// True if the entity has or inherits the provided id, false otherwise.
    ///
    /// # See also
    ///
    /// * [`EntityView::has()`]
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    #[inline(always)]
    fn has_id(&self, id: impl IntoId) -> bool {
        unsafe { sys::ecs_has_id(self.world_ptr(), *self.entity_id(), *id.into()) }
    }

    /// Check if entity has the provided component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to check.
    ///
    /// # Returns
    ///
    /// True if the entity has or inherits the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * [`EntityView::has_id()`]
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    fn has<T: ComponentOrPairId>(self) -> bool {
        if !T::IS_ENUM {
            unsafe { sys::ecs_has_id(self.world_ptr(), *self.entity_id(), T::get_id(self.world())) }
        } else {
            let component_id = T::get_id(self.world());
            self.has_id((component_id, ECS_WILDCARD))
        }
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
    fn has_enum<T>(self, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let component_id: sys::ecs_id_t = T::id(self.world());
        // Safety: we know the enum fields are registered because of the previous T::id call
        let enum_constant_entity_id = unsafe { constant.id_variant_unchecked(self.world()) };

        ecs_assert!(
            *enum_constant_entity_id.id != 0,
            FlecsErrorCode::InvalidParameter,
            "Constant was not found in Enum reflection data. Did you mean to use has<E>() instead of has(E)?"
        );

        self.has_id((component_id, enum_constant_entity_id))
    }

    /// Check if entity has the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
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
    fn has_first<First: ComponentId>(self, second: impl Into<Entity>) -> bool {
        let world = self.world();
        self.has_id((First::id(world), second.into()))
    }

    /// Check if entity has the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    fn has_second<Second: ComponentId>(self, first: impl Into<Entity>) -> bool {
        let world = self.world();
        self.has_id((first.into(), Second::id(world)))
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
    fn has_pair_enum<T: ComponentId, U: ComponentId + EnumComponentInfo>(
        &self,
        constant: U,
    ) -> bool {
        let world = self.world();
        let component_id: sys::ecs_id_t = T::id(world);
        let enum_constant_entity_id = constant.id_variant(world);

        self.has_id((component_id, enum_constant_entity_id))
    }

    /// Check if the entity owns the provided entity (pair, component, entity).
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
    fn owns_id(self, entity_id: impl IntoId) -> bool {
        unsafe { sys::ecs_owns_id(self.world_ptr(), *self.entity_id(), *entity_id.into()) }
    }

    /// Check if the entity owns the provided component.
    /// A component is owned if it is not shared from a base entity.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component to check.
    ///
    /// # Returns
    ///
    /// - `true` if the entity owns the provided component, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    fn owns<T: ComponentOrPairId>(self) -> bool {
        unsafe { sys::ecs_owns_id(self.world_ptr(), *self.entity_id(), T::get_id(self.world())) }
    }

    /// Check if the entity owns the provided pair.
    /// A pair is owned if it is not shared from a base entity.
    ///
    /// # Type Parameters
    /// - `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `second`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// - `true` if the entity owns the provided pair, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    fn owns_first<First: ComponentId>(self, second: impl Into<Entity>) -> bool {
        unsafe {
            sys::ecs_owns_id(
                self.world_ptr(),
                *self.entity_id(),
                ecs_pair(First::id(self.world()), *second.into()),
            )
        }
    }

    /// Check if the entity owns the provided pair.
    /// A pair is owned if it is not shared from a base entity.
    ///
    /// # Type Parameters
    /// - `Second`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `first`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// - `true` if the entity owns the provided pair, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    fn owns_second<Second: ComponentId>(self, first: impl Into<Entity>) -> bool {
        unsafe {
            sys::ecs_owns_id(
                self.world_ptr(),
                *self.entity_id(),
                ecs_pair(*first.into(), Second::id(self.world())),
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
    fn is_enabled_id(&self, id: impl IntoId) -> bool {
        unsafe { sys::ecs_is_enabled_id(self.world_ptr(), *self.entity_id(), *id.into()) }
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
    fn is_enabled<T: ComponentOrPairId>(&self) -> bool {
        unsafe {
            sys::ecs_is_enabled_id(self.world_ptr(), *self.entity_id(), T::get_id(self.world()))
        }
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
    fn is_enabled_first<T: ComponentId>(&self, second: impl Into<Entity>) -> bool {
        let world = self.world();
        self.is_enabled_id((T::id(world), second.into()))
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
    fn is_enabled_second<U: ComponentId>(&self, first: impl Into<Entity>) -> bool {
        let world = self.world();
        self.is_enabled_id((first.into(), U::id(world)))
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
    /// # See also
    ///
    /// * C++ API: `entity_view::clone`
    #[doc(alias = "entity_view::clone")]
    #[inline(always)]
    fn duplicate(self, copy_value: bool) -> EntityView<'w> {
        let dest_entity = EntityView::new(self.world());
        unsafe {
            sys::ecs_clone(
                self.world_ptr_mut(),
                *dest_entity.id,
                *self.entity_id(),
                copy_value,
            )
        };
        dest_entity
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
    /// # Safety
    /// This function makes use of `unsafe` operations to interact with the underlying ECS.
    /// Ensure that the provided `dest_id` is valid or zero
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::clone`
    #[doc(alias = "entity_view::clone")]
    #[inline(always)]
    fn duplicate_into(self, copy_value: bool, dest_id: impl Into<Entity>) -> EntityView<'w> {
        let mut dest_id = *dest_id.into();
        if dest_id == 0 {
            dest_id = unsafe { sys::ecs_new(self.world_ptr_mut()) };
        }

        let dest_entity = EntityView::new_from(self.world(), dest_id);
        unsafe { sys::ecs_clone(self.world_ptr_mut(), dest_id, *self.entity_id(), copy_value) };
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
    fn mut_current_stage(self, stage: impl WorldProvider<'w>) -> EntityView<'w> {
        ecs_assert!(
            !stage.world().is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use readonly world/stage to create mutable handle"
        );

        EntityView::new_from(stage, *self.entity_id())
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
    fn mut_stage_of<T>(self, entity: T) -> EntityView<'w>
    where
        T: Into<Entity> + WorldProvider<'w>,
    {
        ecs_assert!(
            !entity.world().is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use entity created for readonly world/stage to create mutable handle"
        );

        EntityView::new_from(entity.world(), *self.entity_id())
    }
}
