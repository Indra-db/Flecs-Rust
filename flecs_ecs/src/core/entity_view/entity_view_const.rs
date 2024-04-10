use std::{
    ffi::{c_void, CStr},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use crate::sys;
use flecs_ecs::core::*;

#[derive(Clone, Copy)]
pub struct EntityView<'a> {
    pub id: Id<'a>,
}

impl<'a, T> PartialEq<T> for EntityView<'a>
where
    T: IntoId,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.raw_id == other.get_id()
    }
}

impl<'a> Eq for EntityView<'a> {}

impl<'a, T> PartialOrd<T> for EntityView<'a>
where
    T: IntoId,
{
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        Some(self.raw_id.cmp(&other.get_id()))
    }
}

impl<'a> Ord for EntityView<'a> {
    #[inline]
    fn cmp(&self, other: &EntityView) -> std::cmp::Ordering {
        self.raw_id.cmp(&other.raw_id)
    }
}

impl<'a> Deref for EntityView<'a> {
    type Target = Id<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl<'a> DerefMut for EntityView<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}

impl<'a> std::fmt::Display for EntityView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.get_name() {
            write!(f, "{}", name)
        } else {
            write!(f, "{}", self.raw_id)
        }
    }
}

impl<'a> std::fmt::Debug for EntityView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();
        let id = self.raw_id;
        let archetype_str = self
            .archetype()
            .to_string()
            .unwrap_or_else(|| "empty".to_string());
        write!(
            f,
            "Entity name: {} -- id: {} -- archetype: {}",
            name, id, archetype_str
        )
    }
}

