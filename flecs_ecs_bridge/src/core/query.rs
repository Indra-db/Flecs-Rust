use std::ops::{Deref, DerefMut};
use std::os::raw::{c_char, c_void};

use crate::ecs_assert;

use super::c_binding::bindings::{
    _ecs_abort, ecs_filter_iter, ecs_filter_next, ecs_filter_str, ecs_filter_t, ecs_get_entity,
    ecs_os_api, ecs_query_changed, ecs_query_desc_t, ecs_query_fini, ecs_query_get_filter,
    ecs_query_get_group_info, ecs_query_init, ecs_query_iter, ecs_query_next, ecs_query_orphaned,
    ecs_table_lock, ecs_table_unlock,
};
use super::c_types::*;
use super::entity::Entity;
use super::filter::FilterView;
use super::iterable::Iterable;
use super::term::Term;
use super::utility::errors::FlecsErrorCode;
use super::world::World;

pub struct QueryBase<'a, T>
where
    T: Iterable<'a>,
{
    world: *mut WorldT,
    query: *mut QueryT,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Default for QueryBase<'a, T>
where
    T: Iterable<'a>,
{
    fn default() -> Self {
        Self {
            world: std::ptr::null_mut(),
            query: std::ptr::null_mut(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T> QueryBase<'a, T>
where
    T: Iterable<'a>,
{
    fn new(world: *mut WorldT, query: *mut QueryT) -> Self {
        Self {
            world,
            query,
            _phantom: std::marker::PhantomData,
        }
    }

    fn new_with_desc(world: *mut WorldT, desc: *mut ecs_query_desc_t) -> Self {
        let query = unsafe { ecs_query_init(world, desc) };
        let obj = Self {
            world,
            query,
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
                    abort_func()
                };
            }

            if !(*desc).filter.terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func((*desc).filter.terms_buffer as *mut _)
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
    /// # C++ API Equivalent
    ///
    /// `query_base::changed`
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
    /// # C++ API Equivalent
    ///
    /// `query_base::orphaned`
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
    /// # C++ API Equivalent
    ///
    /// `query_base::get_group_info`
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
    /// # C++ API Equivalent
    ///
    /// `query_base::group_ctx`

    pub fn get_group_context(&mut self, group_id: u64) -> *mut c_void {
        let group_info = self.get_group_info(group_id);

        if !group_info.is_null() {
            unsafe { (*group_info).ctx }
        } else {
            std::ptr::null_mut()
        }
    }

    /// Free the query
    pub fn destruct(&mut self) {
        unsafe { ecs_query_fini(self.query) }
        self.world = std::ptr::null_mut();
        self.query = std::ptr::null_mut();
    }

    fn each_term(&self, mut func: impl FnMut(Term), query: *mut QueryT) {
        unsafe {
            let filter = ecs_query_get_filter(query);
            for i in 0..(*filter).term_count {
                let term = Term::new(self.world, *(*filter).terms.add(i as usize));
                func(term);
            }
        }
    }

    pub fn filter(&self) -> FilterView<'a, T> {
        FilterView::<T>::new_view(self.world, unsafe { ecs_query_get_filter(self.query) })
    }
    fn term(&self, index: i32) -> Term {
        let filter: *const ecs_filter_t = unsafe { ecs_query_get_filter(self.query) };
        ecs_assert!(
            !filter.is_null(),
            FlecsErrorCode::InvalidParameter,
            "query filter is null"
        );
        Term::new(self.world, unsafe { *(*filter).terms.add(index as usize) })
    }

    fn field_count(&self) -> i32 {
        unsafe { (*ecs_query_get_filter(self.query)).term_count }
    }

    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    fn to_string(&self) -> String {
        let filter = unsafe { ecs_query_get_filter(self.query) };
        let result: *mut c_char = unsafe { ecs_filter_str(self.world, filter as *const _) };
        let rust_string =
            String::from(unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = ecs_os_api.free_ {
                free_func(result as *mut _)
            }
        }
        rust_string
    }

    pub fn entity(&self) -> Entity {
        Entity::new_from_existing(self.world, unsafe {
            ecs_get_entity(self.query as *const c_void)
        })
    }
}

#[derive(Default)]
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
    pub fn new(world: &World) -> Self {
        let mut desc = ecs_query_desc_t::default();
        T::register_ids_descriptor(world.world, &mut desc.filter);
        let mut filter: FilterT = Default::default();
        desc.filter.storage = &mut filter;
        let query = unsafe { ecs_query_init(world.world, &desc) };
        Self {
            base: QueryBase::new(world.world, query),
        }
    }

    pub fn new_ownership(world: *mut WorldT, query: *mut QueryT) -> Self {
        Self {
            base: QueryBase::new(world, query),
        }
    }

    pub fn new_with_desc(world: *mut WorldT, desc: *mut ecs_query_desc_t) -> Self {
        Self {
            base: QueryBase::new_with_desc(world, desc),
        }
    }

    fn get_iter(&mut self, world: *mut WorldT) -> IterT {
        if !world.is_null() {
            self.world = world;
        }
        unsafe { ecs_query_iter(self.world, self.query) }
    }

    #[inline]
    pub fn each_entity(&mut self, func: impl FnMut(&mut Entity, T::TupleType)) {}

    pub fn each(&mut self, mut func: impl FnMut(T::TupleType)) {
        unsafe {
            let mut iter = ecs_query_iter(self.world, self.query);
            let func_ref = &mut func;
            while ecs_query_next(&mut iter) {
                let iter_count = iter.count as usize;
                let array_ptr = T::get_array_ptrs_of_components(&iter);
                ecs_table_lock(self.world, iter.table);
                for i in 0..iter_count {
                    let tuple = T::get_tuple(&array_ptr, i);
                    func_ref(tuple);
                }
                ecs_table_unlock(self.world, iter.table);
            }
        }
    }

    fn each_entity_impl(
        &mut self,
        mut func: impl FnMut(&mut Entity, T::TupleType),
        filter: *mut FilterT,
    ) {
        unsafe {
            let mut iter = ecs_query_iter(self.world, self.query);
            let func_ref = &mut func;
            while ecs_query_next(&mut iter) {
                let iter_count = iter.count as usize;
                let array_ptr = T::get_array_ptrs_of_components(&iter);
                ecs_table_lock(self.world, iter.table);
                for i in 0..iter_count {
                    let mut entity = Entity::new_from_existing(self.world, *iter.entities.add(i));
                    let tuple = T::get_tuple(&array_ptr, i);
                    func_ref(&mut entity, tuple);
                }
                ecs_table_unlock(self.world, iter.table);
            }
        }
    }
}

impl<'a, T> Drop for Query<'a, T>
where
    T: Iterable<'a>,
{
    fn drop(&mut self) {
        self.destruct();
    }
}
