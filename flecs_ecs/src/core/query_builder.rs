//! Builder for [`Query`].

use core::ffi::c_void;
use core::mem::ManuallyDrop;

use crate::core::internals::*;
use crate::core::*;
use crate::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::{format, vec::Vec};
use flecs_ecs_derive::extern_abi;

/// Builder for [`Query`].
///
/// # Example
///
/// This example shows how to return a query or query builder from a function. The lifetime `'static`
/// is required in the type `&'static Foo` to ensure the components accessed by the query
/// live long enough to be safely used.
///
/// ```
/// use flecs_ecs::prelude::*;
///
/// #[derive(Component)]
/// struct Foo(u8);
///
/// fn foo_query(world: &World) -> Query<&'static Foo> {
///     query!(world, &Foo).build()
/// }
///
/// fn plugin(world: &World) {
///     let foos = foo_query(world);
///
///     world.system::<()>().each(move |_| {
///         foos.each(|foo| {
///             // ..
///         });
///     });
/// }
/// ```
pub struct QueryBuilder<'a, T>
where
    T: QueryTuple,
{
    pub(crate) desc: sys::ecs_query_desc_t,
    pub(crate) term_builder: TermBuilder,
    world: WorldRef<'a>,
    _phantom: core::marker::PhantomData<T>,
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct QueryFlags: u32 {
        const MatchPrefab = sys::EcsQueryMatchPrefab;
        const MatchDisabled = sys::EcsQueryMatchDisabled;
        const MatchEmptyTables = sys::EcsQueryMatchEmptyTables;
        const AllowUnresolvedByName = sys::EcsQueryAllowUnresolvedByName;
        const TableOnly = sys::EcsQueryTableOnly;
    }
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct ObserverFlags: u32 {
        const BypassQuery = sys::EcsObserverBypassQuery;
        const YieldOnCreate = sys::EcsObserverYieldOnCreate;
        const YieldOnDelete = sys::EcsObserverYieldOnDelete;
    }
}

