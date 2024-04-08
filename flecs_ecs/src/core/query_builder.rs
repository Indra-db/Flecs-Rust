//! Cached query implementation. Fast to iterate, but slower to create than Filter

use std::{
    ffi::CStr,
    ops::Deref,
    os::raw::{c_int, c_void},
};

use crate::sys::{
    ecs_ctx_free_t, ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t, ecs_group_by_action_t,
    ecs_group_create_action_t, ecs_group_delete_action_t, ecs_order_by_action_t, ecs_query_desc_t,
};

use super::{
    builder::Builder,
    c_types::{TermT, SEPARATOR},
    component_registration::ComponentId,
    filter_builder::{FilterBuilder, FilterBuilderImpl},
    iterable::{Filterable, Iterable},
    query::Query,
    term::TermBuilder,
    world::World,
    EntityT, IdT, IntoEntityId, IntoWorld, TableT, Term, WorldT,
};

/// Fast to iterate, but slower to create than Filter
pub struct QueryBuilder<'a, T>
where
    T: Iterable,
{
    pub filter_builder: FilterBuilder<'a, T>,
    pub desc: ecs_query_desc_t,
}

impl<'a, T> Deref for QueryBuilder<'a, T>
where
    T: Iterable,
{
    type Target = FilterBuilder<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.filter_builder
    }
}

impl<'a, T> QueryBuilder<'a, T>
where
    T: Iterable,
{
    /// Create a new query builder
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    ///
    /// See also
    ///
    /// * C++ API: `builder::builder`
    #[doc(alias = "builder::builder")]
    pub fn new(world: &'a World) -> Self {
        let mut desc = Default::default();

        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::new_from_desc(world, &mut desc.filter, 0),
        };

        let entity_desc = ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.filter.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }

    /// Create a new query builder with a name
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `name` - The name of the observer
    ///
    /// See also
    ///
    /// * C++ API: `query_builder::query_builder`
    #[doc(alias = "query_builder::query_builder")]
    pub fn new_named(world: &'a World, name: &CStr) -> Self {
        let mut desc = Default::default();

        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::new_from_desc(world, &mut desc.filter, 0),
        };

        let entity_desc = ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.filter.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }

    /// Create a new query builder from an existing descriptor
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `desc` - The descriptor to create the observer from
    ///
    /// See also
    ///
    /// * C++ API: `query_builder_i::query_builder_i`
    #[doc(alias = "query_builder_i::query_builder_i")]
    pub fn new_from_desc(world: &'a World, desc: &mut ecs_query_desc_t) -> Self {
        Self {
            desc: *desc,
            filter_builder: FilterBuilder::new_from_desc(world, &mut desc.filter, 0),
        }
    }

    /// Create a new query builder from an existing descriptor with a term index
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `desc` - The descriptor to create the observer from
    /// * `term_index` - The index of the term to create the observer from
    ///
    /// See also
    ///
    /// * C++ API: `query_builder_i::query_builder_i`
    #[doc(alias = "query_builder_i::query_builder_i")]
    pub fn new_from_desc_term_index(
        world: &'a World,
        desc: &mut ecs_query_desc_t,
        term_index: i32,
    ) -> Self {
        let mut obj = Self {
            desc: *desc,
            filter_builder: FilterBuilder::new_from_desc(world, &mut desc.filter, term_index),
        };

        let entity_desc = ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.filter.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }
}

impl<'a, T> Filterable<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    fn current_term(&mut self) -> &mut TermT {
        unsafe { &mut *self.filter_builder.term.term_ptr }
    }

    fn next_term(&mut self) {
        self.filter_builder.next_term();
    }
}

impl<'a, T> FilterBuilderImpl<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn desc_filter_mut(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.desc.filter
    }

    #[inline]
    fn expr_count_mut(&mut self) -> &mut i32 {
        self.filter_builder.expr_count_mut()
    }

    #[inline]
    fn term_index_mut(&mut self) -> &mut i32 {
        self.filter_builder.term_index_mut()
    }
}

impl<'a, T> TermBuilder<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn term_mut(&mut self) -> &mut Term<'a> {
        self.filter_builder.term_mut()
    }

    #[inline]
    fn term_ptr_mut(&mut self) -> *mut TermT {
        self.filter_builder.term_ptr_mut()
    }

    #[inline]
    fn term_id_ptr_mut(&mut self) -> *mut super::c_types::TermIdT {
        self.filter_builder.term_id_ptr_mut()
    }
}

impl<'a, T> Builder<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    type BuiltType = Query<'a, T>;

    /// Build the `observer_builder` into an query
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        let world = &self.filter_builder.world;
        Query::<T>::new_from_desc(world, &mut self.desc)
    }
}

// This is a raw function pointer type, compatible with C to pass to the desc.
type OrderByFn<T> = extern "C" fn(EntityT, *const T, EntityT, *const T) -> c_int;
// Assuming some imports and definitions from your previous example, and adding the required ones for this example.
type GroupByFn = extern "C" fn(*mut WorldT, *mut TableT, IdT, *mut c_void) -> u64;