impl<'a> EntityView<'a> {
    /// Create new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(world: impl IntoWorld<'a>) -> Self {
        let id = unsafe { sys::ecs_new_id(world.world_ptr_mut()) };
        Self {
            id: Id::new(world, id),
        }
    }

    /// Creates a wrapper around an existing entity / id.
    ///
    /// # Arguments
    ///
    /// * `world` - The world the entity belongs to. If strictly only a storage is needed, this can be None.
    /// * `id` - The entity id.
    ///
    /// # Safety
    ///
    /// The world must be not be None if you want to do operations on the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    pub fn new_from(world: impl IntoWorld<'a>, id: impl IntoId) -> Self {
        Self {
            id: Id::new(world, id),
        }
    }

    /// Create a named entity.
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Arguments
    ///
    /// - `world`: The world in which to create the entity.
    /// - `name`: The entity name.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    pub fn new_named(world: impl IntoWorld<'a>, name: &CStr) -> Self {
        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            symbol: std::ptr::null(),
            use_low_id: false,
            add: [0; 32],
            add_expr: std::ptr::null(),
        };
        let id = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &desc) };
        Self {
            id: Id::new(world, id),
        }
    }

    /// Entity id 0.
    /// This function is useful when the API must provide an entity that
    /// belongs to a world, but the entity id is 0.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::null`
    #[doc(alias = "entity::null")]
    pub fn new_null(world: &'a World) -> EntityView<'a> {
        Self::new_from(world, 0)
    }

    /// checks if entity is valid
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::is_valid`
    #[doc(alias = "entity_view::is_valid")]
    pub fn is_valid(self) -> bool {
        unsafe { sys::ecs_is_valid(self.world.world_ptr_mut(), self.raw_id) }
    }

    /// Checks if entity is alive.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::is_alive`
    #[doc(alias = "entity_view::is_alive")]
    pub fn is_alive(self) -> bool {
        unsafe { sys::ecs_is_alive(self.world.world_ptr_mut(), self.raw_id) }
    }

    /// Returns the entity name.
    ///
    /// if the entity has no name, this will return an empty string
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    #[doc(alias = "entity_view::name")]
    pub fn name(self) -> &'a str {
        self.get_name().unwrap_or("")
    }

    /// Returns the entity name.
    ///
    /// if the entity has no name, this will return none
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    #[doc(alias = "entity_view::name")]
    pub fn get_name(self) -> Option<&'a str> {
        self.get_name_cstr().and_then(|s| s.to_str().ok())
    }

    /// Returns the entity name as a `CStr`.
    ///
    /// if the entity has no name, this will return an empty string
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    pub fn name_cstr(self) -> &'a CStr {
        self.get_name_cstr().unwrap_or(c"")
    }

    /// Returns the entity name as a `CStr`.
    ///
    /// if the entity has no name, this will return None
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    pub fn get_name_cstr(self) -> Option<&'a CStr> {
        NonNull::new(
            unsafe { sys::ecs_get_name(self.world.world_ptr_mut(), self.raw_id) } as *mut _,
        )
        .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) })
    }

    /// Returns the entity symbol.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::symbol`
    #[doc(alias = "entity_view::symbol")]
    pub fn symbol(self) -> &'a CStr {
        unsafe { CStr::from_ptr(sys::ecs_get_symbol(self.world.world_ptr_mut(), self.raw_id)) }
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
    pub fn path_w_sep(self, sep: &CStr, init_sep: &CStr) -> Option<String> {
        self.path_from_id_w_sep(0, sep, init_sep)
    }

    /// Return the hierarchical entity path using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path`
    #[doc(alias = "entity_view::path")]
    pub fn path(self) -> Option<String> {
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
    pub fn path_from_id_w_sep(
        &self,
        parent: impl IntoEntity,
        sep: &CStr,
        init_sep: &CStr,
    ) -> Option<String> {
        NonNull::new(unsafe {
            sys::ecs_get_path_w_sep(
                self.world.world_ptr_mut(),
                parent.get_id(),
                self.raw_id,
                sep.as_ptr(),
                init_sep.as_ptr(),
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
    pub fn path_from_id(self, parent: impl IntoEntity) -> Option<String> {
        NonNull::new(unsafe {
            sys::ecs_get_path_w_sep(
                self.world.world_ptr_mut(),
                parent.get_id(),
                self.raw_id,
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
    pub fn path_from<T: ComponentId>(self) -> Option<String> {
        self.path_from_id_w_sep(T::get_id(self.world), SEPARATOR, SEPARATOR)
    }

    pub fn path_from_w_sep<T: ComponentId>(&self, sep: &CStr, init_sep: &CStr) -> Option<String> {
        self.path_from_id_w_sep(T::get_id(self.world), sep, init_sep)
    }

    /// Return the hierarchical entity path relative to a parent type using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path_from`
    #[doc(alias = "entity_view::path_from")]
    pub fn hierarchy_path_from_parent_type<T: ComponentId>(self) -> Option<String> {
        self.path_from_id(T::get_id(self.world))
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
    pub fn is_enabled_self(self) -> bool {
        unsafe { !sys::ecs_has_id(self.world.world_ptr_mut(), self.raw_id, flecs::Disabled::ID) }
    }

    /// get the entity's archetype
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::type`
    #[doc(alias = "entity_view::type")]
    #[inline(always)]
    pub fn archetype(self) -> Archetype<'a> {
        self.table()
            .map(|t| t.archetype())
            .unwrap_or(Archetype::new(self.world, &[]))
    }

    /// get the entity's type/table
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::table`
    #[doc(alias = "entity_view::table")]
    #[inline(always)]
    pub fn table(self) -> Option<Table<'a>> {
        NonNull::new(unsafe { sys::ecs_get_table(self.world.world_ptr_mut(), self.raw_id) })
            .map(|t| Table::new(self.world, t))
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
    pub fn table_range(self) -> Option<TableRange<'a>> {
        NonNull::new(unsafe { sys::ecs_record_find(self.world.world_ptr_mut(), self.raw_id) }).map(
            |record| unsafe {
                TableRange::new_raw(
                    self.world,
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
    /// * `func` - The closure invoked for each matching ID. Must match the signature `FnMut(Id)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::each`
    #[doc(alias = "entity_view::each")]
    pub fn for_each_component<F>(self, mut func: F)
    where
        F: FnMut(Id),
    {
        let archetype = self.archetype();

        for &id in archetype.as_slice() {
            // Union object is not stored in type, so handle separately
            if ecs_pair_first(id) == flecs::Union::ID {
                let ent = Id::new(
                    self.world,
                    (ecs_pair_second(id), unsafe {
                        sys::ecs_get_target(
                            self.world.world_ptr_mut(),
                            self.raw_id,
                            ecs_pair_second(self.raw_id),
                            0,
                        )
                    }),
                );

                func(ent);
            } else {
                let ent = Id {
                    world: self.world,
                    raw_id: id,
                };
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
    pub fn for_each_matching_pair<F>(
        &self,
        pred: impl IntoEntity,
        obj: impl IntoEntity,
        mut func: F,
    ) where
        F: FnMut(Id),
    {
        // this is safe because we are only reading the world
        let real_world = self.world.real_world();

        let Some(table) =
            NonNull::new(unsafe { sys::ecs_get_table(real_world.world_ptr_mut(), self.raw_id) })
        else {
            return;
        };

        let table = Table::new(real_world, table);

        let mut pattern: IdT = pred.get_id();
        if obj.get_id() != 0 {
            pattern = ecs_pair(pred.get_id(), obj.get_id());
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
            let ent = Id::new(self.world, ids[cur as usize]);
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
    pub fn for_each_target_id<F>(self, relationship: impl IntoEntity, mut func: F)
    where
        F: FnMut(EntityView),
    {
        self.for_each_matching_pair(relationship.get_id(), ECS_WILDCARD, |id| {
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
    pub fn for_each_target<T>(self, func: impl FnMut(EntityView))
    where
        T: ComponentId,
    {
        self.for_each_target_id(
            EntityView::new_from(self.world, T::get_id(self.world)),
            func,
        );
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
    pub fn for_each_children_id<F>(self, relationship: impl IntoEntity, mut func: F)
    where
        F: FnMut(EntityView),
    {
        // When the entity is a wildcard, this would attempt to query for all
        //entities with (ChildOf, *) or (ChildOf, _) instead of querying for
        //the children of the wildcard entity.
        if self.raw_id == flecs::Wildcard::ID || self.raw_id == flecs::Any::ID {
            // this is correct, wildcard entities don't have children
            return;
        }

        let mut terms: [sys::ecs_term_t; 2] = unsafe { MaybeUninit::zeroed().assume_init() };

        let mut filter: sys::ecs_filter_t = unsafe { sys::ECS_FILTER_INIT };
        filter.terms = terms.as_mut_ptr();
        filter.term_count = 2;

        let mut desc: sys::ecs_filter_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
        desc.terms[0].first.id = relationship.get_id();
        desc.terms[0].second.id = self.raw_id;
        desc.terms[0].second.flags = sys::EcsIsEntity;
        desc.terms[1].id = flecs::Prefab::ID;
        desc.terms[1].oper = sys::ecs_oper_kind_t_EcsOptional;

        desc.storage = &mut filter;

        if !unsafe { sys::ecs_filter_init(self.world.world_ptr_mut(), &desc) }.is_null() {
            let mut it: sys::ecs_iter_t =
                unsafe { sys::ecs_filter_iter(self.world.world_ptr_mut(), &filter) };
            while unsafe { sys::ecs_filter_next(&mut it) } {
                for i in 0..it.count as usize {
                    unsafe {
                        //TODO should investigate if this is correct
                        let id = it.entities.add(i);
                        let ent = EntityView::new_from(self.world, *id);
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
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    pub fn for_each_children<T, F>(self, func: F)
    where
        T: ComponentId,
        F: FnMut(EntityView),
    {
        self.for_each_children_id(T::get_id(self.world), func);
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
    pub fn for_each_child_of<F>(self, func: F)
    where
        F: FnMut(EntityView),
    {
        self.for_each_children_id(flecs::ChildOf::ID, func);
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
    #[inline(always)]
    pub fn get<T: ComponentId>(self) -> Option<&'a T::UnderlyingType> {
        if !T::IS_ENUM {
            if T::IS_TAG {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidParameter,
                    "component {} has no size",
                    std::any::type_name::<T>()
                );
                None
            } else {
                let component_id = T::get_id(self.world);

                unsafe {
                    (sys::ecs_get_id(self.world.world_ptr_mut(), self.raw_id, component_id)
                        as *const T::UnderlyingType)
                        .as_ref()
                }
            }
        } else {
            let component_id: IdT = T::get_id(self.world);
            let target: IdT = unsafe {
                sys::ecs_get_target(self.world.world_ptr_mut(), self.raw_id, component_id, 0)
            };

            if target == 0 {
                // if there is no matching pair for (r,*), try just r
                unsafe {
                    (sys::ecs_get_id(self.world.world_ptr_mut(), self.raw_id, component_id)
                        as *const T::UnderlyingType)
                        .as_ref()
                }
            } else {
                // get constant value from constant entity
                let constant_value = unsafe {
                    sys::ecs_get_id(self.world.world_ptr_mut(), target, component_id)
                        as *const T::UnderlyingType
                };

                ecs_assert!(
                    !constant_value.is_null(),
                    FlecsErrorCode::InternalError,
                    "missing enum constant value {}",
                    std::any::type_name::<T>()
                );

                unsafe { constant_value.as_ref() }
            }
        }
    }

    pub fn get_callback<T: ComponentId>(self, callback: impl FnOnce(&T::UnderlyingType)) -> bool {
        if let Some(component) = self.get::<T>() {
            callback(component);
            return true;
        }
        false
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
    pub unsafe fn get_unchecked<T: ComponentId + ComponentType<Struct>>(
        &self,
    ) -> &T::UnderlyingType {
        if !T::IS_ENUM {
            if T::IS_TAG {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidParameter,
                    "component {} has no size",
                    std::any::type_name::<T>()
                );
                panic!("cannot get a tag component, it has no size");
            } else {
                let component_id = T::get_id_unchecked();

                let ptr = sys::ecs_get_id(self.world.world_ptr_mut(), self.raw_id, component_id)
                    as *const T::UnderlyingType;
                ecs_assert!(
                    !ptr.is_null(),
                    FlecsErrorCode::InternalError,
                    "missing component {}",
                    std::any::type_name::<T>()
                );
                &*ptr
            }
        } else {
            let component_id: IdT = T::get_id(self.world);
            let target: IdT = unsafe {
                sys::ecs_get_target(self.world.world_ptr_mut(), self.raw_id, component_id, 0)
            };

            if target == 0 {
                // if there is no matching pair for (r,*), try just r
                unsafe {
                    &*(sys::ecs_get_id(self.world.world_ptr_mut(), self.raw_id, component_id)
                        as *const T::UnderlyingType)
                }
            } else {
                // get constant value from constant entity
                let constant_value = unsafe {
                    sys::ecs_get_id(self.world.world_ptr_mut(), target, component_id)
                        as *const T::UnderlyingType
                };

                ecs_assert!(
                    !constant_value.is_null(),
                    FlecsErrorCode::InternalError,
                    "missing enum constant value {}",
                    std::any::type_name::<T>()
                );

                unsafe { &*constant_value }
            }
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
    pub unsafe fn get_enum_unchecked<T: ComponentId + ComponentType<Enum>>(
        &self,
    ) -> &T::UnderlyingType {
        let component_id: IdT = T::get_id(self.world);
        let target: IdT =
            sys::ecs_get_target(self.world.world_ptr_mut(), self.raw_id, component_id, 0);

        if target == 0 {
            // if there is no matching pair for (r,*), try just r
            let ptr = sys::ecs_get_id(self.world.world_ptr_mut(), self.raw_id, component_id)
                as *const T::UnderlyingType;
            ecs_assert!(
                !ptr.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                std::any::type_name::<T>()
            );
            &*ptr
        } else {
            // get constant value from constant entity
            let constant_value = sys::ecs_get_id(self.world.world_ptr_mut(), target, component_id)
                as *const T::UnderlyingType;
            ecs_assert!(
                !constant_value.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                std::any::type_name::<T>()
            );
            &*constant_value
        }
    }

    /// Get an option immutable reference for the first element of a pair
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// An option containing the reference to the first element of the pair if it exists, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_pair_first_id<First>(self, second: impl IntoEntity) -> Option<&'static First>
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let component_id = First::get_id(self.world);

        ecs_assert!(
            std::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<First>()
        );

        unsafe {
            (sys::ecs_get_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                ecs_pair(component_id, second.get_id()),
            ) as *const First)
                .as_ref()
        }
    }

    /// Get an immutable reference for the first element of a pair
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    /// * `Second`: The second part of the pair.
    ///
    /// # Returns
    ///
    /// An option containing the reference to the first element of the pair if it exists, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_pair_first<First, Second>(self) -> Option<&'static First>
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId,
    {
        self.get_pair_first_id(Second::get_id(self.world))
    }

    /// Get an immutable reference for the second element of a pair.
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # Returns
    ///
    /// An option containing the reference to the second element of the pair if it exists, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_pair_second_id<Second>(self, first: impl IntoEntity) -> Option<&'static Second>
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let component_id = Second::get_id(self.world);
        ecs_assert!(
            std::mem::size_of::<Second>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<Second>()
        );

        unsafe {
            (sys::ecs_get_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                ecs_pair(first.get_id(), component_id),
            ) as *const Second)
                .as_ref()
        }
    }

    /// Get an immutable reference for the second element of a pair.
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_pair_second<First, Second>(self) -> Option<&'static Second>
    where
        First: ComponentId + ComponentType<Struct> + EmptyComponent,
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        self.get_pair_second_id(First::get_id(self.world))
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
    pub fn get_untyped(self, component_id: impl IntoId) -> *const c_void {
        unsafe {
            sys::ecs_get_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                component_id.get_id(),
            )
        }
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
    pub fn target<First: ComponentId>(self, index: i32) -> EntityView<'a> {
        EntityView::new_from(self.world, unsafe {
            sys::ecs_get_target(
                self.world.world_ptr_mut(),
                self.raw_id,
                First::get_id(self.world),
                index,
            )
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
    pub fn target_id(self, first: impl IntoEntity, index: i32) -> EntityView<'a> {
        EntityView::new_from(self.world, unsafe {
            sys::ecs_get_target(
                self.world.world_ptr_mut(),
                self.raw_id,
                first.get_id(),
                index,
            )
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
    pub fn target_for_id(
        &self,
        relationship: impl IntoEntity,
        component_id: impl IntoEntity,
    ) -> EntityView<'a> {
        EntityView::new_from(self.world, unsafe {
            sys::ecs_get_target_for_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                relationship.get_id(),
                component_id.get_id(),
            )
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
    #[doc(alias = "entity_view::target_for")]
    #[inline(always)]
    pub fn target_for<T: IntoComponentId>(self, relationship: impl IntoEntity) -> EntityView<'a> {
        self.target_for_id(relationship, T::get_id(self.world))
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
    #[doc(alias = "entity_view::target_for")]
    pub fn target_for_pair_first<First: ComponentId>(
        &self,
        second: impl IntoEntity,
    ) -> *const First {
        let comp_id = First::get_id(self.world);
        ecs_assert!(
            std::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "First element is size 0"
        );
        unsafe {
            sys::ecs_get_id(
                self.world.world_ptr_mut(),
                comp_id,
                ecs_pair(comp_id, second.get_id()),
            ) as *const First
        }
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
    pub fn depth_id(self, relationship: impl IntoEntity) -> i32 {
        unsafe {
            sys::ecs_get_depth(
                self.world.world_ptr_mut(),
                self.raw_id,
                relationship.get_id(),
            )
        }
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
    pub fn depth<T: ComponentId>(self) -> i32 {
        self.depth_id(T::get_id(self.world))
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
    pub fn parent(self) -> EntityView<'a> {
        self.target_id(ECS_CHILD_OF, 0)
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
    pub fn lookup_name_optional(self, path: &CStr, search_path: bool) -> Option<EntityView<'a>> {
        ecs_assert!(
            self.raw_id != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid lookup from null handle"
        );
        let id = unsafe {
            sys::ecs_lookup_path_w_sep(
                self.world.world_ptr_mut(),
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
            Some(EntityView::new_from(self.world, id))
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
    pub fn has_id(self, entity: impl IntoId) -> bool {
        unsafe { sys::ecs_has_id(self.world.world_ptr_mut(), self.raw_id, entity.get_id()) }
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
    pub fn has<T: IntoComponentId>(self) -> bool {
        if !T::IS_ENUM {
            unsafe {
                sys::ecs_has_id(
                    self.world.world_ptr_mut(),
                    self.raw_id,
                    T::get_id(self.world),
                )
            }
        } else {
            let component_id = T::get_id(self.world);
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
    pub fn has_enum<T>(self, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        let component_id: IdT = T::get_id(self.world);
        // Safety: we know the enum fields are registered because of the previous T::get_id call
        let enum_constant_entity_id = unsafe { constant.get_id_variant_unchecked(self.world) };
        self.has_id((component_id, enum_constant_entity_id))
    }

    // this is pub(crate) because it's used for development purposes only
    pub(crate) fn has_enum_id<T>(self, enum_id: impl IntoEntity, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        let enum_constant_entity_id = constant.get_id_variant(self.world);
        self.has_id((enum_id, enum_constant_entity_id))
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
    pub fn has_pair_first<First: ComponentId>(self, second: impl IntoEntity) -> bool {
        self.has_id((First::get_id(self.world), second.get_id()))
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
    pub fn has_pair_second<Second: ComponentId>(self, first: impl IntoEntity) -> bool {
        self.has_id((first.get_id(), Second::get_id(self.world)))
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
    pub fn has_pair_enum<T: ComponentId, U: ComponentId + CachedEnumData>(
        &self,
        constant: U,
    ) -> bool {
        let component_id: IdT = T::get_id(self.world);
        let enum_constant_entity_id = constant.get_id_variant(self.world);

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
    pub fn owns_id(self, entity_id: impl IntoId) -> bool {
        unsafe { sys::ecs_owns_id(self.world.world_ptr_mut(), self.raw_id, entity_id.get_id()) }
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
    pub fn owns<T: IntoComponentId>(self) -> bool {
        unsafe {
            sys::ecs_owns_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                T::get_id(self.world),
            )
        }
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
    pub fn owns_pair_first<First: ComponentId>(self, second: impl IntoEntity) -> bool {
        unsafe {
            sys::ecs_owns_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                ecs_pair(First::get_id(self.world), second.get_id()),
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
    pub fn owns_pair_second<Second: ComponentId>(self, first: impl IntoEntity) -> bool {
        unsafe {
            sys::ecs_owns_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                ecs_pair(first.get_id(), Second::get_id(self.world)),
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
    pub fn is_enabled_id(self, id: impl IntoId) -> bool {
        unsafe { sys::ecs_is_enabled_id(self.world.world_ptr_mut(), self.raw_id, id.get_id()) }
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
    pub fn is_enabled<T: IntoComponentId>(self) -> bool {
        unsafe {
            sys::ecs_is_enabled_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                T::get_id(self.world),
            )
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
    pub fn is_enabled_pair_first<T: ComponentId>(self, second: impl IntoEntity) -> bool {
        self.is_enabled_id((T::get_id(self.world), second))
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
    pub fn is_enabled_pair_second<U: ComponentId>(self, first: impl IntoEntity) -> bool {
        self.is_enabled_id((first, U::get_id(self.world)))
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
    pub fn duplicate(self, copy_value: bool) -> EntityView<'a> {
        let dest_entity = EntityView::new(self.world());
        unsafe {
            sys::ecs_clone(
                self.world.world_ptr_mut(),
                dest_entity.raw_id,
                self.raw_id,
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
    /// ## Safety
    /// This function makes use of `unsafe` operations to interact with the underlying ECS.
    /// Ensure that the provided `dest_id` is valid or zero
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::clone`
    #[doc(alias = "entity_view::clone")]
    #[inline(always)]
    pub fn duplicate_into(self, copy_value: bool, dest_id: impl IntoEntity) -> EntityView<'a> {
        let mut dest_id = dest_id.get_id();
        if dest_id == 0 {
            dest_id = unsafe { sys::ecs_new_id(self.world.world_ptr_mut()) };
        }

        let dest_entity = EntityView::new_from(self.world, dest_id);
        unsafe { sys::ecs_clone(self.world.world_ptr_mut(), dest_id, self.raw_id, copy_value) };
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
    pub fn mut_current_stage(self, stage: &World) -> EntityView {
        ecs_assert!(
            !stage.is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use readonly world/stage to create mutable handle"
        );

        EntityView::new_from(stage, self.raw_id)
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
    pub fn mut_stage_of<T>(self, entity: T) -> EntityView<'a>
    where
        T: IntoEntity + IntoWorld<'a>,
    {
        ecs_assert!(
            !entity.world().is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use entity created for readonly world/stage to create mutable handle"
        );

        EntityView::new_from(entity.world(), self.raw_id)
    }

    //might not be needed, in the original c++ impl it was used in the get_mut functions.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::set_stage`
    #[doc(alias = "entity_view::set_stage")]
    #[doc(hidden)]
    fn set_stage(self, stage: impl IntoWorld<'a>) -> EntityView<'a> {
        EntityView::new_from(stage, self.raw_id)
    }

    /// Turn entity into an enum constant.
    ///
    /// # Safety
    ///
    /// This function returns an Option because the entity might not be a constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Returns
    ///
    /// * `Some(&T)` - The enum constant if the entity is a constant.
    /// * `None` - If the entity is not a constant.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::to_constant`
    #[doc(alias = "entity_view::to_constant")]
    pub fn to_constant<T: ComponentId>(self) -> Option<&'a T::UnderlyingType> {
        let ptr = self.get::<T>();
        ecs_assert!(
            ptr.is_some(),
            FlecsErrorCode::InvalidParameter,
            "entity is not a constant"
        );
        ptr
    }

    /// Turn entity into an enum constant.
    ///
    /// # Safety
    ///
    /// ensure that the entity is a constant before calling this function.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::to_constant`
    #[doc(alias = "entity_view::to_constant")]
    pub unsafe fn to_constant_unchecked<T: ComponentId + ComponentType<Enum>>(
        &self,
    ) -> &T::UnderlyingType {
        self.get_enum_unchecked::<T>()
    }
}

// Event mixin
impl<'a> EntityView<'a> {
    /// Emit event for entity
    ///
    /// # Safety
    /// Caller must ensure that any type associated with `event` is a ZST
    ///
    /// # Arguments
    ///
    /// * event - the event to emit
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::emit`
    #[doc(alias = "entity_view::emit")]
    pub unsafe fn emit_id(self, event: impl IntoEntity) {
        self.world()
            .event_id(event)
            .set_entity_to_emit(self.to_entity())
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
    pub fn emit<T: EmptyComponent + ComponentId>(self) {
        unsafe { self.emit_id(T::get_id(self)) }
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
    pub fn emit_payload<T: NotEmptyComponent + ComponentId>(self, payload: &mut T) {
        self.world()
            .event::<T>()
            .set_entity_to_emit(self.to_entity())
            .set_event_data(payload)
            .emit();
    }

    /// Enqueue event for entity.
    ///
    /// # Safety
    ///
    ///
    /// # Arguments
    ///
    /// * event - the event to enqueue
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    pub unsafe fn enqueue_id(self, event: impl IntoEntity) {
        self.world()
            .event_id(event)
            .set_entity_to_emit(self.to_entity())
            .enqueue();
    }

    /// Enqueue event for entity
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to enqueue. Type must be empty.
    ///
    /// # Usage:
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```rust
    /// world.defer_begin();
    /// entity.enqueue::<MyEvent>();
    /// world.defer_end();
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    pub fn enqueue<T: EmptyComponent + ComponentId>(self) {
        unsafe { self.enqueue_id(T::get_id(self.world)) };
    }

    /// enqueue event with payload for entity.
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to enqueue. Type must contain data (not empty struct).
    ///
    /// # Usage:
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```rust
    /// struct Resize {
    ///    width: i32,
    ///   height: i32,
    /// }
    ///
    /// world.defer_begin();
    /// entity.enqueue_payload(Resize{width: 10, height: 20});
    /// world.defer_end();
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    pub fn enqueue_payload<T: NotEmptyComponent + ComponentId>(self, payload: &mut T) {
        self.world()
            .event::<T>()
            .set_entity_to_emit(self.to_entity())
            .set_event_data(payload)
            .enqueue();
    }
}

// Event/Observe mixin
impl<'a> EntityView<'a> {
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
    pub fn observe<C>(self, func: impl FnMut()) -> Self
    where
        C: ComponentId + EmptyComponent,
    {
        self.observe_impl::<C, _>(func)
    }

    fn observe_impl<C, Func>(self, func: Func) -> Self
    where
        Func: FnMut(),
        C: ComponentId,
    {
        let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.empty = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_empty = Some(Self::on_free_empty);

        Self::entity_observer_create(
            self.world.world_ptr_mut(),
            C::get_id(self.world),
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
    ///  - `func(&mut EntityView)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe_entity<C>(self, func: impl FnMut(&mut EntityView)) -> Self
    where
        C: ComponentId + EmptyComponent,
    {
        self.observe_entity_impl::<C, _>(func)
    }

    fn observe_entity_impl<C, Func>(self, func: Func) -> Self
    where
        Func: FnMut(&mut EntityView),
        C: ComponentId,
    {
        let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.empty_entity = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_empty_entity = Some(Self::on_free_empty_entity);

        Self::entity_observer_create(
            self.world.world_ptr_mut(),
            C::get_id(self.world),
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
    pub fn observe_payload<C>(self, func: impl FnMut(&mut C)) -> Self
    where
        C: ComponentId + NotEmptyComponent,
    {
        self.observe_payload_impl::<C, _>(func)
    }

    fn observe_payload_impl<C, Func>(self, func: Func) -> Self
    where
        Func: FnMut(&mut C),
        C: ComponentId,
    {
        let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.payload = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_payload = Some(Self::on_free_payload::<C>);

        Self::entity_observer_create(
            self.world.world_ptr_mut(),
            C::get_id(self.world),
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
    ///  - `func(&mut EntityView, &mut EventData)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe_payload_entity<C>(self, func: impl FnMut(&mut EntityView, &mut C)) -> Self
    where
        C: ComponentId + NotEmptyComponent,
    {
        self.observe_payload_entity_impl::<C, _>(func)
    }

    fn observe_payload_entity_impl<C, Func>(self, func: Func) -> Self
    where
        Func: FnMut(&mut EntityView, &mut C),
        C: ComponentId,
    {
        let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.payload_entity = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_payload_entity = Some(Self::on_free_payload_entity::<C>);

        Self::entity_observer_create(
            self.world.world_ptr_mut(),
            C::get_id(self.world),
            self.raw_id,
            binding_ctx,
            Some(Self::run_payload_entity::<C, Func> as unsafe extern "C" fn(_)),
        );
        self
    }
}

// entity observer creation
impl<'a> EntityView<'a> {
    pub(crate) fn entity_observer_create(
        world: *mut WorldT,
        event: EntityT,
        entity: EntityT,
        binding_ctx: *mut ObserverEntityBindingCtx,
        callback: sys::ecs_iter_action_t,
    ) {
        let mut desc = sys::ecs_observer_desc_t::default();
        desc.events[0] = event;
        desc.filter.terms[0].id = ECS_ANY;
        desc.filter.terms[0].src.id = entity;
        desc.callback = callback;
        desc.binding_ctx = binding_ctx as *mut c_void;
        desc.binding_ctx_free = Some(Self::binding_entity_ctx_drop);

        let observer = unsafe { sys::ecs_observer_init(world, &desc) };
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

        sys::ecs_table_lock((*iter).world, (*iter).table);

        for _i in 0..iter_count {
            empty();
        }

        sys::ecs_table_unlock((*iter).world, (*iter).table);
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
        Func: FnMut(&mut EntityView),
    {
        let ctx: *mut ObserverEntityBindingCtx = (*iter).binding_ctx as *mut _;
        let empty = (*ctx).empty_entity.unwrap();
        let empty = &mut *(empty as *mut Func);
        let iter_count = (*iter).count as usize;

        sys::ecs_table_lock((*iter).world, (*iter).table);

        for _i in 0..iter_count {
            let world = WorldRef::from_ptr((*iter).world);
            empty(&mut EntityView::new_from(
                world,
                sys::ecs_field_src(iter, 1),
            ));
        }

        sys::ecs_table_unlock((*iter).world, (*iter).table);
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

        sys::ecs_table_lock((*iter).world, (*iter).table);

        for _i in 0..iter_count {
            let data = (*iter).param as *mut C;
            let data_ref = &mut *data;
            empty(data_ref);
        }

        sys::ecs_table_unlock((*iter).world, (*iter).table);
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
        Func: FnMut(&mut EntityView, &mut C),
    {
        let ctx: *mut ObserverEntityBindingCtx = (*iter).binding_ctx as *mut _;
        let empty = (*ctx).payload_entity.unwrap();
        let empty = &mut *(empty as *mut Func);
        let iter_count = (*iter).count as usize;

        sys::ecs_table_lock((*iter).world, (*iter).table);

        for _i in 0..iter_count {
            let data = (*iter).param as *mut C;
            let data_ref = &mut *data;
            let world = WorldRef::from_ptr((*iter).world);
            empty(
                &mut EntityView::new_from(world, sys::ecs_field_src(iter, 1)),
                data_ref,
            );
        }

        sys::ecs_table_unlock((*iter).world, (*iter).table);
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
        let ptr_func: *mut fn(&mut EntityView) = ptr as *mut fn(&mut EntityView);
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
        let ptr_func: *mut fn(&mut EntityView, &mut C) = ptr as *mut fn(&mut EntityView, &mut C);
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
