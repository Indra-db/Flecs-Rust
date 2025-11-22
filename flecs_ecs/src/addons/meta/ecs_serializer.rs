use crate::core::*;
use crate::sys;
use core::ffi::c_void;

pub trait EcsSerializer {
    unsafe fn value_id<T: IntoEntity>(&self, type_id: T, value: *const T::CastType) -> i32;
    fn value<T: ComponentId>(&self, value: &T) -> i32;
    fn member(&self, name: &str) -> i32;
}

impl EcsSerializer for sys::ecs_serializer_t {
    unsafe fn value_id<T: IntoEntity>(&self, type_id: T, value: *const T::CastType) -> i32 {
        if let Some(value_func) = self.value {
            unsafe {
                value_func(
                    self,
                    *type_id.into_entity(WorldRef::from_ptr(self.world as *mut _)),
                    value as *const core::ffi::c_void,
                )
            }
        } else {
            0
        }
    }

    fn value<T: ComponentId>(&self, value: &T) -> i32 {
        unsafe {
            self.value_id(
                T::get_id(WorldRef::from_ptr(self.world as *mut _)),
                value as *const T as *const c_void,
            )
        }
    }

    fn member(&self, name: &str) -> i32 {
        let name = compact_str::format_compact!("{}\0", name);
        if let Some(member_func) = self.member {
            unsafe { member_func(self, name.as_ptr() as *const _) }
        } else {
            0
        }
    }
}
