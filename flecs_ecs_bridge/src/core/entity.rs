use std::ffi::{c_void, CStr};

use crate::core::c_binding::bindings::ecs_get_world;

use super::{
    c_binding::bindings::{ecs_get_name, ecs_get_path_w_sep, ecs_get_symbol, ecs_is_alive},
    c_types::*,
    component::{CachedComponentData, ComponentType},
    id::Id,
};

#[derive(Default)]
pub struct Entity {
    pub id: Id,
}

impl Entity {
    pub fn new(world: *mut WorldT, id: EntityT) -> Self {
        unsafe {
            Self {
                id: Id::new(
                    if world.is_null() {
                        std::ptr::null_mut()
                    } else {
                        ecs_get_world(world as *mut c_void) as *mut WorldT
                    },
                    id,
                ),
            }
        }
    }

    pub const fn new_only_id(id: EntityT) -> Self {
        Self {
            id: Id::new_only_id(id),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id.world != std::ptr::null_mut() && unsafe { ecs_is_alive(self.id.world, self.id.id) }
    }

    pub fn get_name(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(ecs_get_name(self.id.world, self.id.id))
                .to_str()
                .unwrap_or("")
        }
    }

    pub fn get_symbol(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(ecs_get_symbol(self.id.world, self.id.id))
                .to_str()
                .unwrap_or("")
        }
    }

    pub fn get_hierachy_path(&self, sep: &str, init_sep: &str) -> String {
        self.get_hierachy_path_from_parent_id(0, sep, init_sep)
    }

    pub fn get_hierachy_path_from_parent_id(
        &self,
        parent: EntityT,
        sep: &str,
        init_sep: &str,
    ) -> String {
        //let path = unsafe {
        //    ecs_get_path_w_sep(
        //        self.id.world,
        //        parent,
        //        self.id.id,
        //        sep.as_ptr(),
        //        init_sep.as_ptr(),
        //    )
        //};
        ////TODO check if we can return &str
        //unsafe { CStr::from_ptr(path) }
        //    .to_str()
        //    .unwrap_or("")
        //    .to_string()
        String::new()
    }

    pub fn get_hierachy_path_from_parent_type<T: CachedComponentData>(
        &self,
        sep: &str,
        init_sep: &str,
    ) -> String {
        self.get_hierachy_path_from_parent_id(T::get_id(self.id.world), sep, init_sep)
    }
}
