//! Query API. Queries are used to iterate over entities that match a filter.
//! Queries are better for persistence than filters, but are slower to create.

use std::{marker::PhantomData, os::raw::c_void, ptr::NonNull};

use sys::ecs_get_alive;

use crate::core::*;
use crate::sys;

/// Queries are used to iterate over entities that match a certain filter of components.
/// They can be either:
/// - cached, which means they are stored in the world and can be retrieved by name or entity.
///   They don't go out of scope until explicitly destroyed.
///   They are slower to create than uncached queries, but faster to iterate.
/// - uncached, which means they are created on the fly and are only valid for the duration of the query, scope.
///   They are faster to create than cached queries, but slower to iterate.
///
/// # Safety
///
/// Queries are reference counter and won't cause any lifetime issues nor dangling references.
/// You need to ensure that you're holding no query objects anymore when the world is destroyed.
/// This will otherwise panic.
pub struct Query<T>
where
    T: Iterable,
{
    pub(crate) query: NonNull<QueryT>,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Query<T>
where
    T: Iterable,
{
    fn clone(&self) -> Self {
        unsafe { Query::<T>::new_from(self.query) }
    }
}

impl<T> Drop for Query<T>
where
    T: Iterable,
{
    fn drop(&mut self) {
        self.world().world_ctx_mut().dec_query_ref_count();

        // Only free if query is not associated with entity. Queries are associated with entities
        // when they are either named or cached, such as system, cached queries and named queries. These queries have to be either explicitly
        // deleted with the .destruct() method, or will be deleted when the
        // world is deleted.
        unsafe {
            if self.query.as_ref().entity == 0
                && sys::flecs_poly_release_(self.query.as_ptr() as *mut c_void) == 0
            {
                sys::ecs_query_fini(self.query.as_ptr());
            }
        }
    }
}

impl<T> IterOperations for Query<T>
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

impl<T> IterAPI<(), T> for Query<T>
where
    T: Iterable,
{
    #[inline(always)]
    fn entity(&self) -> EntityView {
        EntityView::new_from(self.world(), unsafe { (*self.query.as_ptr()).entity })
    }

    #[inline(always)]
    fn world(&self) -> WorldRef<'_> {
        unsafe { WorldRef::from_ptr(self.world_ptr_mut()) }
    }

    #[inline(always)]
    fn world_ptr_mut(&self) -> *mut sys::ecs_world_t {
        unsafe { (*self.query_ptr()).world }
    }
}

impl<T> Query<T>
where
    T: Iterable,
{
    /// wraps the query pointer in a new query
    ///
    /// # Safety
    ///
    /// this is unsafe due to the fact that the type of the query is not checked.
    /// the user is responsible for ensuring that the query is of the correct type.
    /// if not possible, only use `.iter` functions that do not pass in the components in the callback
    ///
    /// # Arguments
    ///
    /// * `query` - The query pointer to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `query::query`
    #[doc(alias = "query::query")]
    #[inline]
    pub unsafe fn new_from(query: NonNull<QueryT>) -> Self {
        unsafe { sys::flecs_poly_claim_(query.as_ptr() as *mut c_void) };

        let new_query = Self {
            query,
            _phantom: std::marker::PhantomData,
        };

        new_query.world().world_ctx_mut().inc_query_ref_count();
        new_query
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
    pub(crate) fn new_from_desc<'a>(
        world: impl IntoWorld<'a>,
        desc: &mut sys::ecs_query_desc_t,
    ) -> Self {
        let world_ptr = world.world_ptr_mut();

        let query_ptr = unsafe { sys::ecs_query_init(world_ptr, desc) };

        ecs_assert!(
            !query_ptr.is_null(),
            "Failed to create query from query descriptor"
        );

        let query = unsafe { NonNull::new_unchecked(query_ptr) };

        let new_query = Self {
            query,
            _phantom: PhantomData,
        };

        new_query.world().world_ctx_mut().inc_query_ref_count();
        new_query
    }

    pub(crate) fn new_from_entity<'a>(
        world: impl IntoWorld<'a>,
        entity: impl Into<Entity>,
    ) -> Option<Query<()>> {
        let world_ptr = world.world_ptr_mut();
        let entity = *entity.into();
        unsafe {
            if ecs_get_alive(world_ptr, entity) != 0 {
                let query_poly = sys::ecs_get_id(
                    world_ptr,
                    entity,
                    ecs_pair(flecs::Poly::ID, flecs::Query::ID),
                );

                if !query_poly.is_null() {
                    sys::flecs_poly_claim_(query_poly as *mut c_void);
                    let query = NonNull::new_unchecked(query_poly as *mut QueryT);
                    let new_query = Query::<()>::new_from(query);
                    new_query.world().world_ctx_mut().inc_query_ref_count();
                    return Some(new_query);
                }
            }
            None
        }
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

        if unsafe { (*self.query.as_ptr()).entity } != 0 {
            if unsafe { sys::flecs_poly_release_(self.query.as_ptr() as *mut c_void) } > 0 {
                panic!("The code base still has lingering references to `Query` objects. This is a bug in the user code. 
                Please ensure that all `Query` objects are out of scope that are a clone/copy of the current one.");
            }
            unsafe { sys::ecs_query_fini(self.query.as_ptr()) };
        }
    }

    pub fn reference_count(&self) -> i32 {
        unsafe { sys::flecs_poly_refcount(self.query.as_ptr() as *mut c_void) }
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
