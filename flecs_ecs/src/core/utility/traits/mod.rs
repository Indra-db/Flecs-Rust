mod inout_oper;
mod into_component_id;
mod into_entity;
mod into_table;
mod into_world;
mod iter;
mod reactor;

pub use inout_oper::*;
pub use into_component_id::*;
pub use into_entity::*;
pub use into_table::*;
pub use into_world::*;
pub use iter::*;
pub use reactor::*;

#[doc(hidden)]
pub mod private {
    use std::{ffi::c_void, ptr};

    use flecs_ecs_sys::{ecs_ctx_free_t, ecs_iter_t};

    use crate::core::{Iterable, ObserverSystemBindingCtx};

    #[allow(non_camel_case_types)]
    #[doc(hidden)]
    pub trait internal_ReactorAPI<'a, T>
    where
        T: Iterable,
    {
        fn set_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self;

        fn set_binding_context_free(&mut self, binding_ctx_free: ecs_ctx_free_t) -> &mut Self;

        fn desc_binding_context(&self) -> *mut c_void;

        fn set_desc_callback(&mut self, callback: Option<unsafe extern "C" fn(*mut ecs_iter_t)>);

        /// Get the binding context
        fn get_binding_context(&mut self) -> &mut ObserverSystemBindingCtx {
            let mut binding_ctx: *mut ObserverSystemBindingCtx =
                self.desc_binding_context() as *mut _;

            if binding_ctx.is_null() {
                let new_binding_ctx = Box::<ObserverSystemBindingCtx>::default();
                let static_ref = Box::leak(new_binding_ctx);
                binding_ctx = static_ref;
                self.set_binding_context(binding_ctx as *mut c_void);
                self.set_binding_context_free(Some(Self::binding_ctx_drop));
            }
            unsafe { &mut *binding_ctx }
        }

        /// drop the binding context
        extern "C" fn binding_ctx_drop(ptr: *mut c_void) {
            let ptr_struct: *mut ObserverSystemBindingCtx = ptr as *mut ObserverSystemBindingCtx;
            unsafe {
                ptr::drop_in_place(ptr_struct);
            }
        }
    }
}
