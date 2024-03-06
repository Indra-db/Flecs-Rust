//! Filters are cheaper to create, but slower to iterate than queries.

use super::{
    c_binding::{
        bindings::{
            ecs_filter_copy, ecs_filter_desc_t, ecs_filter_fini, ecs_filter_init, ecs_filter_iter,
            ecs_filter_move, ecs_filter_next, ecs_filter_str, ecs_get_entity, ecs_os_api,
            ecs_table_lock, ecs_table_unlock,
        },
        ecs_abort_,
    },
    c_types::FilterT,
    entity::Entity,
    iter::Iter,
    iterable::Iterable,
    term::{Term, TermType},
    world::World,
    FlecsErrorCode,
};

use std::ffi::c_char;

struct FilterBase<'a, T>
where
    T: Iterable<'a>,
{
    pub world: World,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> FilterBase<'a, T>
where
    T: Iterable<'a>,
{
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
    fn each_impl(&mut self, mut func: impl FnMut(T::TupleType), filter: *mut FilterT) {
        unsafe {
            let mut iter = ecs_filter_iter(self.world.raw_world, filter);

            while ecs_filter_next(&mut iter) {
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
    fn each_entity_impl(
        &mut self,
        mut func: impl FnMut(&mut Entity, T::TupleType),
        filter: *mut FilterT,
    ) {
        unsafe {
            let mut iter = ecs_filter_iter(self.world.raw_world, filter);

            while ecs_filter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(self.world.raw_world, iter.table);

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
    fn iter_impl(
        &mut self,
        mut func: impl FnMut(&mut Iter, T::TupleSliceType),
        filter: *mut FilterT,
    ) {
        unsafe {
            let mut iter = ecs_filter_iter(self.world.raw_world, filter);

            while ecs_filter_next(&mut iter) {
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
                let mut iter_t = Iter::new(&mut iter);
                func(&mut iter_t, tuple);
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
    pub fn iter_only_impl(&mut self, mut func: impl FnMut(&mut Iter), filter: *mut FilterT) {
        unsafe {
            let mut iter = ecs_filter_iter(self.world.raw_world, filter);
            while ecs_filter_next(&mut iter) {
                let mut iter_t = Iter::new(&mut iter);
                func(&mut iter_t);
            }
        }
    }

    /// Get the entity of the current filter
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to get the entity from
    ///
    /// # Returns
    ///
    /// The entity of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::entity`
    #[doc(alias = "filter_base::entity")]
    fn entity_impl(&self, filter: *mut FilterT) -> Entity {
        Entity::new_from_existing_raw(self.world.raw_world, unsafe {
            ecs_get_entity(filter as *const _)
        })
    }

    /// Each term iterator.
    /// The "`each_term`" iterator accepts a function that is invoked for each term
    /// in the filter. The following function signature is valid:
    ///  - func(term: &mut Term)
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::term`
    #[doc(alias = "filter_base::term")]
    fn each_term_impl(&self, mut func: impl FnMut(&mut Term), filter: *mut FilterT) {
        unsafe {
            for i in 0..(*filter).term_count {
                let mut term = Term::new(
                    Some(&self.world),
                    TermType::Term(*(*filter).terms.add(i as usize)),
                );
                func(&mut term);
                term.reset(); // prevent freeing resources
            }
        }
    }

    /// Get the term of the current filter at the given index
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the term to get
    /// * `filter`: the filter to get the term from
    ///
    /// # Returns
    ///
    /// The term requested
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::term`
    #[doc(alias = "filter_base::term")]
    fn get_term_impl(&self, index: usize, filter: *mut FilterT) -> Term {
        Term::new(
            Some(&self.world),
            TermType::Term(unsafe { *(*filter).terms.add(index) }),
        )
    }

    /// Get the field count of the current filter
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to get the field count from
    ///
    /// # Returns
    ///
    /// The field count of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::field_count`
    #[doc(alias = "filter_base::field_count")]
    fn field_count_impl(&self, filter: *mut FilterT) -> i8 {
        unsafe { (*filter).field_count }
    }

    /// Convert filter to string expression. Convert filter terms to a string expression.
    /// The resulting expression can be parsed to create the same filter.
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to convert to a string
    ///
    /// # Returns
    ///
    /// The string representation of the filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::str`
    #[doc(alias = "filter_base::str")]
    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    fn to_string_impl(&self, filter: *mut FilterT) -> String {
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
}

pub struct FilterView<'a, T>
where
    T: Iterable<'a>,
{
    base: FilterBase<'a, T>,
    filter_ptr: *mut FilterT,
}

impl<'a, T> Clone for FilterView<'a, T>
where
    T: Iterable<'a>,
{
    fn clone(&self) -> Self {
        Self {
            base: FilterBase {
                world: self.base.world.clone(),
                _phantom: std::marker::PhantomData,
            },
            filter_ptr: self.filter_ptr,
        }
    }
}

impl<'a, T> FilterView<'a, T>
where
    T: Iterable<'a>,
{
    /// Create a new filter view
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter view from
    /// * `filter`: the filter to create the view from
    ///
    /// # See also
    ///
    /// * C++ API: `filter_view::filter_view`
    #[doc(alias = "filter_view::filter_view")]
    pub fn new(world: &World, filter: *const FilterT) -> Self {
        Self {
            base: FilterBase {
                world: world.clone(),
                _phantom: std::marker::PhantomData,
            },
            filter_ptr: filter as *mut FilterT,
        }
    }

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
    pub fn each(&mut self, func: impl FnMut(T::TupleType)) {
        self.base.each_impl(func, self.filter_ptr);
    }

    /// Get the entity of the current filter
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to get the entity from
    ///
    /// # Returns
    ///
    /// The entity of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::entity`
    #[doc(alias = "filter_base::entity")]
    pub fn each_entity(&mut self, func: impl FnMut(&mut Entity, T::TupleType)) {
        self.base.each_entity_impl(func, self.filter_ptr);
    }

    /// Get the entity of the current filter
    ///
    /// # Returns
    ///
    /// The entity of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::entity`
    #[doc(alias = "filter_base::entity")]
    pub fn entity(&self) -> Entity {
        self.base.entity_impl(self.filter_ptr)
    }

    /// Each term iterator.
    /// The "`each_term`" iterator accepts a function that is invoked for each term
    /// in the filter. The following function signature is valid:
    ///  - func(term: &mut Term)
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::term`
    #[doc(alias = "filter_base::term")]
    pub fn each_term(&self, func: impl FnMut(&mut Term)) {
        self.base.each_term_impl(func, self.filter_ptr);
    }

    /// Get the term of the current filter at the given index
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the term to get
    ///
    /// # Returns
    ///
    /// The term requested
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::term`
    #[doc(alias = "filter_base::term")]
    pub fn get_term(&self, index: usize) -> Term {
        self.base.get_term_impl(index, self.filter_ptr)
    }

    /// Get the field count of the current filter
    ///
    /// # Returns
    ///
    /// The field count of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::field_count`
    #[doc(alias = "filter_base::field_count")]
    pub fn field_count(&self) -> i8 {
        self.base.field_count_impl(self.filter_ptr)
    }

    /// Convert filter to string expression. Convert filter terms to a string expression.
    /// The resulting expression can be parsed to create the same filter.
    ///
    /// # Returns
    ///
    /// The string representation of the filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::str`
    #[doc(alias = "filter_base::str")]
    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    pub fn to_string(&self) -> String {
        self.base.to_string_impl(self.filter_ptr)
    }
}

/// Filters are cheaper to create, but slower to iterate than queries.
pub struct Filter<'a, T>
where
    T: Iterable<'a>,
{
    base: FilterBase<'a, T>,
    filter: FilterT,
}

impl<'a, T> Filter<'a, T>
where
    T: Iterable<'a>,
{
    /// Create a new filter
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter from
    ///
    /// # See also
    ///
    /// * C++ API: `filter::filter`
    #[doc(alias = "filter::filter")]
    pub fn new(world: &World) -> Self {
        let mut desc = ecs_filter_desc_t::default();
        T::register_ids_descriptor(world.raw_world, &mut desc);
        let mut filter: FilterT = Default::default();
        desc.storage = &mut filter;
        unsafe { ecs_filter_init(world.raw_world, &desc) };
        Filter {
            base: FilterBase {
                world: world.clone(),
                _phantom: std::marker::PhantomData,
            },
            filter,
        }
    }

    /// Wrap an existing raw filter
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter from
    /// * `filter`: the filter to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `filter::filter`
    #[doc(alias = "filter::filter")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_ownership(world: &World, filter: *mut FilterT) -> Self {
        let mut filter_obj = Filter {
            base: FilterBase {
                world: world.clone(),
                _phantom: std::marker::PhantomData,
            },
            filter: Default::default(),
        };

        unsafe { ecs_filter_move(&mut filter_obj.filter, filter) };

        filter_obj
    }

    //TODO: this needs testing -> desc.storage pointer becomes invalid after this call as it re-allocates after this new
    // determine if this is a problem
    /// Create a new filter from a filter descriptor
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter from
    /// * `desc`: the filter descriptor to create the filter from
    ///
    /// # See also
    ///
    /// * C++ API: `filter::filter`
    #[doc(alias = "filter::filter")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_from_desc(world: &World, desc: *mut ecs_filter_desc_t) -> Self {
        let mut filter_obj = Filter {
            base: FilterBase {
                world: world.clone(),
                _phantom: std::marker::PhantomData,
            },
            filter: Default::default(),
        };

        unsafe {
            (*desc).storage = &mut filter_obj.filter;
        }

        unsafe {
            if ecs_filter_init(filter_obj.base.world.raw_world, desc).is_null() {
                ecs_abort_(
                    FlecsErrorCode::InvalidParameter.to_int(),
                    file!().as_ptr() as *const i8,
                    line!() as i32,
                    std::ptr::null(),
                );

                if let Some(abort_func) = ecs_os_api.abort_ {
                    abort_func();
                }
            }

            if !(*desc).terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func((*desc).terms_buffer as *mut _);
                }
            }
        }

        filter_obj
    }

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
    #[inline]
    pub fn each(&mut self, func: impl FnMut(T::TupleType)) {
        self.base.each_impl(func, &mut self.filter);
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
    #[inline]
    pub fn each_entity(&mut self, func: impl FnMut(&mut Entity, T::TupleType)) {
        self.base.each_entity_impl(func, &mut self.filter);
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
    /// * C++ API: `iterable::iter`
    #[doc(alias = "iterable::iter")]
    #[inline]
    pub fn iter(&mut self, func: impl FnMut(&mut Iter, T::TupleSliceType)) {
        self.base.iter_impl(func, &mut self.filter);
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
    #[inline]
    pub fn iter_only(&mut self, func: impl FnMut(&mut Iter)) {
        self.base.iter_only_impl(func, &mut self.filter);
    }

    /// Get the entity of the current filter
    ///
    /// # Returns
    ///
    /// The entity of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::entity`
    #[doc(alias = "filter_base::entity")]
    pub fn entity(&mut self) -> Entity {
        self.base.entity_impl(&mut self.filter)
    }

    /// Each term iterator.
    /// The "`each_term`" iterator accepts a function that is invoked for each term
    /// in the filter. The following function signature is valid:
    ///  - func(term: &mut Term)
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::each_term`
    #[doc(alias = "filter_base::each_term")]
    pub fn each_term(&mut self, func: impl FnMut(&mut Term)) {
        self.base.each_term_impl(func, &mut self.filter);
    }

    /// Get the term of the current filter at the given index
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the term to get
    ///
    /// # Returns
    ///
    /// The term requested
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::term`
    #[doc(alias = "filter_base::term")]
    pub fn get_term(&mut self, index: usize) -> Term {
        self.base.get_term_impl(index, &mut self.filter)
    }

    /// Get the field count of the current filter
    ///
    /// # Returns
    ///
    /// The field count of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::field_count`
    #[doc(alias = "filter_base::field_count")]
    pub fn field_count(&mut self) -> i8 {
        self.base.field_count_impl(&mut self.filter)
    }

    /// Convert filter to string expression. Convert filter terms to a string expression.
    /// The resulting expression can be parsed to create the same filter.
    ///
    /// # Returns
    ///
    /// The string representation of the filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::str`
    #[doc(alias = "filter_base::str")]
    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    pub fn to_string(&mut self) -> String {
        self.base.to_string_impl(&mut self.filter)
    }
}

impl<'a, T> Drop for Filter<'a, T>
where
    T: Iterable<'a>,
{
    fn drop(&mut self) {
        // this is a hack to prevent ecs_filter_fini from freeing the memory of our stack allocated filter
        // we do actually own this filter. ecs_filter_fini is called to free the memory of the terms
        //self.filter.owned = false;
        //TODO the above code, `.owned` got removed in upgrading flecs from 3.2.4 to 3.2.11,
        // so we need to find a new? way to prevent the memory from being freed if it's stack allocated
        unsafe { ecs_filter_fini(&mut self.filter) }
    }
}

impl<'a, T> Clone for Filter<'a, T>
where
    T: Iterable<'a>,
{
    fn clone(&self) -> Self {
        let mut new_filter = Filter::<'a, T> {
            base: FilterBase {
                world: self.base.world.clone(),
                _phantom: std::marker::PhantomData,
            },
            filter: Default::default(),
        };

        unsafe { ecs_filter_copy(&mut new_filter.filter, &self.filter) };
        new_filter
    }
}
