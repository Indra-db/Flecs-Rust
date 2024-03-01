use super::{
    c_binding::bindings::{
        _ecs_abort, ecs_filter_copy, ecs_filter_desc_t, ecs_filter_fini, ecs_filter_init,
        ecs_filter_iter, ecs_filter_move, ecs_filter_next, ecs_filter_str, ecs_get_entity,
        ecs_os_api, ecs_table_lock, ecs_table_unlock,
    },
    c_types::FilterT,
    entity::Entity,
    iter::Iter,
    iterable::Iterable,
    term::{Term, TermType},
    utility::errors::FlecsErrorCode,
    world::World,
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

    fn iter_impl(&mut self, mut func: impl FnMut(&Iter, T::TupleSliceType), filter: *mut FilterT) {
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
                let iter_t = Iter::new(&mut iter);
                func(&iter_t, tuple);
                ecs_table_unlock(self.world.raw_world, iter.table);
            }
        }
    }

    pub fn iter_only_impl(&mut self, mut func: impl FnMut(&Iter), filter: *mut FilterT) {
        unsafe {
            let mut iter = ecs_filter_iter(self.world.raw_world, filter);
            while ecs_filter_next(&mut iter) {
                let iter_t = Iter::new(&mut iter);
                func(&iter_t);
            }
        }
    }

    fn entity_impl(&self, filter: *mut FilterT) -> Entity {
        Entity::new_from_existing_raw(self.world.raw_world, unsafe {
            ecs_get_entity(filter as *const _)
        })
    }

    fn each_term_impl(&self, mut func: impl FnMut(Term), filter: *mut FilterT) {
        unsafe {
            for i in 0..(*filter).term_count {
                let term = Term::new(
                    Some(&self.world),
                    TermType::Term(*(*filter).terms.add(i as usize)),
                );
                func(term);
            }
        }
    }

    fn get_term_impl(&self, index: usize, filter: *mut FilterT) -> Term {
        Term::new(
            Some(&self.world),
            TermType::Term(unsafe { *(*filter).terms.add(index) }),
        )
    }

    fn field_count_impl(&self, filter: *mut FilterT) -> i32 {
        unsafe { (*filter).field_count }
    }

    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    fn to_string_impl(&self, filter: *mut FilterT) -> String {
        let result: *mut c_char =
            unsafe { ecs_filter_str(self.world.raw_world, filter as *const _) };
        let rust_string =
            String::from(unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = ecs_os_api.free_ {
                free_func(result as *mut _)
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
    pub fn new_view(world: &World, filter: *const FilterT) -> Self {
        Self {
            base: FilterBase {
                world: world.clone(),
                _phantom: std::marker::PhantomData,
            },
            filter_ptr: filter as *mut FilterT,
        }
    }

    pub fn each(&mut self, func: impl FnMut(T::TupleType)) {
        self.base.each_impl(func, self.filter_ptr);
    }

    pub fn each_entity(&mut self, func: impl FnMut(&mut Entity, T::TupleType)) {
        self.base.each_entity_impl(func, self.filter_ptr);
    }

    pub fn entity(&self) -> Entity {
        self.base.entity_impl(self.filter_ptr)
    }

    pub fn each_term(&self, func: impl FnMut(Term)) {
        self.base.each_term_impl(func, self.filter_ptr);
    }

    pub fn get_term(&self, index: usize) -> Term {
        self.base.get_term_impl(index, self.filter_ptr)
    }

    pub fn field_count(&self) -> i32 {
        self.base.field_count_impl(self.filter_ptr)
    }

    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    pub fn to_string(&self) -> String {
        self.base.to_string_impl(self.filter_ptr)
    }
}

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
                _ecs_abort(
                    FlecsErrorCode::InvalidParameter.to_int(),
                    file!().as_ptr() as *const i8,
                    line!() as i32,
                    std::ptr::null(),
                );

                if let Some(abort_func) = ecs_os_api.abort_ {
                    abort_func()
                }
            }

            if !(*desc).terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func((*desc).terms_buffer as *mut _)
                }
            }
        }

        filter_obj
    }

    #[inline]
    pub fn each(&mut self, func: impl FnMut(T::TupleType)) {
        self.base.each_impl(func, &mut self.filter);
    }

    #[inline]
    pub fn each_entity(&mut self, func: impl FnMut(&mut Entity, T::TupleType)) {
        self.base.each_entity_impl(func, &mut self.filter);
    }

    #[inline]
    pub fn iter(&mut self, func: impl FnMut(&Iter, T::TupleSliceType)) {
        self.base.iter_impl(func, &mut self.filter);
    }

    #[inline]
    pub fn iter_only(&mut self, func: impl FnMut(&Iter)) {
        self.base.iter_only_impl(func, &mut self.filter);
    }

    pub fn entity(&mut self) -> Entity {
        self.base.entity_impl(&mut self.filter)
    }

    pub fn each_term(&mut self, func: impl FnMut(Term)) {
        self.base.each_term_impl(func, &mut self.filter)
    }

    pub fn get_term(&mut self, index: usize) -> Term {
        self.base.get_term_impl(index, &mut self.filter)
    }

    pub fn field_count(&mut self) -> i32 {
        self.base.field_count_impl(&mut self.filter)
    }

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
        self.filter.owned = false;
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
