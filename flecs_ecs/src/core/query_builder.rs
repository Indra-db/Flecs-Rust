use std::{
    ops::Deref,
    os::raw::{c_int, c_void},
};

use super::{
    builder::Builder,
    c_binding::bindings::{
        ecs_ctx_free_t, ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t,
        ecs_group_by_action_t, ecs_group_create_action_t, ecs_group_delete_action_t,
        ecs_order_by_action_t, ecs_query_desc_t,
    },
    c_types::{EntityT, IdT, TableT, TermT, WorldT, SEPARATOR},
    component_registration::CachedComponentData,
    filter_builder::{FilterBuilder, FilterBuilderImpl},
    iterable::{Filterable, Iterable},
    query::{Query, QueryBase},
    term::TermBuilder,
    world::World,
};

// todo! does this need its own world? filter builder already has one?
pub struct QueryBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    pub filter_builder: FilterBuilder<'a, 'w, T>,
    pub desc: ecs_query_desc_t,
    pub world: &'w World,
}

impl<'a, 'w, T> Deref for QueryBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    type Target = FilterBuilder<'a, 'w, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.filter_builder
    }
}

impl<'a, 'w, T> QueryBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    pub fn new(world: &'w World) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::new_with_desc(world, &mut desc.filter, 0),
            world,
        };
        T::populate(&mut obj);
        obj
    }

    pub fn new_named(world: &'w World, name: &str) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            filter_builder: FilterBuilder::new(world),
            world,
        };
        T::populate(&mut obj);
        let entity_desc = ecs_entity_desc_t {
            name: std::ffi::CString::new(name).unwrap().into_raw(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.filter.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        obj
    }
}

impl<'a, 'w, T> Filterable for QueryBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    fn get_world(&self) -> *mut WorldT {
        self.filter_builder.world.raw_world
    }

    fn current_term(&mut self) -> &mut TermT {
        self.filter_builder.current_term()
    }

    fn next_term(&mut self) {
        self.filter_builder.next_term()
    }
}

impl<'a, 'w, T> FilterBuilderImpl for QueryBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_filter(&mut self) -> &mut ecs_filter_desc_t {
        self.filter_builder.get_desc_filter()
    }

    #[inline]
    fn get_expr_count(&mut self) -> &mut i32 {
        self.filter_builder.get_expr_count()
    }

    #[inline]
    fn get_term_index(&mut self) -> &mut i32 {
        self.filter_builder.get_term_index()
    }
}

impl<'a, 'w, T> TermBuilder for QueryBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_world(&self) -> *mut WorldT {
        self.filter_builder.world.raw_world
    }

    #[inline]
    fn get_term(&mut self) -> &mut super::term::Term {
        self.filter_builder.get_term()
    }

    #[inline]
    fn get_raw_term(&mut self) -> *mut TermT {
        self.filter_builder.get_raw_term()
    }

    #[inline]
    fn get_term_id(&mut self) -> *mut super::c_types::TermIdT {
        self.filter_builder.get_term_id()
    }
}

impl<'a, 'w, T> Builder for QueryBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    type BuiltType = Query<'a, 'w, T>;

    fn build(&mut self) -> Self::BuiltType {
        let desc_filter = self.filter_builder.desc;
        self.desc.filter = desc_filter;
        Query::<'a, 'w, T>::new_from_desc(self.world, &mut self.desc)
    }
}

// This is a raw function pointer type, compatible with C to pass to the desc.
type OrderByFn<T> = extern "C" fn(EntityT, *const T, EntityT, *const T) -> c_int;
// Assuming some imports and definitions from your previous example, and adding the required ones for this example.
type GroupByFn = extern "C" fn(*mut WorldT, *mut TableT, IdT, *mut c_void) -> u64;

pub trait QueryBuilderImpl: FilterBuilderImpl {
    fn get_desc_query(&mut self) -> &mut ecs_query_desc_t;

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
    /// # Parameters
    ///
    /// * `compare`: The compare function used to sort the components.
    ///
    /// # C++ API equivalent
    ///
    /// `query_builder_i::order_by`
    fn order_by<T>(&mut self, compare: OrderByFn<T>) -> &mut Self
    where
        T: CachedComponentData,
    {
        let cmp: ecs_order_by_action_t = Some(unsafe { std::mem::transmute(compare) });
        self.order_by_id(T::get_id(self.get_world()), cmp);
        self
    }

