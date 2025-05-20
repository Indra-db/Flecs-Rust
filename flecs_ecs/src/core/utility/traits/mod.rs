mod id_operations;
mod inout_oper;
mod into_component_id;
mod into_entity;
mod into_id;
mod into_table;
mod query_api;
mod system_api;
mod world_provider;

pub use id_operations::*;
pub use inout_oper::*;
pub use into_component_id::*;
pub use into_entity::*;
pub use into_id::*;
pub use into_table::*;
pub use query_api::*;
pub use system_api::*;
pub use world_provider::*;

use crate::core::{ImplementsClone, ImplementsDefault};

#[doc(hidden)]
pub mod private {
    use crate::core::*;
    use crate::sys;
    use core::ffi::c_void;

    #[cfg(feature = "std")]
    extern crate std;

    extern crate alloc;
    use alloc::boxed::Box;

    #[allow(non_camel_case_types)]
    #[doc(hidden)]
    pub trait internal_SystemAPI<'a, P, T>
    where
        T: QueryTuple,
        P: ComponentId,
    {
        fn set_callback_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self;

        fn set_callback_binding_context_free(
            &mut self,
            binding_ctx_free: sys::ecs_ctx_free_t,
        ) -> &mut Self;

        fn set_run_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self;

        fn set_run_binding_context_free(
            &mut self,
            run_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
        ) -> &mut Self;

        fn desc_binding_context(&self) -> *mut c_void;

        fn set_desc_callback(
            &mut self,
            callback: Option<unsafe extern "C-unwind" fn(*mut sys::ecs_iter_t)>,
        );

        fn set_desc_run(
            &mut self,
            callback: Option<unsafe extern "C-unwind" fn(*mut sys::ecs_iter_t)>,
        );

        /// Callback of the each functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        unsafe extern "C-unwind" fn execute_each<const CALLED_FROM_RUN: bool, Func>(
            iter: *mut sys::ecs_iter_t,
        ) where
            Func: FnMut(T::TupleType<'_>),
        {
            unsafe {
                const {
                    assert!(
                        !T::CONTAINS_ANY_TAG_TERM,
                        "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
                    );
                }

                let iter = &mut *iter;
                iter.flags |= sys::EcsIterCppEach;

                let each = &mut *(iter.callback_ctx as *mut Func);

                let mut components_data = T::create_ptrs(&*iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                if !CALLED_FROM_RUN {
                    sys::ecs_table_lock(iter.world, iter.table);
                }

                for i in 0..iter_count {
                    let tuple = components_data.get_tuple(&*iter, i);
                    each(tuple);
                }

                if !CALLED_FROM_RUN {
                    sys::ecs_table_unlock(iter.world, iter.table);
                }
            }
        }

        /// Callback of the `each_entity` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        unsafe extern "C-unwind" fn execute_each_entity<const CALLED_FROM_RUN: bool, Func>(
            iter: *mut sys::ecs_iter_t,
        ) where
            Func: FnMut(EntityView, T::TupleType<'_>),
        {
            unsafe {
                const {
                    assert!(
                        !T::CONTAINS_ANY_TAG_TERM,
                        "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
                    );
                }

                let iter = &mut *iter;
                iter.flags |= sys::EcsIterCppEach;

                let each_entity = &mut *(iter.callback_ctx as *mut Func);

                let mut components_data = T::create_ptrs(&*iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        // If query has no This terms, count can be 0. Since each does not
                        // have an entity parameter, just pass through components
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                ecs_assert!(
                    !iter.entities.is_null(),
                    FlecsErrorCode::InvalidOperation,
                    "System does not return entities ($this variable is not populated).\nSystem: {:?}",
                    WorldRef::from_ptr(iter.world).entity_from_id(iter.system)
                );

                if !CALLED_FROM_RUN {
                    sys::ecs_table_lock(iter.world, iter.table);
                }

                for i in 0..iter_count {
                    let world = WorldRef::from_ptr(iter.world);
                    let entity = EntityView::new_from(world, *iter.entities.add(i));
                    let tuple = components_data.get_tuple(&*iter, i);

                    each_entity(entity, tuple);
                }

                if !CALLED_FROM_RUN {
                    sys::ecs_table_unlock(iter.world, iter.table);
                }
            }
        }

        /// Callback of the `each_iter` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        unsafe extern "C-unwind" fn execute_each_iter<Func>(iter: *mut sys::ecs_iter_t)
        where
            Func: FnMut(TableIter<false, P>, usize, T::TupleType<'_>),
        {
            unsafe {
                const {
                    assert!(
                        !T::CONTAINS_ANY_TAG_TERM,
                        "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
                    );
                }

                let iter = &mut *iter;
                iter.flags |= sys::EcsIterCppEach;

                let each_iter = &mut *(iter.callback_ctx as *mut Func);
                let mut components_data = T::create_ptrs(&*iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                sys::ecs_table_lock(iter.world, iter.table);

                for i in 0..iter_count {
                    let tuple = components_data.get_tuple(&*iter, i);
                    let iter_t = TableIter::new(iter);

                    each_iter(iter_t, i, tuple);
                }
                sys::ecs_table_unlock(iter.world, iter.table);
            }
        }

        /// Callback of the `iter_only` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        unsafe extern "C-unwind" fn execute_run<Func>(iter: *mut sys::ecs_iter_t)
        where
            Func: FnMut(TableIter<true, P>),
        {
            unsafe {
                let iter = &mut *iter;
                let run = &mut *(iter.run_ctx as *mut Func);
                let mut iter_t = TableIter::new(&mut *iter);
                iter_t.iter_mut().flags &= !sys::EcsIterIsValid;
                run(iter_t);
                // ecs_assert!(
                //     iter.flags & sys::EcsIterIsValid == 0,
                //     FlecsErrorCode::InvalidOperation,
                //     "iterators must be manually finished with ecs_iter_fini"
                // );
            }
        }

        extern "C-unwind" fn free_callback<Func>(ptr: *mut c_void) {
            unsafe {
                drop(Box::from_raw(ptr as *mut Func));
            };
        }

        // /// Get the binding context
        // fn get_binding_context(&mut self, is_run: bool) -> &mut ReactorBindingType {
        //     let mut binding_ctx: *mut ReactorBindingType = self.desc_binding_context() as *mut _;

        //     if binding_ctx.is_null() {
        //         let new_binding_ctx = Box::<ReactorBindingType>::default();
        //         let static_ref = Box::leak(new_binding_ctx);
        //         binding_ctx = static_ref;
        //         if is_run {
        //             self.set_run_binding_context(binding_ctx as *mut c_void);
        //             self.set_run_binding_context_free(Some(Self::binding_ctx_drop));
        //         } else {
        //             self.set_callback_binding_context(binding_ctx as *mut c_void);
        //             self.set_callback_binding_context_free(Some(Self::binding_ctx_drop));
        //         }
        //     }
        //     unsafe { &mut *binding_ctx }
        // }

        // /// drop the binding context
        // extern "C-unwind" fn binding_ctx_drop(ptr: *mut c_void) {
        //     let ptr_struct: *mut ReactorBindingType = ptr as *mut ReactorBindingType;
        //     unsafe {
        //         ptr::drop_in_place(ptr_struct);
        //     }
        // }
    }

    #[allow(non_camel_case_types)]
    #[doc(hidden)]
    pub trait internal_ParSystemAPI<'a, P, T>: internal_SystemAPI<'a, P, T>
    where
        T: QueryTuple,
        P: ComponentId,
    {
        fn set_multi_threaded(&mut self, multi_threaded: bool);
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
