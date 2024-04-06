//! Query API. Queries are used to iterate over entities that match a filter.
//! Queries are better for persistence than filters, but are slower to create.

use std::os::raw::c_void;

use crate::sys::{
    ecs_abort_, ecs_get_entity, ecs_os_api, ecs_query_changed, ecs_query_desc_t, ecs_query_fini,
    ecs_query_get_filter, ecs_query_get_group_info, ecs_query_init, ecs_query_iter, ecs_query_next,
    ecs_query_orphaned,
};

use super::{
    c_types::{FilterT, IterT, QueryGroupInfoT, QueryT},
    entity::Entity,
    filter::FilterView,
    iterable::Iterable,
    world::World,
    FlecsErrorCode, IntoEntityId, IterAPI, IterOperations,
};

/// Cached query implementation. Fast to iterate, but slower to create than `Filters`
#[derive(Clone)]
pub struct Query<'a, T>
where
    T: Iterable<'a>,
{
    pub world: World,
    pub query: *mut QueryT,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> IterOperations for Query<'a, T>
where
    T: Iterable<'a>,
{
    #[inline(always)]
    fn retrieve_iter(&self) -> IterT {
        unsafe { ecs_query_iter(self.world.raw_world, self.query) }
    }

    #[inline(always)]
    fn iter_next(&self, iter: &mut IterT) -> bool {
        unsafe { ecs_query_next(iter) }
    }

    fn filter_ptr(&self) -> *const FilterT {
        unsafe { ecs_query_get_filter(self.query) }
    }

    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut IterT) -> bool {
        ecs_query_next
    }
}

impl<'a, T> IterAPI<'a, T> for Query<'a, T>
where
    T: Iterable<'a>,
{
    fn as_entity(&self) -> Entity {
        Entity::new_from_existing_raw(self.world.raw_world, unsafe {
            ecs_get_entity(self.query as *const c_void)
        })
    }
}

impl<'a, T> Query<'a, T>
where
    T: Iterable<'a>,
{
    /// Create a new query
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the query in
    ///
    /// # See also
    ///
    /// * C++ API: `query::query`
    #[doc(alias = "query::query")]
    pub fn new(world: &World) -> Self {
        let mut desc = ecs_query_desc_t::default();
        T::register_ids_descriptor(world.raw_world, &mut desc.filter);
        let mut filter: FilterT = Default::default();
        desc.filter.storage = &mut filter;
        let query = unsafe { ecs_query_init(world.raw_world, &desc) };
        Self {
            world: world.clone(),
            query,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new query from a query descriptor
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the query in
    /// * `desc` - The query descriptor to create the query from
    ///
    /// # See also
    ///
    /// * C++ API: `query::query`
    #[doc(alias = "query::query")]
    pub fn new_ownership(world: &World, query: *mut QueryT) -> Self {
        Self {
            world: world.clone(),
            query,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new query from a query descriptor
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the query in
    /// * `desc` - The query descriptor to create the query from
    ///
    /// # See also
    ///
    /// * C++ API: `query::query`
    #[doc(alias = "query::query")]
    pub fn new_from_desc(world: &World, desc: &mut ecs_query_desc_t) -> Self {
        let obj = Self {
            world: world.clone(),
            query: unsafe { ecs_query_init(world.raw_world, desc) },
            _phantom: std::marker::PhantomData,
        };
        unsafe {
            if obj.query.is_null() {
                ecs_abort_(
                    FlecsErrorCode::InvalidParameter.to_int(),
                    file!().as_ptr() as *const i8,
                    line!() as i32,
                    std::ptr::null(),
                );

                if let Some(abort_func) = ecs_os_api.abort_ {
                    abort_func();
                };
            }

            if !desc.filter.terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func(desc.filter.terms_buffer as *mut _);
                }
            }
        };
        obj
    }

    /// Free the query
    /// Destroy a query. This operation destroys a query and its resources.
    /// If the query is used as the parent of subqueries, those subqueries will be
    /// orphaned and must be deinitialized as well.
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::destruct`
    #[doc(alias = "query_base::destruct")]
    pub fn destruct(self) {
        //calls drop
    }

    /// Get the iterator for the query
    ///
    /// # Arguments
    ///
    /// * `world` - The world to get the iterator for
    ///
    /// # See also
    ///
    /// * C++ API: `query::get_iter`
    #[doc(alias = "query::get_iter")]
    fn get_iter_raw(&mut self, world: &World) -> IterT {
        if !world.is_null() {
            self.world = world.clone();
        }
        unsafe { ecs_query_iter(self.world.raw_world, self.query) }
    }

    ///  Returns whether the query data changed since the last iteration.
    ///  This operation must be invoked before obtaining the iterator, as this will
    ///  reset the changed state. The operation will return true after:
    /// - new entities have been matched with
    /// - matched entities were deleted
    /// - matched components were changed
    ///
    /// # Returns
    ///
    /// return true if entities changed, otherwise false.
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::changed`
    #[doc(alias = "query_base::changed")]
    pub fn is_changed(&self) -> bool {
        unsafe { ecs_query_changed(self.query, std::ptr::null()) }
    }

    /// Returns whether query is orphaned.
    /// When the parent query of a subquery is deleted, it is left in an orphaned
    /// state. The only valid operation on an orphaned query is deleting it. Only
    /// subqueries can be orphaned.
    ///
    /// # Returns
    ///
    /// true if query is orphaned, otherwise false.
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::orphaned`
    #[doc(alias = "query_base::orphaned")]
    pub fn is_orphaned(&mut self) -> bool {
        unsafe { ecs_query_orphaned(self.query) }
    }

    /// Get info for group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The group id to get info for
    ///
    /// # Returns
    ///
    /// Returns a pointer to the group info
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::get_group_info`
    #[doc(alias = "query_base::get_group_info")]
    pub fn group_info(&self, group_id: impl IntoEntityId) -> *const QueryGroupInfoT {
        unsafe { ecs_query_get_group_info(self.query, group_id.get_id()) }
    }

    /// Get context for group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The group id to get context for
    ///
    /// # Returns
    ///
    /// Returns a (void) pointer to the group context
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::group_ctx`
    #[doc(alias = "query_base::group_ctx")]
    pub fn group_context(&self, group_id: impl IntoEntityId) -> *mut c_void {
        let group_info = self.group_info(group_id);

        if !group_info.is_null() {
            unsafe { (*group_info).ctx }
        } else {
            std::ptr::null_mut()
        }
    }

    /// Get the filter of the query as read only
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::filter`
    #[doc(alias = "query_base::filter")]
    pub fn filter(&self) -> FilterView<'a, T> {
        FilterView::<T>::new(&self.world, unsafe { ecs_query_get_filter(self.query) })
    }
}

impl<'a, T> Drop for Query<'a, T>
where
    T: Iterable<'a>,
{
    /// Destroy a query. This operation destroys a query and its resources.
    /// If the query is used as the parent of subqueries, those subqueries will be orphaned
    /// and must be deinitialized as well.
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::~query_base`
    #[doc(alias = "query_base::~query_base")]
    fn drop(&mut self) {
        unsafe { ecs_query_fini(self.query) }
    }
}
