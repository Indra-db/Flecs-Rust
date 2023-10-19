use libc::{memcpy, memset};

use crate::{
    core::{
        c_binding::bindings::{ecs_term_is_initialized, ecs_term_t, FLECS_TERM_DESC_MAX},
        iterable::ArrayElement,
    },
    ecs_assert,
};

use super::{
    c_binding::bindings::{
        _ecs_abort, ecs_filter_copy, ecs_filter_desc_t, ecs_filter_fini, ecs_filter_init,
        ecs_filter_iter, ecs_filter_move, ecs_filter_next, ecs_filter_str, ecs_flags32_t,
        ecs_get_entity, ecs_iter_t, ecs_os_api, ecs_table_lock, ecs_table_unlock,
    },
    c_types::{FilterT, IdT, TermT, WorldT},
    component_registration::{CachedComponentData, ComponentType, Enum},
    entity::Entity,
    enum_type::CachedEnumData,
    iterable::{Filterable, Iterable},
    term::{Term, TermBuilder},
    utility::{errors::FlecsErrorCode, functions::type_to_inout, traits::InOutType},
    world::World,
};

use std::{ffi::c_char, os::raw::c_void};

struct FilterBase<'a, T>
where
    T: Iterable<'a>,
{
    pub world: *mut WorldT,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Default for FilterBase<'a, T>
where
    T: Iterable<'a>,
{
    fn default() -> Self {
        Self {
            world: std::ptr::null_mut(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T> FilterBase<'a, T>
where
    T: Iterable<'a>,
{
    fn each_impl(&mut self, mut func: impl FnMut(T::TupleType), filter: *mut FilterT) {
        unsafe {
            let mut iter = ecs_filter_iter(self.world, filter);

            while ecs_filter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(self.world, iter.table);

                for i in 0..iter_count {
                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };

                    func(tuple);
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
            let mut iter = ecs_filter_iter(self.world, filter);

            while ecs_filter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(self.world, iter.table);

                for i in 0..iter_count {
                    let mut entity = Entity::new_from_existing(self.world, *iter.entities.add(i));

                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };

                    func(&mut entity, tuple);
                }

                ecs_table_unlock(self.world, iter.table);
            }
        }
    }

    fn entity_impl(&self, filter: *mut FilterT) -> Entity {
        Entity::new_from_existing(self.world, unsafe { ecs_get_entity(filter as *const _) })
    }

    fn each_term_impl(&self, mut func: impl FnMut(Term), filter: *mut FilterT) {
        unsafe {
            for i in 0..(*filter).term_count {
                let term = Term::new(self.world, *(*filter).terms.add(i as usize));
                func(term);
            }
        }
    }

    fn get_term_impl(&self, index: usize, filter: *mut FilterT) -> Term {
        Term::new(self.world, unsafe { *(*filter).terms.add(index) })
    }

    fn field_count_impl(&self, filter: *mut FilterT) -> i32 {
        unsafe { (*filter).field_count }
    }

    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    fn to_string_impl(&self, filter: *mut FilterT) -> String {
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
}

pub struct FilterView<'a, T>
where
    T: Iterable<'a>,
{
    base: FilterBase<'a, T>,
    filter_ptr: *mut FilterT,
}

impl<'a, T> Default for FilterView<'a, T>
where
    T: Iterable<'a>,
{
    fn default() -> Self {
        Self {
            base: Default::default(),
            filter_ptr: std::ptr::null_mut(),
        }
    }
}

impl<'a, T> Clone for FilterView<'a, T>
where
    T: Iterable<'a>,
{
    fn clone(&self) -> Self {
        Self {
            base: FilterBase {
                world: self.base.world,
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
    pub fn new_view(world: *mut WorldT, filter: *const FilterT) -> Self {
        Self {
            base: FilterBase {
                world,
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

#[derive(Default)]
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
        T::register_ids_descriptor(world.world, &mut desc);
        let mut filter: FilterT = Default::default();
        desc.storage = &mut filter;
        unsafe { ecs_filter_init(world.world, &desc) };
        Filter {
            base: FilterBase {
                world: world.world,
                _phantom: std::marker::PhantomData,
            },
            filter,
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_ownership(world: *mut WorldT, filter: *mut FilterT) -> Self {
        let mut filter_obj = Filter {
            base: FilterBase {
                world,
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
    pub fn new_from_desc(world: *mut WorldT, desc: *mut ecs_filter_desc_t) -> Self {
        let mut filter_obj = Filter {
            base: FilterBase {
                world,
                _phantom: std::marker::PhantomData,
            },
            filter: Default::default(),
        };

        unsafe {
            (*desc).storage = &mut filter_obj.filter;
        }

        unsafe {
            if ecs_filter_init(filter_obj.base.world, desc).is_null() {
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

    pub fn each(&mut self, func: impl FnMut(T::TupleType)) {
        self.base.each_impl(func, &mut self.filter);
    }

    #[inline]
    pub fn each_entity(&mut self, func: impl FnMut(&mut Entity, T::TupleType)) {
        self.base.each_entity_impl(func, &mut self.filter);
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
                world: self.base.world,
                _phantom: std::marker::PhantomData,
            },
            filter: Default::default(),
        };

        unsafe { ecs_filter_copy(&mut new_filter.filter, &self.filter) };
        new_filter
    }
}