    /// Sorts the output of a query.
    ///
    /// This is similar to `order_by<T>`, but uses a component identifier instead.
    ///
    /// # Parameters
    ///
    /// * `component`: The component used to sort.
    /// * `compare`: The compare function used to sort the components.
    /// # C++ API equivalent
    ///
    /// `query_builder_i::order_by`
    fn order_by_id(&mut self, component: IdT, compare: ecs_order_by_action_t) -> &mut Self {
        let desc = self.get_desc_query();
        desc.order_by = compare;
        desc.order_by_component = component;
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
    /// # Type Parameters
    ///
    /// * `T`: The component used to determine the group rank.
    ///
    /// # Parameters
    ///
    /// * `group_by_action`: Callback that determines group id for table.
    ///
    /// # C++ API equivalent
    ///
    /// `query_builder_i::group_by`
    fn group_by<T>(&mut self, group_by_action: GroupByFn) -> &mut Self
    where
        T: CachedComponentData,
    {
        let action: ecs_group_by_action_t = Some(unsafe { std::mem::transmute(group_by_action) });
        self.group_by_id(T::get_id(self.get_world()), action);
        self
    }

    /// Group and sort matched tables.
    ///
    /// This is similar to `group_by<T>`, but uses a component identifier instead.
    ///
    /// # Parameters
    ///
    /// * `component`: The component used to determine the group rank.
    /// * `group_by_action`: Callback that determines group id for table.
    ///
    /// # C++ API equivalent
    ///
    /// `query_builder_i::group_by`
    fn group_by_id(&mut self, component: IdT, group_by_action: ecs_group_by_action_t) -> &mut Self {
        let desc = self.get_desc_query();
        desc.group_by = group_by_action;
        desc.group_by_id = component;
        self
    }

    /// Group and sort matched tables.
    ///
    /// This function is similar to `group_by<T>`, but uses a default `group_by` action.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component used to determine the group rank.
    fn group_by_default<T>(&mut self) -> &mut Self
    where
        T: CachedComponentData,
    {
        self.group_by_id(T::get_id(self.get_world()), None)
    }

    /// Group and sort matched tables.
    ///
    /// This is similar to `group_by_default<T>`, but uses a component identifier instead.
    ///
    /// # Parameters
    ///
    /// * `component`: The component used to determine the group rank.
    fn group_by_with_component(&mut self, component: IdT) -> &mut Self {
        self.group_by_id(component, None)
    }

    /// Specify context to be passed to the `group_by` function.
    ///
    /// # Parameters
    ///
    /// * `ctx`: Context to pass to the `group_by` function.
    /// * `ctx_free`: Function to clean up the context (called when the query is deleted).
    fn group_by_ctx(&mut self, ctx: *mut c_void, ctx_free: ecs_ctx_free_t) -> &mut Self {
        let desc = self.get_desc_query();
        desc.group_by_ctx = ctx;
        desc.group_by_ctx_free = ctx_free;
        self
    }

    /// Specify the `on_group_create` action.
    ///
    /// # Parameters
    ///
    /// * `action`: The action to execute when a group is created.
    fn on_group_create(&mut self, action: ecs_group_create_action_t) -> &mut Self {
        let desc = self.get_desc_query();
        desc.on_group_create = action;
        self
    }

    /// Specify the `on_group_delete` action.
    ///
    /// # Parameters
    ///
    /// * `action`: The action to execute when a group is deleted.
    fn on_group_delete(&mut self, action: ecs_group_delete_action_t) -> &mut Self {
        let desc = self.get_desc_query();
        desc.on_group_delete = action;
        self
    }

    /// Specify parent query (creates subquery)
    fn observable<'a, 'w, T: Iterable<'a>>(&mut self, parent: &QueryBase<'a, 'w, T>) -> &mut Self {
        let desc = self.get_desc_query();
        desc.parent = parent.query;
        self
    }
}

impl<'a, 'w, T> QueryBuilderImpl for QueryBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_query(&mut self) -> &mut ecs_query_desc_t {
        &mut self.desc
    }
}
