use std::ffi::c_char;

use super::{
    c_binding::bindings::{
        _ecs_abort, ecs_filter_copy, ecs_filter_desc_t, ecs_filter_fini, ecs_filter_init,
        ecs_filter_move, ecs_filter_str, ecs_get_entity, ecs_os_api, ECS_FILTER_INIT,
    },
    c_types::{FilterT, WorldT},
    entity::Entity,
    term::Term,
    utility::errors::FlecsErrorCode,
};

pub struct FilterBase {
    world: *mut WorldT,
    filter: FilterT,
    filter_ptr: *const FilterT,
}

impl Default for FilterBase {
    fn default() -> Self {
        FilterBase {
            world: std::ptr::null_mut(),
            filter: unsafe { ECS_FILTER_INIT },
            filter_ptr: std::ptr::null(),
        }
    }
}
impl FilterBase {
    pub fn new(world: *mut WorldT, filter: *const FilterT) -> Self {
        FilterBase {
            world,
            filter: unsafe { ECS_FILTER_INIT },
            filter_ptr: filter,
        }
    }

    pub fn new_ownership(world: *mut WorldT, filter: *mut FilterT) -> Self {
        let mut filter_obj = FilterBase {
            world,
            filter: unsafe { ECS_FILTER_INIT },
            filter_ptr: std::ptr::null(),
        };

        unsafe {
            ecs_filter_move(
                &filter_obj.filter as *const _ as *mut FilterT,
                filter as *mut FilterT,
            )
        };

        filter_obj.filter_ptr = &filter_obj.filter as *const _;
        filter_obj
    }

    pub fn new_from_desc(world: *mut WorldT, desc: *mut ecs_filter_desc_t) -> Self {
        let mut filter_obj = FilterBase {
            world,
            filter: unsafe { ECS_FILTER_INIT },
            filter_ptr: std::ptr::null(),
        };

        unsafe {
            (*desc).storage = &mut filter_obj.filter as *mut _ as *mut _;
        }

        unsafe {
            if ecs_filter_init(&mut filter_obj.filter as *const _ as *mut _, desc)
                == std::ptr::null_mut()
            {
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

        filter_obj.filter_ptr = &filter_obj.filter as *const _;
        filter_obj
    }

    pub fn entity(&self) -> Entity {
        Entity::new_from_existing(self.world, unsafe {
            ecs_get_entity(self.filter_ptr as *const _)
        })
    }

    pub fn each_term<F>(&self, mut func: F)
    where
        F: FnMut(Term),
    {
        unsafe {
            for i in 0..(*self.filter_ptr).term_count {
                let term = Term::new(self.world, *(*self.filter_ptr).terms.add(i as usize));
                func(term);
            }
        }
    }

    pub fn get_term(&self, index: i32) -> Term {
        Term::new(self.world, unsafe {
            *(*self.filter_ptr).terms.add(index as usize)
        })
    }

    pub fn field_count(&self) -> i32 {
        unsafe { (*self.filter_ptr).field_count }
    }

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

impl Drop for FilterBase {
    fn drop(&mut self) {
        if !self.filter_ptr.is_null() && self.filter_ptr != &self.filter as *const _ {
            unsafe { ecs_filter_fini(&mut self.filter as *const _ as *mut _) }
        }
    }
}

impl Clone for FilterBase {
    fn clone(&self) -> Self {
        let mut new_filter = FilterBase::default();
        new_filter.world = self.world;
        if !self.filter_ptr.is_null() {
            new_filter.filter_ptr = &self.filter as *const _;
        } else {
            new_filter.filter_ptr = std::ptr::null();
        }
        unsafe { ecs_filter_copy(&mut new_filter.filter, &self.filter) };
        new_filter
    }
}