impl<'a, T> QueryBuilder<'a, T>
where
    T: QueryTuple,
{
    /// Create a new query builder
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    pub fn new(world: &'a World) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            world: world.world(),
            term_builder: Default::default(),
            _phantom: core::marker::PhantomData,
        };

        T::populate(&mut obj);

        obj
    }

    /// Create a new query builder with a name
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `name` - The name of the observer
    pub fn new_named(world: &'a World, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let desc = Default::default();

        let mut obj = Self {
            desc,
            term_builder: Default::default(),
            world: world.world(),
            _phantom: core::marker::PhantomData,
        };

        let entity_desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &entity_desc) };

        T::populate(&mut obj);
        obj
    }

    /// Create a new query builder from an existing descriptor
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `desc` - The descriptor to create the observer from
    pub(crate) fn new_from_desc(
        world: impl WorldProvider<'a>,
        desc: &mut sys::ecs_query_desc_t,
    ) -> Self {
        Self {
            desc: *desc,
            term_builder: Default::default(),
            world: world.world(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a new query builder from an existing descriptor with a term index
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `desc` - The descriptor to create the observer from
    /// * `term_index` - The index of the term to create the observer from
    pub(crate) fn new_from_desc_term_index(
        world: &'a World,
        desc: &mut sys::ecs_query_desc_t,
        term_index: i32,
    ) -> Self {
        let mut obj = Self {
            desc: *desc,
            term_builder: TermBuilder {
                current_term_index: term_index,
                next_term_index: term_index,
                expr_count: 0,
                term_ref_mode: TermRefMode::Src,
                str_ptrs_to_free: Vec::new(),
            },
            world: world.world(),
            _phantom: core::marker::PhantomData,
        };

        T::populate(&mut obj);
        obj
    }
}

#[doc(hidden)]
impl<'a, T: QueryTuple> internals::QueryConfig<'a> for QueryBuilder<'a, T> {
    #[inline(always)]
    fn term_builder(&self) -> &TermBuilder {
        &self.term_builder
    }

    #[inline(always)]
    fn term_builder_mut(&mut self) -> &mut TermBuilder {
        &mut self.term_builder
    }

    #[inline(always)]
    fn query_desc(&self) -> &sys::ecs_query_desc_t {
        &self.desc
    }

    #[inline(always)]
    fn query_desc_mut(&mut self) -> &mut sys::ecs_query_desc_t {
        &mut self.desc
    }

    #[inline(always)]
    fn count_generic_terms(&self) -> i32 {
        T::COUNT
    }
}

impl<'a, T: QueryTuple> TermBuilderImpl<'a> for QueryBuilder<'a, T> {}

impl<'a, T: QueryTuple> QueryBuilderImpl<'a> for QueryBuilder<'a, T> {}

impl<'a, T: QueryTuple> WorldProvider<'a> for QueryBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a, T> Builder<'a> for QueryBuilder<'a, T>
where
    T: QueryTuple,
{
    type BuiltType = Query<T>;

    /// Build the `query_builder` into an query
    ///
    /// # Returns
    ///
    /// The built query
    ///
    /// # Example
    ///
    /// * how to return a query / query builder from a function see example in [`QueryBuilder`]
    fn build(&mut self) -> Self::BuiltType {
        let world = self.world;
        let query = Query::<T>::new_from_desc(world, &mut self.desc);
        for s in self.term_builder.str_ptrs_to_free.iter_mut() {
            unsafe { ManuallyDrop::drop(s) };
        }
        self.term_builder.str_ptrs_to_free.clear();
        query
    }
}

// Assuming some imports and definitions from your previous example, and adding the required ones for this example.
#[cfg(not(target_family = "wasm"))]
type GroupByFn = extern "C-unwind" fn(
    *mut sys::ecs_world_t,
    *mut sys::ecs_table_t,
    sys::ecs_id_t,
    *mut c_void,
) -> u64;

#[cfg(target_family = "wasm")]
type GroupByFn =
    extern "C" fn(*mut sys::ecs_world_t, *mut sys::ecs_table_t, sys::ecs_id_t, *mut c_void) -> u64;

// Type definitions for OrderBy function pointers
#[cfg(not(target_family = "wasm"))]
type OrderByFnPtr<T> = extern "C-unwind" fn(Entity, &T, Entity, &T) -> i32;
#[cfg(target_family = "wasm")]
type OrderByFnPtr<T> = extern "C" fn(Entity, &T, Entity, &T) -> i32;

#[cfg(not(target_family = "wasm"))]
type OrderByFnPtrUnsafe = unsafe extern "C-unwind" fn(
    u64,
    *const core::ffi::c_void,
    u64,
    *const core::ffi::c_void,
) -> i32;
#[cfg(target_family = "wasm")]
type OrderByFnPtrUnsafe =
    unsafe extern "C" fn(u64, *const core::ffi::c_void, u64, *const core::ffi::c_void) -> i32;

#[cfg(not(target_family = "wasm"))]
type OrderByFnVoidPtr = extern "C-unwind" fn(Entity, *const c_void, Entity, *const c_void) -> i32;
#[cfg(target_family = "wasm")]
type OrderByFnVoidPtr = extern "C" fn(Entity, *const c_void, Entity, *const c_void) -> i32;

#[cfg(not(target_family = "wasm"))]
type OrderByFnVoidPtrUnsafe = unsafe extern "C-unwind" fn(
    u64,
    *const core::ffi::c_void,
    u64,
    *const core::ffi::c_void,
) -> i32;
#[cfg(target_family = "wasm")]
type OrderByFnVoidPtrUnsafe =
    unsafe extern "C" fn(u64, *const core::ffi::c_void, u64, *const core::ffi::c_void) -> i32;

/// Functions to build a query using terms.
pub trait QueryBuilderImpl<'a>: TermBuilderImpl<'a> {
    /// set the name of the query-like object
    fn named(&mut self, name: &str) -> &mut Self {
        let name = compact_str::format_compact!("{}\0", name);
        let world_ptr = self.world_ptr_mut();

        let entity_desc: sys::ecs_entity_desc_t = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..core::default::Default::default()
        };
        let entity_field_ref = &mut self.query_desc_mut().entity;
        if *entity_field_ref != 0 {
            unsafe { sys::ecs_delete(world_ptr, *entity_field_ref) };
        }

        *entity_field_ref = unsafe { sys::ecs_entity_init(world_ptr, &entity_desc) };
        self
    }

    /// set querylags
    ///
    /// # Arguments
    ///
    /// * `flags` - the flags to set
    fn query_flags(&mut self, flags: QueryFlags) -> &mut Self {
        self.query_desc_mut().flags |= flags.bits();
        self
    }

    /// Set what cache method to use for the query
    ///
    /// # Arguments
    ///
    /// * `kind` - the cache kind to set
    fn set_cache_kind(&mut self, kind: QueryCacheKind) -> &mut Self {
        self.query_desc_mut().cache_kind = kind as sys::ecs_query_cache_kind_t;
        self
    }

    /// Set the cache method to cached
    fn set_cached(&mut self) -> &mut Self {
        self.set_cache_kind(QueryCacheKind::Auto)
    }

    fn detect_changes(&mut self) -> &mut Self {
        self.query_desc_mut().flags |= sys::EcsQueryDetectChanges;
        self
    }

    /// set expression
    ///
    /// # Arguments
    ///
    /// * `expr` - the expression to set
    fn expr(&mut self, expr: &'a str) -> &mut Self {
        let expr = ManuallyDrop::new(format!("{expr}\0"));
        ecs_assert!(
            *self.expr_count_mut() == 0,
            FlecsErrorCode::InvalidOperation,
            "query_builder::expr() called more than once"
        );

        self.query_desc_mut().expr = expr.as_ptr() as *const _;
        *self.expr_count_mut() += 1;
        self.term_builder_mut().str_ptrs_to_free.push(expr);
        self
    }

    fn with<T>(&mut self, id: T) -> &mut Self
    where
        Access: FromAccessArg<T>,
    {
        let access = <Access as FromAccessArg<T>>::from_access_arg(id, self.world());
        self.term();

        match access.target {
            AccessTarget::Entity(entity) => {
                self.init_current_term(entity);
            }
            AccessTarget::Pair(rel, target) => {
                self.init_current_term(ecs_pair(*rel, *target));
            }
            AccessTarget::Name(name) => {
                self.set_first::<&'static str>(name);
            }
            AccessTarget::PairName(rel, target) => {
                self.set_first::<&'static str>(rel)
                    .set_second::<&'static str>(target);
            }
            AccessTarget::PairEntityName(rel, target) => {
                self.init_current_term(rel);
                self.set_second::<&'static str>(target);
            }
            AccessTarget::PairNameEntity(rel, target) => {
                self.set_first::<&'static str>(rel);
                self.set_second::<Entity>(target);
            }
        }

        match access.mode {
            AccessMode::Read => {
                self.current_term_mut().inout = InOutKind::In as i16;
            }
            AccessMode::ReadWrite => {
                self.current_term_mut().inout = InOutKind::InOut as i16;
            }
            AccessMode::Write => {
                self.current_term_mut().inout = InOutKind::Out as i16;
            }
            _ => {}
        }
        self
    }

    /// set term with enum
    fn with_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &mut self,
        value: T,
    ) -> &mut Self {
        let enum_id = T::entity_id(self.world());
        let enum_field_id = value.id_variant(self.world());
        self.with((enum_id, enum_field_id))
    }

    /// set term with enum wildcard
    fn with_enum_wildcard<T: ComponentType<Enum> + ComponentId>(&mut self) -> &mut Self
    where
        (crate::core::utility::id::Id<T>, u64): InternalIntoEntity,
    {
        self.with((T::id(), flecs::Wildcard::ID))
    }

    /* Without methods, shorthand for .with(...).not() */

    /// set term without Id
    fn without<T>(&mut self, id: T) -> &mut Self
    where
        Access: FromAccessArg<T>,
    {
        self.with(id).not()
    }

    /// set term without enum
    fn without_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.with_enum(value).not()
    }

    /// set term without enum wildcard
    fn without_enum_wildcard<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &mut self,
    ) -> &mut Self
    where
        (crate::core::utility::id::Id<T>, u64): InternalIntoEntity,
    {
        self.with_enum_wildcard::<T>().not()
    }

    /// Term notation for more complex query features
    ///
    /// sets the current term to next one in term list
    fn term(&mut self) -> &mut Self {
        let current_index = self.current_term_index();
        let next_index = self.next_term_index();

        if current_index != next_index {
            *self.current_term_index_mut() = next_index;
        }
        *self.next_term_index_mut() = next_index + 1;

        ecs_assert!(
            current_index < sys::FLECS_TERM_COUNT_MAX as i32,
            FlecsErrorCode::InvalidParameter,
            "Maximum number of terms reached in query builder",
        );

        self
    }

    /// Sets the current term to the one with the provided type.
    /// This loops over all terms to find the one with the provided type.
    /// For performance-critical paths, use `term_at(index: u32)` instead.
    fn term_at_type<T: ComponentId>(&mut self) -> &mut Self {
        let term_id = T::entity_id(self.world());
        let world_ptr = self.world_ptr_mut();

        for i in 0..=self.term_builder().next_term_index - 1 {
            let desc = self.query_desc();
            let cur_term = desc.terms[i as usize];
            let cur_term_id = cur_term.id;
            let cur_term_pair = ecs_pair(cur_term.first.id, cur_term.second.id);

            if (term_id == cur_term_id
                || (cur_term_id != 0
                    && term_id == unsafe { sys::ecs_get_typeid(world_ptr, cur_term_id) }))
                || (term_id == cur_term_pair
                    || (cur_term_pair != 0
                        && term_id == unsafe { sys::ecs_get_typeid(world_ptr, cur_term_pair) }))
            {
                return self.term_at(i as u32);
            }
        }

        panic!("term_at_type() called with type that is not in query",);
    }

    /// Sets the current term to the one at the provided index.
    fn term_at(&mut self, index: u32) -> &mut Self {
        ecs_assert!(
            index < sys::FLECS_TERM_COUNT_MAX,
            FlecsErrorCode::InvalidParameter,
            "term_at() called with invalid index"
        );

        self.set_term_ref_mode(TermRefMode::Src);

        *self.current_term_index_mut() = index as i32;

        self
    }

    //flecs_force_build_debug_c || flecs_force_enable_ecs_asserts

    /*
            /** Sets the current term to the one at the provided index and asserts that the type matches.
         */
        template <typename T>
        Base& term_at(int32_t term_index) {
            this->term_at(term_index);
    #if !defined(FLECS_NDEBUG) || defined(FLECS_KEEP_ASSERT)
            flecs::id_t term_id = _::type<T>::id(this->world_v());
            ecs_term_t cur_term = *this->term_;
            ecs_id_t cur_term_id = cur_term.id;
            ecs_id_t cur_term_pair = ecs_pair(cur_term.first.id, cur_term.second.id);

            ecs_assert((term_id == cur_term_id || (cur_term_id != 0 && term_id == ecs_get_typeid(this->world_v(), cur_term_id))) ||
                (term_id == cur_term_pair || (cur_term_pair != 0 && term_id == ecs_get_typeid(this->world_v(), cur_term_pair))),
                ECS_INVALID_PARAMETER, "term type mismatch");
    #endif
            return *this;
        }
         */

    /// Set the current term to the one with the provided id and assert that the type matches.
    /// this does not do the type checking in release unless `flecs_force_build_debug_c` or `flecs_force_enable_ecs_asserts` is enabled.
    fn term_at_checked<T: ComponentId>(&mut self, index: u32) -> &mut Self {
        ecs_assert!(
            index < sys::FLECS_TERM_COUNT_MAX,
            FlecsErrorCode::InvalidParameter,
            "term_at() called with invalid index"
        );

        self.set_term_ref_mode(TermRefMode::Src);

        *self.current_term_index_mut() = index as i32;

        #[cfg(any(
            feature = "flecs_force_build_debug_c",
            feature = "flecs_force_enable_ecs_asserts",
            debug_assertions
        ))]
        {
            let term_id = T::entity_id(self.world());
            let cur_term = self.current_term();
            let cur_term_id = cur_term.id;
            let cur_term_pair = ecs_pair(cur_term.first.id, cur_term.second.id);

            ecs_assert!(
                (term_id == cur_term_id
                    || (cur_term_id != 0
                        && term_id
                            == unsafe { sys::ecs_get_typeid(self.world_ptr_mut(), cur_term_id) }))
                    || (term_id == cur_term_pair
                        || (cur_term_pair != 0
                            && term_id
                                == unsafe {
                                    sys::ecs_get_typeid(self.world_ptr_mut(), cur_term_pair)
                                })),
                FlecsErrorCode::InvalidParameter,
                "term type mismatch"
            );
        }

        self
    }

    /// Set the id as current term and in mode out
    fn write<T>(&mut self, id: T) -> &mut Self
    where
        Access: FromAccessArg<T>,
    {
        self.with(id);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the type as current term and in mode out
    fn write_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.with_enum(value);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the id as current term and in mode in
    fn read<T>(&mut self, id: T) -> &mut Self
    where
        Access: FromAccessArg<T>,
    {
        self.with(id);
        TermBuilderImpl::read_curr(self)
    }

    /// Set the type as current term and in mode in
    fn read_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.with_enum(value);
        TermBuilderImpl::read_curr(self)
    }

    /* scope_open/scope_close shorthand notation. */

    /// Open a scope for the query
    fn scope_open(&mut self) -> &mut Self {
        self.with(flecs::ScopeOpen::ID).entity(0)
    }

    /// Close a scope for the query
    fn scope_close(&mut self) -> &mut Self {
        self.with(flecs::ScopeClose::ID).entity(0)
    }

    /// Sorts the output of a query.
    ///
    /// This enables sorting of entities across matched tables. As a result of this
    /// operation, the order of entities in the matched tables may change.
    /// Resorting occurs when a query iterator is obtained, and only if the table
    /// data has changed.
    ///
    /// If multiple queries that match the same (down)set of tables specify different
    /// sorting functions, resorting is likely to occur every time an iterator is
    /// obtained, potentially slowing down iterations significantly.
    ///
    /// The sorting function will be applied to the specified component. Resorting
    /// only occurs if that component has changed, or when the entity order in the
    /// table changes. If no component is provided, resorting only occurs when
    /// the entity order changes.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component used to sort.
    ///
    /// # Arguments
    ///
    /// * `compare`: The compare function used to sort the components.
    ///   The signature of the function must be `fn(Entity, &T, Entity, &T) -> i32`.
    fn order_by<T>(&mut self, compare: impl OrderByFn<T>) -> &mut Self
    where
        T: ComponentId,
        Self: QueryBuilderImpl<'a>,
    {
        let cmp: sys::ecs_order_by_action_t = Some(unsafe {
            core::mem::transmute::<OrderByFnPtr<T>, OrderByFnPtrUnsafe>(compare.to_extern_fn())
        });

        self.__internal_order_by_id(T::entity_id(self.world()), cmp);
        self
    }

    #[doc(hidden)]
    fn __internal_order_by_id(
        &mut self,
        component: impl Into<Entity>,
        compare: sys::ecs_order_by_action_t,
    ) {
        let desc = self.query_desc_mut();
        desc.order_by_callback = compare;
        desc.order_by = *component.into();
    }

    /// Sorts the output of a query.
    ///
    /// This is similar to `order_by<T>`, but uses a component identifier instead.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to sort.
    /// * `compare`: The compare function used to sort the components.
    fn order_by_id(
        &mut self,
        component: impl Into<Entity>,
        compare: impl OrderByFnVoid,
    ) -> &mut Self {
        let desc = self.query_desc_mut();
        let cmp: sys::ecs_order_by_action_t = Some(unsafe {
            core::mem::transmute::<OrderByFnVoidPtr, OrderByFnVoidPtrUnsafe>(compare.to_extern_fn())
        });
        desc.order_by_callback = cmp;
        desc.order_by = *component.into();
        self
    }

    /// Group and sort matched tables.
    ///
    /// This function is similar to `order_by`, but instead of sorting individual entities,
    /// it only sorts matched tables. This can be useful if a query needs to enforce a
    /// certain iteration order upon the tables it is iterating, for example by giving
    /// a certain component or tag a higher priority.
    ///
    /// The sorting function assigns a "rank" to each type, which is then used to sort
    /// the tables. Tables with higher ranks will appear later in the iteration.
    ///
    /// Resorting happens when a query iterator is obtained, and only if the set of
    /// matched tables for a query has changed. If table sorting is enabled together
    /// with entity sorting, table sorting takes precedence, and entities will be sorted
    /// within each set of tables that are assigned the same rank.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to determine the group rank.
    /// * `group_by_action`: Callback that determines group id for table.
    fn group_by_fn(
        &mut self,
        component: impl IntoEntity,
        group_by_action: sys::ecs_group_by_action_t,
    ) -> &mut Self {
        let world = self.world();
        let desc = self.query_desc_mut();
        desc.group_by_callback = group_by_action;
        desc.group_by = *component.into_entity(world);
        self
    }

    /// Group and sort matched tables.
    ///
    /// This is similar to `group_by_default<T>`, but uses a component identifier instead.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to determine the group rank.
    fn group_by(&mut self, component: impl IntoEntity) -> &mut Self {
        self.group_by_fn(component, None)
    }

    /// Specify context to be passed to the `group_by` function.
    ///
    /// # Arguments
    ///
    /// * `ctx`: Context to pass to the `group_by` function.
    /// * `ctx_free`: Function to clean up the context (called when the query is deleted).
    fn group_by_ctx(&mut self, ctx: *mut c_void, ctx_free: sys::ecs_ctx_free_t) -> &mut Self {
        let desc = self.query_desc_mut();
        desc.group_by_ctx = ctx;
        desc.group_by_ctx_free = ctx_free;
        self
    }

    /// Specify the `on_group_create` action.
    ///
    /// # Arguments
    ///
    /// * `action`: The action to execute when a group is created.
    fn on_group_create(&mut self, action: sys::ecs_group_create_action_t) -> &mut Self {
        let desc = self.query_desc_mut();
        desc.on_group_create = action;
        self
    }

    /// Specify the `on_group_delete` action.
    ///
    /// # Arguments
    ///
    /// * `action`: The action to execute when a group is deleted.
    fn on_group_delete(&mut self, action: sys::ecs_group_delete_action_t) -> &mut Self {
        let desc = self.query_desc_mut();
        desc.on_group_delete = action;
        self
    }
}

pub trait OrderByFn<T>
where
    T: ComponentId,
{
    fn to_extern_fn(self) -> OrderByFnPtr<T>;
}

impl<F, T: ComponentId> OrderByFn<T> for F
where
    F: Fn(Entity, &T, Entity, &T) -> i32,
{
    fn to_extern_fn(self) -> OrderByFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(e1: Entity, e1_data: &T, e2: Entity, e2_data: &T) -> i32
        where
            F: Fn(Entity, &T, Entity, &T) -> i32,
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(e1, e1_data, e2, e2_data)
        }

        output::<F, T>
    }
}

pub trait OrderByFnVoid {
    fn to_extern_fn(self) -> OrderByFnVoidPtr;
}

impl<F> OrderByFnVoid for F
where
    F: Fn(Entity, *const c_void, Entity, *const c_void) -> i32,
{
    fn to_extern_fn(self) -> OrderByFnVoidPtr {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F>(e1: Entity, e1_data: *const c_void, e2: Entity, e2_data: *const c_void) -> i32
        where
            F: Fn(Entity, *const c_void, Entity, *const c_void) -> i32,
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(e1, e1_data, e2, e2_data)
        }

        output::<F>
    }
}
