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

use crate::core::{
    ComponentId, ComponentPointers, EntityView, ImplementsClone, ImplementsDefault,
    ImplementsPartialEq, ImplementsPartialOrd, QueryTuple, TableIter, ecs_assert,
};
#[cfg(feature = "flecs_safety_locks")]
use crate::core::{DECREMENT, INCREMENT, do_read_write_locks};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::{FlecsErrorCode, table_lock, table_unlock};
use crate::sys;

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
                let iter = &mut *iter;
                let each = &mut *(iter.callback_ctx as *mut Func);
                internal_each_iter_next::<T, CALLED_FROM_RUN>(iter, each);
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
                let iter = &mut *iter;
                let world = WorldRef::from_ptr(iter.world);
                let each_entity = &mut *(iter.callback_ctx as *mut Func);
                internal_each_entity_iter_next::<T, CALLED_FROM_RUN>(iter, &world, each_entity);
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
                internal_run::<P>(iter, run);
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

impl<T: PartialEq> ImplementsPartialEq<T> {
    pub const IMPLS: bool = true;
}

impl<T: PartialOrd> ImplementsPartialOrd<T> {
    pub const IMPLS: bool = true;
}

pub trait FlecsConstantId {
    const ID: u64;
}
/// A little “extractor” trait: given the iterator and an index,
/// produce whatever you want to pass as the first argument to the user‑closure.
pub(crate) trait EntityExtractor {
    /// The type of the “extra” first parameter to the callback.
    type Output;

    /// Called inside the per‑element loop to build that argument.
    unsafe fn extract(&self, iter: &sys::ecs_iter_t, idx: usize) -> Self::Output;
}

/// When you don’t want to pass an entity, use this ZST.
pub(crate) struct NoEntity;
impl EntityExtractor for NoEntity {
    type Output = ();
    #[inline(always)]
    unsafe fn extract(&self, _iter: &sys::ecs_iter_t, _idx: usize) {}
}

/// When you do want an `EntityView`, carry your `&WorldRef` here.
pub(crate) struct WithEntity<'w>(pub &'w WorldRef<'w>);
impl<'w> EntityExtractor for WithEntity<'w> {
    type Output = EntityView<'w>;
    #[inline(always)]
    unsafe fn extract(&self, iter: &sys::ecs_iter_t, idx: usize) -> EntityView<'w> {
        // SAFETY: same as your original
        let raw = unsafe { *iter.entities.add(idx) };
        EntityView::new_from_raw(self.0, raw)
    }
}

#[inline(always)]
pub(crate) fn internal_run<P: ComponentId>(
    iter: &mut sys::ecs_iter_t,
    func: &mut impl FnMut(TableIter<true, P>),
) {
    iter.flags &= !sys::EcsIterIsValid;
    let iter_t = unsafe { TableIter::new(iter) };
    func(iter_t);
}

#[inline(always)]
pub(crate) fn internal_each_generic<
    T: QueryTuple,
    E: EntityExtractor,
    const CALLED_FROM_RUN: bool,
    F: FnMut(E::Output, T::TupleType<'_>),
>(
    iter: &mut sys::ecs_iter_t,
    extractor: E,
    mut func: F,
) {
    const {
        assert!(
            !T::CONTAINS_ANY_TAG_TERM,
            "a type provided in the query signature is a Tag and cannot be used with \
             `.each`. use `.run` instead or provide the tag with `.with()`"
        );
    }

    #[cfg(feature = "flecs_safety_locks")]
    let world = unsafe { WorldRef::from_ptr((*iter).world) };
    #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    let world_ptr = iter.world;
    iter.flags |= sys::EcsIterCppEach;
    let (is_any_array, mut components_data) = T::create_ptrs(iter);
    let count = if iter.count == 0 && iter.table.is_null() {
        1_usize
    } else {
        iter.count as usize
    };

    ecs_assert!(
        !iter.entities.is_null(),
        FlecsErrorCode::InvalidOperation,
        "query/system does not return entities ($this variable is not populated).\nSystem: {:?}",
        {
            let world = unsafe { WorldRef::from_ptr(iter.world) };
            let e = world.entity_from_id(iter.system);
            (e.id(), e.get_name())
        }
    );

    #[cfg(feature = "flecs_safety_locks")]
    do_read_write_locks::<INCREMENT>(iter, T::COUNT as usize, &world);

    // only lock/unlock in debug or forced‑assert builds, and only
    // if we’re not in the “called from run” path:
    #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    if !CALLED_FROM_RUN {
        table_lock(world_ptr, iter.table);
    }

    if !is_any_array.a_ref && !is_any_array.a_row {
        for i in 0..count {
            let extra = unsafe { extractor.extract(iter, i) };
            let tuple = components_data.get_tuple(i);
            func(extra, tuple);
        }
    } else if is_any_array.a_row {
        for i in 0..count {
            let extra = unsafe { extractor.extract(iter, i) };
            let tuple = components_data.get_tuple_with_row(iter, i);
            func(extra, tuple);
        }
    } else {
        for i in 0..count {
            let extra = unsafe { extractor.extract(iter, i) };
            let tuple = components_data.get_tuple_with_ref(i);
            func(extra, tuple);
        }
    }

    #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    if !CALLED_FROM_RUN {
        table_unlock(world_ptr, iter.table);
    }

    #[cfg(feature = "flecs_safety_locks")]
    do_read_write_locks::<DECREMENT>(iter, T::COUNT as usize, &world);
}

#[inline(always)]
pub(crate) fn internal_each_iter_next<T: QueryTuple, const CALLED_FROM_RUN: bool>(
    iter: &mut sys::ecs_iter_t,
    func: &mut impl FnMut(T::TupleType<'_>),
) {
    // drop the `()` from the call
    internal_each_generic::<T, NoEntity, CALLED_FROM_RUN, _>(iter, NoEntity, move |(), t| func(t));
}

#[inline(always)]
pub(crate) fn internal_each_entity_iter_next<T: QueryTuple, const CALLED_FROM_RUN: bool>(
    iter: &mut sys::ecs_iter_t,
    world: &WorldRef<'_>,
    func: &mut impl FnMut(EntityView, T::TupleType<'_>),
) {
    // pass a WithEntity extractor so it builds the EntityView
    let extractor = WithEntity(world);
    internal_each_generic::<T, WithEntity<'_>, CALLED_FROM_RUN, _>(iter, extractor, func);
}
