//! Query API. Queries are used to iterate over entities that match a filter.
//! Queries are better for persistence than filters, but are slower to create.

use std::{marker::PhantomData, os::raw::c_void, ptr::NonNull};

use crate::core::*;
use crate::sys;

/// Cached query implementation. Fast to iterate, but slower to create than `Filters`
#[derive(Clone)]
pub struct Query<'a, T>
where
    T: Iterable,
{
    pub world: WorldRef<'a>,
    pub query: NonNull<QueryT>,
    _phantom: PhantomData<T>,
}

impl<'a, T> IterOperations for Query<'a, T>
where
    T: Iterable,
{
    #[inline(always)]
    fn retrieve_iter(&self) -> IterT {
        unsafe { sys::ecs_query_iter(self.world.world_ptr_mut(), self.query.as_ptr()) }
    }

    #[inline(always)]
    fn iter_next(&self, iter: &mut IterT) -> bool {
        unsafe { sys::ecs_query_next(iter) }
    }

    fn filter_ptr(&self) -> *const FilterT {
        unsafe { sys::ecs_query_get_filter(self.query.as_ptr()) }
    }

    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut IterT) -> bool {
        sys::ecs_query_next
    }
}

impl<'a, T> IterAPI<'a, T> for Query<'a, T>
where
    T: Iterable,
{
    fn as_entity(&self) -> EntityView<'a> {
        EntityView::new_from(self.world, unsafe {
            sys::ecs_get_entity(self.query.as_ptr() as *const c_void)
        })
    }
}

impl<'a, T> Query<'a, T>
where
    T: Iterable,
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
    pub fn new(world: impl IntoWorld<'a>) -> Self {
        let mut desc = sys::ecs_query_desc_t::default();
        T::register_ids_descriptor(world.world_ptr_mut(), &mut desc.filter);
        let mut filter: FilterT = Default::default();
        desc.filter.storage = &mut filter;
        let query =
            unsafe { NonNull::new_unchecked(sys::ecs_query_init(world.world_ptr_mut(), &desc)) };
        Self {
            world: world.world(),
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
    pub fn new_ownership(world: impl IntoWorld<'a>, query: NonNull<QueryT>) -> Self {
        Self {
            world: world.world(),
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
    pub fn new_from_desc(world: impl IntoWorld<'a>, desc: &mut sys::ecs_query_desc_t) -> Self {
        NonNull::new(unsafe { sys::ecs_query_init(world.world_ptr_mut(), desc) })
            .map(|query| {
                let obj = Self {
                    world: world.world(),
                    query,
                    _phantom: PhantomData,
                };

                if !desc.filter.terms_buffer.is_null() {
                    unsafe {
                        if let Some(free_func) = sys::ecs_os_api.free_ {
                            free_func(desc.filter.terms_buffer as *mut _);
                        }
                    }
                }
                obj
            })
            .expect("Failed to create query.")
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
    fn get_iter_raw(&mut self, world: &'a World) -> IterT {
        self.world = world.world();
        unsafe { sys::ecs_query_iter(self.world.world_ptr_mut(), self.query.as_ptr()) }
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
        unsafe { sys::ecs_query_changed(self.query.as_ptr(), std::ptr::null()) }
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
    pub fn is_orphaned(&self) -> bool {
        unsafe { sys::ecs_query_orphaned(self.query.as_ptr()) }
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
    pub fn group_info(&self, group_id: impl IntoEntity) -> *const QueryGroupInfoT {
        unsafe { sys::ecs_query_get_group_info(self.query.as_ptr(), group_id.get_id()) }
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
    pub fn group_context(&self, group_id: impl IntoEntity) -> *mut c_void {
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
        FilterView::<T>::new(self.world, unsafe {
            sys::ecs_query_get_filter(self.query.as_ptr())
        })
    }
}

impl<'a, T> Drop for Query<'a, T>
where
    T: Iterable,
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
        unsafe { sys::ecs_query_fini(self.query.as_ptr()) }
    }
}
