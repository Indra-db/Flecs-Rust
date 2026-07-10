//! Class that extends the capabilities of a [`Query`] by providing additional operations on the query's iterator.
use core::ffi::c_void;

use crate::core::*;
use crate::sys;

pub struct QueryIter<'a, P, T>
where
    T: QueryTuple,
{
    iter: sys::ecs_iter_t,
    iter_next: ExternIterNextFn,
    _phantom: core::marker::PhantomData<&'a (P, T)>,
}

impl<P, T> QueryIter<'_, P, T>
where
    T: QueryTuple,
{
    pub fn new(iter: sys::ecs_iter_t, iter_next: ExternIterNextFn) -> Self {
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
    fn iter_next_func(&self) -> ExternIterNextFn {
        self.iter_next
    }
}

impl<'a, P, T> QueryAPI<'a, P, T> for QueryIter<'a, P, T>
where
    T: QueryTuple,
    Self: WorldProvider<'a>,
{
    fn entity(&self) -> EntityView<'_> {
        let world = unsafe { WorldRef::from_ptr(self.iter.real_world) };
        EntityView::new_from(world, unsafe {
            sys::ecs_get_entity(self.iter.query as *const c_void)
        })
    }
}

/// Formats the query as a string expression using `ecs_query_str`.
/// The resulting expression can be parsed to create the same query.
impl<P, T> core::fmt::Display for QueryIter<'_, P, T>
where
    T: QueryTuple,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", query_expr_string(IterOperations::query_ptr(self)))
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

impl<'a, P, T> QueryIter<'a, P, T>
where
    T: QueryTuple,
{
    /// Return a paged iterator.
    ///
    /// Paged iterators limit the results to those starting at `offset`, returning at most `limit` results.
    ///
    /// # Arguments
    ///
    /// * `offset` - Number of entities to skip.
    /// * `limit`  - Maximum number of entities to return.
    pub fn page(self, offset: i32, limit: i32) -> ChainedIter<'a, P, T> {
        ChainedIter::new_page(self, offset, limit)
    }

    /// Return a worker iterator.
    ///
    /// Worker iterators evenly distribute matched entities across N workers.
    /// Useful for multi-threaded processing of query results.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index of this worker (0..count-1).
    /// * `count` - Total number of workers.
    pub fn worker(self, index: i32, count: i32) -> ChainedIter<'a, P, T> {
        ChainedIter::new_worker(self, index, count)
    }
}

/// An iterator that wraps a [`QueryIter`] with page or worker constraints.
///
/// Zero-allocation design: stores the parent `ecs_iter_t` inline (no heap).
///
/// The parent is wrapped in [`core::cell::UnsafeCell`] because `ecs_page_next`/`ecs_worker_next`
/// mutate the parent through the `chain_it` raw pointer stored in the child iter.
/// `UnsafeCell` is the correct primitive for this: it grants interior mutability
/// through a raw pointer while the struct is borrowed via `&self`.
///
/// Safety invariant: `retrieve_iter` must not be called while a previously returned
/// child iter is still alive (i.e. after `run()`/`each()` returns is fine).
/// The `QueryAPI` methods all call `retrieve_iter` exactly once per invocation,
/// so this invariant is upheld by construction.
pub struct ChainedIter<'a, P, T>
where
    T: QueryTuple,
{
    /// Parent iter — wrapped in `UnsafeCell` so C can mutate it through `chain_it`.
    /// Must not be moved after `retrieve_iter` is called (enforced by `&self` borrow).
    parent: core::cell::UnsafeCell<sys::ecs_iter_t>,
    param1: i32,
    param2: i32,
    make_iter: unsafe extern "C-unwind" fn(*const sys::ecs_iter_t, i32, i32) -> sys::ecs_iter_t,
    iter_next: ExternIterNextFn,
    _phantom: core::marker::PhantomData<&'a (P, T)>,
}

impl<'a, P, T> ChainedIter<'a, P, T>
where
    T: QueryTuple,
{
    fn new_page(parent: QueryIter<'a, P, T>, offset: i32, limit: i32) -> Self {
        Self {
            parent: core::cell::UnsafeCell::new(parent.iter),
            param1: offset,
            param2: limit,
            make_iter: sys::ecs_page_iter,
            iter_next: sys::ecs_page_next,
            _phantom: core::marker::PhantomData,
        }
    }

    fn new_worker(parent: QueryIter<'a, P, T>, index: i32, count: i32) -> Self {
        Self {
            parent: core::cell::UnsafeCell::new(parent.iter),
            param1: index,
            param2: count,
            make_iter: sys::ecs_worker_iter,
            iter_next: sys::ecs_worker_next,
            _phantom: core::marker::PhantomData,
        }
    }
}

#[doc(hidden)]
impl<P, T> IterOperations for ChainedIter<'_, P, T>
where
    T: QueryTuple,
{
    #[inline(always)]
    fn retrieve_iter(&self) -> sys::ecs_iter_t {
        // SAFETY: &self is borrowed for the entire run()/each() call, so self.parent
        // cannot move. The child iter stores chain_it = self.parent.get(), which
        // remains valid and exclusively owned (no other &mut exists) for the duration.
        unsafe { (self.make_iter)(self.parent.get() as *const _, self.param1, self.param2) }
    }

    fn retrieve_iter_stage<'w>(&self, _stage: impl WorldProvider<'w>) -> sys::ecs_iter_t {
        panic!("Cannot change stage on a chained (page/worker) iterator.");
    }

    #[inline(always)]
    fn iter_next(&self, iter: &mut sys::ecs_iter_t) -> bool {
        unsafe { (self.iter_next)(iter) }
    }

    #[inline(always)]
    fn query_ptr(&self) -> *const sys::ecs_query_t {
        unsafe { (*self.parent.get()).query }
    }

    #[inline(always)]
    fn iter_next_func(&self) -> ExternIterNextFn {
        self.iter_next
    }
}

impl<'a, P, T> QueryAPI<'a, P, T> for ChainedIter<'a, P, T>
where
    T: QueryTuple,
    Self: WorldProvider<'a>,
{
    fn entity(&self) -> EntityView<'_> {
        let world = unsafe { WorldRef::from_ptr((*self.parent.get()).real_world) };
        EntityView::new_from(world, unsafe {
            sys::ecs_get_entity((*self.parent.get()).query as *const core::ffi::c_void)
        })
    }
}

/// Formats the query as a string expression using `ecs_query_str`.
/// The resulting expression can be parsed to create the same query.
impl<P, T> core::fmt::Display for ChainedIter<'_, P, T>
where
    T: QueryTuple,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", query_expr_string(IterOperations::query_ptr(self)))
    }
}

impl<'a, P, T> WorldProvider<'a> for ChainedIter<'a, P, T>
where
    T: QueryTuple,
{
    fn world(&self) -> WorldRef<'a> {
        unsafe { WorldRef::from_ptr((*self.parent.get()).world) }
    }
}
