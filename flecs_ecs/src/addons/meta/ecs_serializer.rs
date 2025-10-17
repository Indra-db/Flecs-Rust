use crate::core::*;
use crate::sys;
use core::ffi::c_void;

pub trait EcsSerializer {
    fn value_id(&self, type_id: impl Into<Entity>, value: *const c_void) -> i32;
    fn value<T: ComponentId>(&self, value: &T) -> i32;
    fn member(&self, name: &str) -> i32;
}

impl EcsSerializer for sys::ecs_serializer_t {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn value_id(&self, type_id: impl IntoEntity, value: *const c_void) -> i32 {
        if let Some(value_func) = self.value {
            unsafe {
                value_func(
                    self,
                    *type_id.into_entity(WorldRef::from_ptr(self.world as *mut _)),
                    value,
                )
            }
        } else {
            0
        }
    }

    fn value<T: ComponentId>(&self, value: &T) -> i32 {
        self.value_id(
            T::get_id(unsafe { WorldRef::from_ptr(self.world as *mut _) }),
            value as *const T as *const c_void,
        )
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
