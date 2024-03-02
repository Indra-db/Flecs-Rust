//! Query API. Queries are used to iterate over entities that match a filter.
//! Queries are better for persistance than filters, but are slower to create.

use std::ops::{Deref, DerefMut};
use std::os::raw::{c_char, c_void};

use crate::ecs_assert;

use super::c_binding::bindings::{
    _ecs_abort, ecs_filter_str, ecs_filter_t, ecs_get_entity, ecs_os_api, ecs_query_changed,
    ecs_query_desc_t, ecs_query_fini, ecs_query_get_filter, ecs_query_get_group_info,
    ecs_query_init, ecs_query_iter, ecs_query_next, ecs_query_orphaned, ecs_table_lock,
    ecs_table_unlock,
};
use super::entity::Entity;
use super::filter::FilterView;
use super::iter::Iter;
use super::iterable::Iterable;
use super::term::{Term, TermType};
use super::world::World;
use super::{c_types::*, FlecsErrorCode};

/// Cached query implementation. Fast to iterate, but slower to create than `Filters`
pub struct QueryBase<'a, T>
where
    T: Iterable<'a>,
{
    pub world: World,
    pub query: *mut QueryT,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> QueryBase<'a, T>
where
    T: Iterable<'a>,
{
    /// Create a query base, not public API
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the query in
    /// * `query` - The query to create
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::query_base`
    #[doc(alias = "query_base::query_base")]
    fn new(world: &World, query: *mut QueryT) -> Self {
        Self {
            world: world.clone(),
            query,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a query base from a query descriptor, not public API
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the query in
    /// * `desc` - The query descriptor to create the query from
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::query_base`
    #[doc(alias = "query_base::query_base")]
    fn new_from_desc(world: &World, desc: *mut ecs_query_desc_t) -> Self {
        let obj = Self {
            world: world.clone(),
            query: unsafe { ecs_query_init(world.raw_world, desc) },
            _phantom: std::marker::PhantomData,
        };
        unsafe {
            if obj.query.is_null() {
                _ecs_abort(
                    FlecsErrorCode::InvalidParameter.to_int(),
                    file!().as_ptr() as *const i8,
                    line!() as i32,
                    std::ptr::null(),
                );

                if let Some(abort_func) = ecs_os_api.abort_ {
                    abort_func();
                };
            }

            if !(*desc).filter.terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func((*desc).filter.terms_buffer as *mut _);
                }
            }
        };
        obj
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
    pub fn changed(&mut self) -> bool {
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
    pub fn orphaned(&mut self) -> bool {
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
    pub fn get_group_info(&mut self, group_id: u64) -> *const QueryGroupInfoT {
        unsafe { ecs_query_get_group_info(self.query, group_id) }
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
    pub fn get_group_context(&mut self, group_id: u64) -> *mut c_void {
        let group_info = self.get_group_info(group_id);

        if !group_info.is_null() {
            unsafe { (*group_info).ctx }
        } else {
            std::ptr::null_mut()
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
    pub fn destruct(mut self) {
        unsafe { ecs_query_fini(self.query) }
        self.query = std::ptr::null_mut();
    }

    fn each_term(&self, mut func: impl FnMut(Term), query: *mut QueryT) {
        unsafe {
            let filter = ecs_query_get_filter(query);
            for i in 0..(*filter).term_count {
                let term = Term::new(
                    Some(&self.world),
                    TermType::Term(*(*filter).terms.add(i as usize)),
                );
                func(term);
            }
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

    /// Get the Term at index stored on the filter of the query
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the term to get
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::term`
    #[doc(alias = "query_base::term")]
    fn term(&self, index: i32) -> Term {
        let filter: *const ecs_filter_t = unsafe { ecs_query_get_filter(self.query) };
        ecs_assert!(
            !filter.is_null(),
            FlecsErrorCode::InvalidParameter,
            "query filter is null"
        );
        Term::new(
            Some(&self.world),
            TermType::Term(unsafe { *(*filter).terms.add(index as usize) }),
        )
    }

    /// Get the number of terms in the filter of the query
    ///
    /// # Returns
    ///
    /// The number of terms in the filter of the query
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::term_count`
    #[doc(alias = "query_base::term_count")]
    fn field_count(&self) -> i32 {
        unsafe { (*ecs_query_get_filter(self.query)).term_count }
    }

    /// Convert filter to string expression. Convert filter terms to a string expression.
    /// The resulting expression can be parsed to create the same filter.
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::str`
    #[doc(alias = "query_base::str")]
    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    fn to_string(&self) -> String {
        let filter = unsafe { ecs_query_get_filter(self.query) };
        let result: *mut c_char =
            unsafe { ecs_filter_str(self.world.raw_world, filter as *const _) };
        let rust_string =
            String::from(unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = ecs_os_api.free_ {
                free_func(result as *mut _);
            }
        }
        rust_string
    }

    /// Get the query as an `Entity`
    ///
    /// # Returns
    ///
    /// The query as an `Entity`
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::entity`
    #[doc(alias = "query_base::entity")]
    pub fn entity(&self) -> Entity {
        Entity::new_from_existing_raw(self.world.raw_world, unsafe {
            ecs_get_entity(self.query as *const c_void)
        })
    }
}

/// Cached query implementation. Fast to iterate, but slower to create than `Filters`
pub struct Query<'a, T>
where
    T: Iterable<'a>,
{
    pub base: QueryBase<'a, T>,
}

impl<'a, T> Deref for Query<'a, T>
where
    T: Iterable<'a>,
{
    type Target = QueryBase<'a, T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a, T> DerefMut for Query<'a, T>
where
    T: Iterable<'a>,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
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
            base: QueryBase::new(world, query),
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
            base: QueryBase::new(world, query),
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
    pub fn new_from_desc(world: &World, desc: *mut ecs_query_desc_t) -> Self {
        Self {
            base: QueryBase::new_from_desc(world, desc),
        }
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

    // TODO once we have tests in place, I will split this functionality up into multiple functions, which should give a small performance boost
    // by caching if the query has used a "is_ref" operation.
    // is_ref is true for any query that contains fields that are not matched on the entity itself
    // so parents, prefabs but also singletons, or fields that are matched on a fixed entity (.with<Foo>().src(my_entity))
    /// Each iterator.
    /// The "each" iterator accepts a function that is invoked for each matching entity.
    /// The following function signatures is valid:
    ///  - func(comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::each`
    #[doc(alias = "iterable::each")]
    pub fn each(&mut self, mut func: impl FnMut(T::TupleType)) {
        unsafe {
            let mut iter = ecs_query_iter(self.world.raw_world, self.query);

            while ecs_query_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(self.world.raw_world, iter.table);

                for i in 0..iter_count {
                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };
                    func(tuple);
                }

                ecs_table_unlock(self.world.raw_world, iter.table);
            }
        }
    }

    /// Each iterator.
    /// The "each" iterator accepts a function that is invoked for each matching entity.
    /// The following function signatures is valid:
    ///  - func(e : Entity , comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::each`
    #[doc(alias = "iterable::each")]
    pub fn each_entity(&mut self, mut func: impl FnMut(&mut Entity, T::TupleType)) {
        unsafe {
            let mut iter = ecs_query_iter(self.world.raw_world, self.query);

            while ecs_query_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(self.world.raw_world, iter.table);

                // TODO random thought, I think I can determine the elements is a ref or not before the for loop and then pass two arrays with the indices of the ref and non ref elements
                // I will come back to this in the future, my thoughts are somewhere else right now. If my assumption is correct, this will get rid of the branch in the for loop
                // and potentially allow for more conditions for vectorization to happen. This could potentially offer a (small) performance boost since the branch predictor avoids probably
                // most of the cost since the branch is almost always the same.
                // update: I believe it's not possible due to not knowing the order of the components in the tuple. I will leave this here for now, maybe I will come back to it in the future.
                for i in 0..iter_count {
                    let mut entity =
                        Entity::new_from_existing_raw(self.world.raw_world, *iter.entities.add(i));

                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };

                    func(&mut entity, tuple);
                }

                ecs_table_unlock(self.world.raw_world, iter.table);
            }
        }
    }

    /// iter iterator.
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut Iter, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Iter iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::iter`
    #[doc(alias = "iterable::iter")]
    pub fn iter(&mut self, mut func: impl FnMut(&Iter, T::TupleSliceType)) {
        unsafe {
            let mut iter = ecs_query_iter(self.world.raw_world, self.query);

            while ecs_query_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(self.world.raw_world, iter.table);

                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_slices_with_ref(
                        array_components,
                        is_ref_array_components,
                        iter_count,
                    )
                } else {
                    T::get_tuple_slices(array_components, iter_count)
                };
                let iter_t = Iter::new(&mut iter);
                func(&iter_t, tuple);
                ecs_table_unlock(self.world.raw_world, iter.table);
            }
        }
    }

    /// iter iterator.
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut Iter)
    ///
    /// Iter iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::iter`
    #[doc(alias = "iterable::iter")]
    pub fn iter_only(&mut self, mut func: impl FnMut(&Iter)) {
        unsafe {
            let mut iter = ecs_query_iter(self.world.raw_world, self.query);
            while ecs_query_next(&mut iter) {
                let iter_t = Iter::new(&mut iter);
                func(&iter_t);
            }
        }
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