pub trait QueryBuilderImpl<'a>: FilterBuilderImpl<'a> {
    fn desc_query_mut(&mut self) -> &mut ecs_query_desc_t;

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
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::order_by`
    #[doc(alias = "query_builder_i::order_by")]
    fn order_by<T>(&mut self, compare: OrderByFn<T>) -> &mut Self
    where
        T: ComponentId,
    {
        let cmp: ecs_order_by_action_t = Some(unsafe { std::mem::transmute(compare) });
        self.order_by_id(T::get_id(self.get_world()), cmp);
        self
    }

    /// Sorts the output of a query.
    ///
    /// This is similar to `order_by<T>`, but uses a component identifier instead.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to sort.
    /// * `compare`: The compare function used to sort the components.
    /// # See also
    ///
    /// * C++ API: `query_builder_i::order_by`
    #[doc(alias = "query_builder_i::order_by")]
    fn order_by_id(
        &mut self,
        component: impl IntoEntityId,
        compare: ecs_order_by_action_t,
    ) -> &mut Self {
        let desc = self.desc_query_mut();
        desc.order_by = compare;
        desc.order_by_component = component.get_id();
        self
    }

    /// Group and sort matched tables.
    ///
    /// This function is similar to `group_by<T>`, but uses a default `group_by` action.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component used to determine the group rank.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by`
    #[doc(alias = "query_builder_i::group_by")]
    fn group_by<T>(&mut self) -> &mut Self
    where
        T: ComponentId,
    {
        self.group_by_id_fn(T::get_id(self.get_world()), None)
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
    /// # Type Parameters
    ///
    /// * `T`: The component used to determine the group rank.
    ///
    /// # Arguments
    ///
    /// * `group_by_action`: Callback that determines group id for table.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by`
    #[doc(alias = "query_builder_i::group_by")]
    fn group_by_fn<T>(&mut self, group_by_action: ecs_group_by_action_t) -> &mut Self
    where
        T: ComponentId,
    {
        self.group_by_id_fn(T::get_id(self.get_world()), group_by_action);
        self
    }

    /// Group and sort matched tables.
    ///
    /// This is similar to `group_by<T>`, but uses a component identifier instead.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to determine the group rank.
    /// * `group_by_action`: Callback that determines group id for table.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by`
    #[doc(alias = "query_builder_i::group_by")]
    fn group_by_id_fn(
        &mut self,
        component: impl IntoEntityId,
        group_by_action: ecs_group_by_action_t,
    ) -> &mut Self {
        let desc = self.desc_query_mut();
        desc.group_by = group_by_action;
        desc.group_by_id = component.get_id();
        self
    }

    /// Group and sort matched tables.
    ///
    /// This is similar to `group_by_default<T>`, but uses a component identifier instead.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to determine the group rank.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by`
    #[doc(alias = "query_builder_i::group_by")]
    fn group_by_id(&mut self, component: impl IntoEntityId) -> &mut Self {
        self.group_by_id_fn(component, None)
    }

    /// Specify context to be passed to the `group_by` function.
    ///
    /// # Arguments
    ///
    /// * `ctx`: Context to pass to the `group_by` function.
    /// * `ctx_free`: Function to clean up the context (called when the query is deleted).
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by_ctx`
    #[doc(alias = "query_builder_i::group_by_ctx")]
    fn group_by_ctx(&mut self, ctx: *mut c_void, ctx_free: ecs_ctx_free_t) -> &mut Self {
        let desc = self.desc_query_mut();
        desc.group_by_ctx = ctx;
        desc.group_by_ctx_free = ctx_free;
        self
    }

    /// Specify the `on_group_create` action.
    ///
    /// # Arguments
    ///
    /// * `action`: The action to execute when a group is created.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::on_group_create`
    #[doc(alias = "query_builder_i::on_group_create")]
    fn on_group_create(&mut self, action: ecs_group_create_action_t) -> &mut Self {
        let desc = self.desc_query_mut();
        desc.on_group_create = action;
        self
    }

    /// Specify the `on_group_delete` action.
    ///
    /// # Arguments
    ///
    /// * `action`: The action to execute when a group is deleted.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::on_group_delete`
    #[doc(alias = "query_builder_i::on_group_delete")]
    fn on_group_delete(&mut self, action: ecs_group_delete_action_t) -> &mut Self {
        let desc = self.desc_query_mut();
        desc.on_group_delete = action;
        self
    }

    /// Specify parent query (creates subquery)
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::observable`
    #[doc(alias = "query_builder_i::observable")]
    fn observable<T: Iterable>(&mut self, parent: &Query<T>) -> &mut Self {
        let desc = self.desc_query_mut();
        desc.parent = parent.query;
        self
    }
}

impl<'a, T> QueryBuilderImpl<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn desc_query_mut(&mut self) -> &mut ecs_query_desc_t {
        &mut self.desc
    }
}

impl<'a, T: Iterable> IntoWorld<'a> for QueryBuilder<'a, T> {
    fn get_world(&self) -> Option<&'a World> {
        self.filter_builder.get_world()
    }
}
