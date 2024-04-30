//! Query API. Queries are used to iterate over entities that match a filter.
//! Queries are better for persistence than filters, but are slower to create.

use std::{marker::PhantomData, os::raw::c_void, ptr::NonNull};

use crate::core::*;
use crate::sys;

/// Cached query implementation. Fast to iterate, but slower to create than `Filters`
///
///
#[derive(Clone)]
pub struct Query<'a, T>
where
    T: Iterable,
{
    pub(crate) query: NonNull<QueryT>,
    _phantom_lifetime: PhantomData<&'a ()>,
    _phantom: PhantomData<T>,
}

impl<'a, T> IterOperations for Query<'a, T>
where
    T: Iterable,
{
    #[inline(always)]
    fn retrieve_iter(&self) -> IterT {
        unsafe { sys::ecs_query_iter(self.world_ptr_mut(), self.query.as_ptr()) }
    }

    #[inline(always)]
    fn iter_next(&self, iter: &mut IterT) -> bool {
        unsafe { sys::ecs_query_next(iter) }
    }

    fn query_ptr(&self) -> *const QueryT {
        self.query.as_ptr()
    }

    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut IterT) -> bool {
        sys::ecs_query_next
    }
}

impl<'a, T> IterAPI<'a, (), T> for Query<'a, T>
where
    T: Iterable,
{
    fn as_entity(&self) -> EntityView<'a> {
        EntityView::new_from(self.world(), unsafe {
            sys::ecs_get_entity(self.query.as_ptr() as *const c_void)
        })
    }
}

impl<'a, T> IntoWorld<'a> for Query<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn world(&self) -> WorldRef<'a> {
        unsafe { WorldRef::from_ptr(self.query.as_ref().world) }
    }
}

impl<'a, T> Query<'a, T>
where
    T: Iterable,
{
    // /// Create a new query
    // ///
    // /// # Arguments
    // ///
    // /// * `world` - The world to create the query in
    // ///
    // /// # See also
    // ///
    // /// * C++ API: `query::query`
    // #[doc(alias = "query::query")]
    // pub fn new(world: impl IntoWorld<'a>) -> Self {
    //     let mut desc = sys::ecs_query_desc_t::default();
    //     T::register_ids_descriptor(world.world_ptr_mut(), &mut desc.filter);
    //     let mut filter: FilterT = Default::default();
    //     desc.filter.storage = &mut filter;
    //     let query =
    //         unsafe { NonNull::new_unchecked(sys::ecs_query_init(world.world_ptr_mut(), &desc)) };
    //     Self {
    //         world: world.world(),
    //         query,
    //         _phantom: std::marker::PhantomData,
    //     }
    // }

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
    pub fn new_ownership(query: NonNull<QueryT>) -> Self {
        Self {
            query,
            _phantom: std::marker::PhantomData,
            _phantom_lifetime: PhantomData,
        }
    }

    /// Create a new query from a query descriptor
    ///
    /// # Panics
    ///
    /// Panics if the query desc is faulty, such as a wrong name of a non-existent components being passed with `with_name`.
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
    pub(crate) fn new_from_desc(
        world: impl IntoWorld<'a>,
        desc: &mut sys::ecs_query_desc_t,
    ) -> Self {
        let query_ptr = unsafe { sys::ecs_query_init(world.world_ptr_mut(), desc) };
        ecs_assert!(
            !query_ptr.is_null(),
            "Failed to create query from query descriptor"
        );
        let query = unsafe { NonNull::new_unchecked(query_ptr) };
        Self {
            query,
            _phantom: PhantomData,
            _phantom_lifetime: PhantomData,
        }
    }

    /// get the query entity
    pub fn entity(&self) -> EntityView<'a> {
        EntityView::new_from(self.world(), unsafe { (*self.query.as_ptr()).entity })
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
        ecs_assert!(
            unsafe { (*self.query.as_ptr()).entity } != 0,
            "destruct() should only be called on queries associated with entities"
        );

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
    unsafe fn get_iter_raw(&mut self) -> IterT {
        unsafe { sys::ecs_query_iter(self.world_ptr_mut(), self.query.as_ptr()) }
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
        unsafe { sys::ecs_query_changed(self.query.as_ptr()) }
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
    pub fn group_info(&self, group_id: impl Into<Entity>) -> *const QueryGroupInfoT {
        unsafe { sys::ecs_query_get_group_info(self.query.as_ptr(), *group_id.into()) }
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
    pub fn group_context(&self, group_id: impl Into<Entity>) -> *mut c_void {
        let group_info = self.group_info(group_id);

        if !group_info.is_null() {
            unsafe { (*group_info).ctx }
        } else {
            std::ptr::null_mut()
        }
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
        // Only free if query is not associated with entity, such as system
        // queries and named queries. Named queries have to be either explicitly
        // deleted with the .destruct() method, or will be deleted when the
        // world is deleted.
        unsafe {
            if (*self.query.as_ptr()).entity != 0 {
                sys::ecs_query_fini(self.query.as_ptr());
            }
        }
    }
}
