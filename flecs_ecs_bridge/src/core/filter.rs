use super::c_types::*;
use super::component_registration::*;
use super::{
    c_binding::bindings::{
        _ecs_abort, ecs_filter_copy, ecs_filter_desc_t, ecs_filter_fini, ecs_filter_init,
        ecs_filter_iter, ecs_filter_move, ecs_filter_next, ecs_filter_str, ecs_get_entity,
        ecs_os_api, ecs_table_lock, ecs_table_unlock, ECS_FILTER_INIT,
    },
    c_types::{FilterT, TermT, WorldT},
    entity::Entity,
    iterable::Iterable,
    term::Term,
    utility::{errors::FlecsErrorCode, functions::ecs_field},
    world::World,
};
use flecs_ecs_bridge_derive::Component;
use std::ffi::c_char;
use std::ops::Deref;
use std::sync::OnceLock;

pub struct Filter<T>
where
    T: Iterable,
{
    pub world: *mut WorldT,
    pub filter_ptr: *mut FilterT,
    pub desc: ecs_filter_desc_t,
    next_term_index: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for Filter<T>
where
    T: Iterable,
{
    fn default() -> Self {
        Filter {
            world: std::ptr::null_mut(),
            filter_ptr: std::ptr::null_mut(),
            desc: Default::default(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T> Filter<T>
where
    T: Iterable,
{
    pub fn new(world: &World) -> Self {
        let mut desc = ecs_filter_desc_t::default();
        T::register_ids_descriptor(world.world, &mut desc);
        let raw_filter = unsafe { ecs_filter_init(world.world, &desc) };
        Filter {
            world: world.world,
            filter_ptr: raw_filter,
            desc,
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn new_w_filter(world: *mut WorldT, filter: *mut FilterT) -> Self {
        Filter {
            world,
            filter_ptr: filter,
            desc: Default::default(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn new_ownership(world: *mut WorldT, filter: *mut FilterT) -> Self {
        let filter_obj = Filter {
            world,
            filter_ptr: std::ptr::null_mut(),
            desc: Default::default(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };

        unsafe { ecs_filter_move(filter_obj.filter_ptr, filter as *mut FilterT) };

        filter_obj
    }

    //TODO: this is not production ready
    fn new_from_desc(world: *mut WorldT, desc: *mut ecs_filter_desc_t) -> Self {
        let filter_obj = Filter {
            world,
            filter_ptr: std::ptr::null_mut(),
            desc: Default::default(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };

        unsafe {
            (*desc).storage = filter_obj.filter_ptr;
        }

        unsafe {
            if ecs_filter_init(filter_obj.world, desc).is_null() {
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

    pub fn each(&mut self, mut func: impl FnMut(&mut Entity, T::TupleType)) {
        unsafe {
            let mut iter = ecs_filter_iter(self.world, self.filter_ptr);
            let func_ref = &mut func;
            while ecs_filter_next(&mut iter) {
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

    pub fn current_term(&mut self) -> &mut TermT {
        &mut self.desc.terms[self.next_term_index]
    }

    pub fn next_term(&mut self) {
        self.next_term_index += 1;
    }

    pub fn entity(&self) -> Entity {
        Entity::new_from_existing(self.world, unsafe {
            ecs_get_entity(self.filter_ptr as *const _)
        })
    }

    pub fn each_term(&self, mut func: impl FnMut(Term)) {
        unsafe {
            for i in 0..(*self.filter_ptr).term_count {
                let term = Term::new(self.world, *(*self.filter_ptr).terms.add(i as usize));
                func(term);
            }
        }
    }

    pub fn get_term(&self, index: usize) -> Term {
        Term::new(self.world, unsafe { *(*self.filter_ptr).terms.add(index) })
    }

    pub fn field_count(&self) -> i32 {
        unsafe { (*self.filter_ptr).field_count }
    }

    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    pub fn to_string(&self) -> String {
        let result: *mut c_char =
            unsafe { ecs_filter_str(self.world, self.filter_ptr as *const _) };
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

impl<T> Drop for Filter<T>
where
    T: Iterable,
{
    fn drop(&mut self) {
        if !self.filter_ptr.is_null() {
            unsafe { ecs_filter_fini(self.filter_ptr) }
        }
    }
}

impl<T> Clone for Filter<T>
where
    T: Iterable,
{
    fn clone(&self) -> Self {
        let mut new_filter = Filter::<T> {
            world: self.world,
            ..Default::default()
        };
        if !self.filter_ptr.is_null() {
            new_filter.filter_ptr = self.filter_ptr;
        } else {
            new_filter.filter_ptr = std::ptr::null_mut();
        }
        unsafe { ecs_filter_copy(new_filter.filter_ptr, self.filter_ptr) };
        new_filter
    }
}

pub trait Filterable: Sized {
    fn current_term(&mut self) -> &mut TermT;
    fn next_term(&mut self);
    fn get_world(&self) -> *mut WorldT;
}

impl<T> Filterable for Filter<T>
where
    T: Iterable,
{
    fn current_term(&mut self) -> &mut TermT {
        self.current_term()
    }

    fn next_term(&mut self) {
        self.next_term()
    }

    fn get_world(&self) -> *mut WorldT {
        self.world
    }
}
