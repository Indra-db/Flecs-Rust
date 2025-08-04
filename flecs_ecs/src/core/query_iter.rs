//! Class that extends the capabilities of a [`Query`] by providing additional operations on the query's iterator.
use core::ffi::c_void;

use crate::core::*;
use crate::sys;

pub struct QueryIter<'a, P, T>
where
    T: QueryTuple,
{
    iter: sys::ecs_iter_t,
    iter_next: unsafe extern "C-unwind" fn(*mut sys::ecs_iter_t) -> bool,
    _phantom: core::marker::PhantomData<&'a (P, T)>,
}

impl<P, T> QueryIter<'_, P, T>
where
    T: QueryTuple,
{
    pub fn new(
        iter: sys::ecs_iter_t,
        iter_next: unsafe extern "C-unwind" fn(*mut sys::ecs_iter_t) -> bool,
    ) -> Self {
        Self {
            iter,
            iter_next,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Limit results to tables with specified group id (grouped queries only)
    ///
    /// # Arguments
    ///
    /// * `group_id`: the group id to set
    pub fn set_group(&mut self, group_id: impl IntoEntity) -> &mut Self {
        unsafe { sys::ecs_iter_set_group(&mut self.iter, *group_id.into_entity(self.world())) }
        self
    }

    /// set variable of iter
    ///
    /// # Arguments
    ///
    /// * `var_id`: the variable id to set
    ///
    /// * `value`: the value to set
    pub fn set_var(&mut self, var_id: i32, value: impl Into<Entity>) -> &mut Self {
        ecs_assert!(var_id != -1, FlecsErrorCode::InvalidParameter, 0);
        unsafe { sys::ecs_iter_set_var(&mut self.iter, var_id, *value.into()) };
        self
    }

    /// set variable of iter as table
    ///
    /// # Arguments
    ///
    /// * `var_id`: the variable id to set
    ///
    /// * `range`: the range to set
    pub fn set_var_table(&mut self, var_id: i32, table: impl IntoTableRange) -> &mut Self {
        ecs_assert!(var_id != -1, FlecsErrorCode::InvalidParameter, 0);
        unsafe { sys::ecs_iter_set_var_as_range(&mut self.iter, var_id, &table.range_raw()) };
        self
    }

    /// set variable for rule iter
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the variable to set
    /// * `value`: the value to set
    pub fn set_var_expr(&mut self, name: &str, value: impl Into<Entity>) -> &mut Self {
        let name = compact_str::format_compact!("{}\0", name);

        let query = self.iter.query;
        let var_id = unsafe { sys::ecs_query_find_var(query, name.as_ptr() as *const _) };
        ecs_assert!(
            var_id != -1,
            FlecsErrorCode::InvalidParameter,
            name.as_str()
        );
        unsafe { sys::ecs_iter_set_var(&mut self.iter, var_id, *value.into()) };
        self
    }

    /// set variable for rule iter as table
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the variable to set
    /// * `range`: the range to set
    pub fn set_var_table_expr(&mut self, name: &str, table: impl IntoTableRange) -> &mut Self {
        let name = compact_str::format_compact!("{}\0", name);

        let query = self.iter.query;
        let var_id = unsafe { sys::ecs_query_find_var(query, name.as_ptr() as *const _) };
        ecs_assert!(
            var_id != -1,
            FlecsErrorCode::InvalidParameter,
            name.as_str()
        );
        unsafe { sys::ecs_iter_set_var_as_range(&mut self.iter, var_id, &table.range_raw()) };
        self
    }
}

#[doc(hidden)]
impl<P, T> IterOperations for QueryIter<'_, P, T>
where
    T: QueryTuple,
{
    #[inline(always)]
    fn retrieve_iter(&self) -> sys::ecs_iter_t {
        self.iter
    }

    fn retrieve_iter_stage<'w>(&self, _stage: impl WorldProvider<'w>) -> sys::ecs_iter_t {
        panic!(
            "Cannot change the stage of an iterator that already exists. Use retrieve_iter_stage on the underlying query instead."
        );
    }

    #[inline(always)]
    fn iter_next(&self, iter: &mut sys::ecs_iter_t) -> bool {
        unsafe { (self.iter_next)(iter) }
    }

    #[inline(always)]
    fn query_ptr(&self) -> *const sys::ecs_query_t {
        self.iter.query
    }

    #[inline(always)]
    fn iter_next_func(&self) -> unsafe extern "C-unwind" fn(*mut sys::ecs_iter_t) -> bool {
        self.iter_next
    }
}

impl<'a, P, T> QueryAPI<'a, P, T> for QueryIter<'a, P, T>
where
    T: QueryTuple,
    Self: WorldProvider<'a>,
{
    fn entity(&self) -> EntityView {
        let world = unsafe { WorldRef::from_ptr(self.iter.real_world) };
        EntityView::new_from(world, unsafe {
            sys::ecs_get_entity(self.iter.query as *const c_void)
        })
    }
}

impl<'a, P, T> WorldProvider<'a> for QueryIter<'a, P, T>
where
    T: QueryTuple,
{
    fn world(&self) -> WorldRef<'a> {
        unsafe { WorldRef::from_ptr(self.iter.world) }
    }
}

// TODO : worker_iterable and page_iterable not implemented yet
