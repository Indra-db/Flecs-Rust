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

use super::{ImplementsClone, ImplementsDefault};

#[doc(hidden)]
pub mod private {
    use std::{ffi::c_void, ptr};

    use flecs_ecs_sys::{ecs_ctx_free_t, ecs_iter_t, ecs_table_lock, ecs_table_unlock};

    use crate::core::{ComponentPointers, Entity, Iter, IterT, Iterable, ObserverSystemBindingCtx};

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

        /// Callback of the each functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        unsafe extern "C" fn run_each<Func>(iter: *mut IterT)
        where
            Func: FnMut(T::TupleType<'_>),
        {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let each = (*ctx).each.unwrap();
            let each = &mut *(each as *mut Func);

            let mut components_data = T::create_ptrs(&*iter);
            let iter_count = {
                if (*iter).count == 0 {
                    1_usize
                } else {
                    (*iter).count as usize
                }
            };

            ecs_table_lock((*iter).world, (*iter).table);

            for i in 0..iter_count {
                let tuple = components_data.get_tuple(i);
                each(tuple);
            }

            ecs_table_unlock((*iter).world, (*iter).table);
        }

        /// Callback of the `each_entity` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        #[doc(alias = "iter_invoker::invoke_callback")]
        unsafe extern "C" fn run_each_entity<Func>(iter: *mut IterT)
        where
            Func: FnMut(&mut Entity, T::TupleType<'_>),
        {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let each_entity = (*ctx).each_entity.unwrap();
            let each_entity = &mut *(each_entity as *mut Func);

            let mut components_data = T::create_ptrs(&*iter);
            let iter_count = {
                if (*iter).count == 0 {
                    1_usize
                } else {
                    (*iter).count as usize
                }
            };

            ecs_table_lock((*iter).world, (*iter).table);

            for i in 0..iter_count {
                let mut entity =
                    Entity::new_from_existing_raw((*iter).world, *(*iter).entities.add(i));
                let tuple = components_data.get_tuple(i);

                each_entity(&mut entity, tuple);
            }
            ecs_table_unlock((*iter).world, (*iter).table);
        }

        /// Callback of the `each_iter` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        #[doc(alias = "iter_invoker::invoke_callback")]
        unsafe extern "C" fn run_each_iter<Func>(iter: *mut IterT)
        where
            Func: FnMut(&mut Iter, usize, T::TupleType<'_>),
        {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let each_iter = (*ctx).each_iter.unwrap();
            let each_iter = &mut *(each_iter as *mut Func);

            let mut components_data = T::create_ptrs(&*iter);
            let iter_count = {
                if (*iter).count == 0 {
                    1_usize
                } else {
                    (*iter).count as usize
                }
            };

            ecs_table_lock((*iter).world, (*iter).table);
            let mut iter_t = Iter::new(&mut (*iter));

            for i in 0..iter_count {
                let tuple = components_data.get_tuple(i);

                each_iter(&mut iter_t, i, tuple);
            }
            ecs_table_unlock((*iter).world, (*iter).table);
        }

        /// Callback of the `iter_only` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        #[doc(alias = "iter_invoker::invoke_callback")]
        unsafe extern "C" fn run_iter_only<Func>(iter: *mut IterT)
        where
            Func: FnMut(&mut Iter),
        {
            unsafe {
                let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
                let iter_only = (*ctx).iter_only.unwrap();
                let iter_only = &mut *(iter_only as *mut Func);
                let iter_count = {
                    if (*iter).count == 0 {
                        1_usize
                    } else {
                        (*iter).count as usize
                    }
                };

                ecs_table_lock((*iter).world, (*iter).table);

                for _ in 0..iter_count {
                    let mut iter_t = Iter::new(&mut *iter);
                    iter_only(&mut iter_t);
                }

                ecs_table_unlock((*iter).world, (*iter).table);
            }
        }

        /// Callback of the iter functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        #[doc(alias = "iter_invoker::invoke_callback")]
        unsafe extern "C" fn run_iter<Func>(iter: *mut IterT)
        where
            Func: FnMut(&mut Iter, T::TupleSliceType<'_>),
        {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let iter_func = (*ctx).iter.unwrap();
            let iter_func = &mut *(iter_func as *mut Func);

            let mut components_data = T::create_ptrs(&*iter);
            let iter_count = {
                if (*iter).count == 0 {
                    1_usize
                } else {
                    (*iter).count as usize
                }
            };

            ecs_table_lock((*iter).world, (*iter).table);

            let tuple = components_data.get_slice(iter_count);
            let mut iter_t = Iter::new(&mut *iter);
            iter_func(&mut iter_t, tuple);
            ecs_table_unlock((*iter).world, (*iter).table);
        }

        // free functions

        extern "C" fn on_free_each(ptr: *mut c_void) {
            let ptr_func: *mut fn(T::TupleType<'_>) = ptr as *mut fn(T::TupleType<'_>);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        extern "C" fn on_free_each_entity(ptr: *mut c_void) {
            let ptr_func: *mut fn(&mut Entity, T::TupleType<'_>) =
                ptr as *mut fn(&mut Entity, T::TupleType<'_>);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        extern "C" fn on_free_each_iter(ptr: *mut c_void) {
            let ptr_func: *mut fn(&mut Iter, usize, T::TupleType<'_>) =
                ptr as *mut fn(&mut Iter, usize, T::TupleType<'_>);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        extern "C" fn on_free_iter_only(ptr: *mut c_void) {
            let ptr_func: *mut fn(&Iter) = ptr as *mut fn(&Iter);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        extern "C" fn on_free_iter(ptr: *mut c_void) {
            let ptr_func: *mut fn(&Iter, T::TupleSliceType<'_>) =
                ptr as *mut fn(&Iter, T::TupleSliceType<'_>);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

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

pub trait DoesNotImpl {
    const IMPLS: bool = false;
}

impl<T> DoesNotImpl for T {}

impl<T: Clone> ImplementsClone<T> {
    pub const IMPLS: bool = true;
}

impl<T: Default> ImplementsDefault<T> {
    pub const IMPLS: bool = true;
}

pub trait FlecsConstantId {
    const ID: u64;
}
