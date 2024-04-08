use std::ffi::c_void;

use flecs_ecs_derive::tuples;
use flecs_ecs_sys::{ecs_iter_action_t, ecs_table_lock, ecs_table_unlock};

use crate::core::{
    builder, iterable::ComponentPointers, iterable::IterableTypeOperation, Builder, Entity, Iter,
    IterT, Iterable, ObserverSystemBindingCtx,
};

use super::{private, WorldRef};

pub trait ReactorAPI<'a, T>: Builder<'a> + private::internal_ReactorAPI<'a, T>
where
    T: Iterable,
{
    /// Set action / ctx
    ///
    /// # Arguments
    ///
    /// * `callback` - the callback to set
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::run`
    #[doc(alias = "system_builder_i::ctx")]
    #[doc(alias = "observer_builder_i::ctx")]
    fn set_run_callback(&mut self, callback: ecs_iter_action_t) -> &mut Self;

    fn set_instanced(&mut self, instanced: bool);

    /// Set context
    ///
    /// # See also
    ///
    /// * C++ API: `observer_builder_i::ctx`
    /// * C++ API: `system_builder_i::ctx`
    #[doc(alias = "observer_builder_i::ctx")]
    #[doc(alias = "system_builder_i::ctx")]
    fn set_context(&mut self, context: *mut c_void) -> &mut Self;

    fn each<F: EachReactorFunction<(), T>>(
        &'a mut self,
        func: F,
    ) -> <Self as builder::Builder>::BuiltType {
        self.each_inner(func)
    }

    fn each_entity<F: EachReactorFunction<(Entity<'static>,), T>>(
        &'a mut self,
        func: F,
    ) -> <Self as builder::Builder>::BuiltType {
        self.each_inner(func)
    }

    fn each_iter<F: EachReactorFunction<(Iter<'static>, usize), T>>(
        &'a mut self,
        func: F,
    ) -> <Self as builder::Builder>::BuiltType {
        self.each_inner(func)
    }

    fn each_inner<F: EachReactorFunction<I, T>, I: EachInput>(
        &'a mut self,
        func: F,
    ) -> <Self as builder::Builder>::BuiltType {
        let binding_ctx = self.get_binding_context();

        let each_func = Box::new(func);
        let each_static_ref = Box::leak(each_func);

        binding_ctx.func = Some(each_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_func = Some(F::free);

        self.set_desc_callback(Some(F::run_iter as unsafe extern "C" fn(_)));

        self.set_instanced(true);

        self.build()
    }

    fn iter<F: IterReactorFunction<(), T>>(&'a mut self, func: F) {
        self.iter_inner(func);
    }

    fn iter_with<F: IterReactorFunction<(Iter<'static>,), T>>(&'a mut self, func: F) {
        self.iter_inner(func);
    }

    fn iter_inner<F: IterReactorFunction<I, T>, I: IterInput>(
        &'a mut self,
        func: F,
    ) -> <Self as builder::Builder>::BuiltType {
        let binding_ctx = self.get_binding_context();

        let iter_func = Box::new(func);
        let iter_static_ref = Box::leak(iter_func);

        binding_ctx.func = Some(iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_func = Some(F::free);

        self.set_desc_callback(Some(F::run_iter as unsafe extern "C" fn(_)));

        //TODO are we sure this shouldn't be instanced?

        self.build()
    }
}

macro_rules! implement_reactor_api {
    ($type:ty) => {
        impl<'a, T> internal_ReactorAPI<'a, T> for $type
        where
            T: Iterable,
        {
            fn set_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.binding_ctx = binding_ctx;
                self
            }

            fn set_binding_context_free(
                &mut self,
                binding_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.binding_ctx_free = binding_ctx_free;
                self
            }

            fn desc_binding_context(&self) -> *mut c_void {
                self.desc.binding_ctx
            }

            fn set_desc_callback(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut flecs_ecs_sys::ecs_iter_t)>,
            ) {
                self.desc.callback = callback;
            }
        }

        impl<'a, T> ReactorAPI<'a, T> for $type
        where
            T: Iterable,
        {
            fn set_run_callback(&mut self, callback: ecs_iter_action_t) -> &mut Self {
                self.desc.run = callback;
                self
            }

            fn set_instanced(&mut self, instanced: bool) {
                self.is_instanced = instanced;
            }

            fn set_context(&mut self, context: *mut c_void) -> &mut Self {
                self.desc.ctx = context;
                self
            }
        }
    };
}

pub(crate) use implement_reactor_api;

pub trait IterInput {
    type Item<'w>;
    unsafe fn from_iter<'w>(iter: &'w *mut IterT) -> Self::Item<'w>;
}

impl IterInput for () {
    type Item<'w> = ();
    unsafe fn from_iter(_: &*mut IterT) -> () {}
}

impl<'a> IterInput for (Iter<'a>,) {
    type Item<'w> = Iter<'w>;
    unsafe fn from_iter<'w>(iter: &'w *mut IterT) -> Self::Item<'w> {
        Iter::new(&mut (**iter))
    }
}

pub trait EachInput {
    type Item<'w>;
    unsafe fn from_iter<'w>(iter: &'w *mut IterT, index: usize) -> Self::Item<'w>;
}

impl EachInput for () {
    type Item<'w> = ();
    unsafe fn from_iter<'w>(_: &'w *mut IterT, _: usize) -> Self::Item<'w> {}
}

impl<'a> EachInput for (Iter<'a>, usize) {
    type Item<'w> = (Iter<'w>, usize);
    unsafe fn from_iter<'w>(iter: &'w *mut IterT, index: usize) -> Self::Item<'w> {
        (Iter::new(&mut (**iter)), index)
    }
}

impl<'a> EachInput for (Entity<'a>,) {
    type Item<'w> = Entity<'w>;
    unsafe fn from_iter<'w>(iter: &'w *mut IterT, index: usize) -> Self::Item<'w> {
        Entity::new_from_existing_raw(
            WorldRef::from_ptr((**iter).world),
            *(**iter).entities.add(index),
        )
    }
}

pub trait IterReactorFunction<I: IterInput, T: Iterable>: Send + Sync + Sized {
    fn run<'w>(&mut self, iter: I::Item<'w>, components: ReactorSlice<T>);

    unsafe fn run_self(&mut self, iter: *mut IterT) {
        let iter_count = {
            if (*iter).count == 0 {
                1_usize
            } else {
                (*iter).count as usize
            }
        };

        let mut components_data = T::get_ptrs(&*iter);
        ecs_table_lock((*iter).world, (*iter).table);

        let tuple = components_data.get_slice(iter_count);
        self.run(I::from_iter(&iter), tuple);
        ecs_table_unlock((*iter).world, (*iter).table);
    }

    unsafe extern "C" fn run_iter(iter: *mut IterT) {
        let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
        let func = (*ctx).func.unwrap();
        let func = &mut *(func as *mut Self);

        func.run_self(iter);
    }

    extern "C" fn free(ptr: *mut c_void) {
        let ptr_func = ptr as *mut Self;
        unsafe {
            std::ptr::drop_in_place(ptr_func);
        }
    }
}

pub trait EachReactorFunction<I: EachInput, T: Iterable>: Send + Sync + Sized {
    fn run<'w>(&mut self, iter: I::Item<'w>, components: ReactorTuple<T>);

    unsafe fn run_self(&mut self, iter: *mut IterT) {
        let iter_count = {
            if (*iter).count == 0 {
                1_usize
            } else {
                (*iter).count as usize
            }
        };

        let mut components_data = T::get_ptrs(&*iter);
        ecs_table_lock((*iter).world, (*iter).table);

        for i in 0..iter_count {
            let tuple = components_data.get_tuple(iter_count);
            self.run(I::from_iter(&iter, i), tuple);
        }
        ecs_table_unlock((*iter).world, (*iter).table);
    }

    unsafe extern "C" fn run_iter(iter: *mut IterT) {
        let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
        let func = (*ctx).func.unwrap();
        let func = &mut *(func as *mut Self);

        func.run_self(iter);
    }

    extern "C" fn free(ptr: *mut c_void) {
        let ptr_func = ptr as *mut Self;
        unsafe {
            std::ptr::drop_in_place(ptr_func);
        }
    }
}

pub type ReactorTuple<'a, T> = <T as Iterable>::TupleType<'a>;
pub type ReactorSlice<'a, T> = <T as Iterable>::TupleSliceType<'a>;
pub type EachInputItem<'a, T> = <T as EachInput>::Item<'a>;
pub type IterInputItem<'a, T> = <T as IterInput>::Item<'a>;

macro_rules! impl_reactor_function {
    ($fn_trait: ident, $input_ident: ident, $input_item: ident, $comp_item: ident, ($($input: ident: $input_ty: ty,)*), $component: ident) => {
        #[allow(non_snake_case)]
        impl<Func: Send + Sync + 'static, $component: IterableTypeOperation> $fn_trait<($($input_ty,)*), $component> for Func
        where
        for <'a> &'a mut Func:
            FnMut($input_item<($($input_ty,)*)>, $comp_item<$component>)
        {
            #[inline]
            fn run(&mut self, input: $input_item<($($input_ty,)*)>, component_data: $comp_item<$component>) {
                #[allow(clippy::too_many_arguments)]
                fn call_inner<$input_ident, $component>(
                    mut f: impl FnMut($input_ident, $component),
                    $input_ident: $input_ident,
                    $component: $component
                ){
                    f($input_ident, $component)
                }
                call_inner(self, input, component_data)
            }
        }
    };
    ($fn_trait: ident, $input_ident: ident, $input_item: ident, $comp_item: ident, ($($input: ident: $input_ty: ty,)*), $($component: ident),*) => {
        #[allow(non_snake_case)]
        impl<Func: Send + Sync + 'static, $($component: IterableTypeOperation),*> $fn_trait<($($input_ty,)*), ($($component,)*)> for Func
        where
        for <'a> &'a mut Func:
            FnMut($input_item<($($input_ty,)*)>,  $($comp_item<$component>),*)
        {
            #[inline]
            fn run(&mut self, input: $input_item<($($input_ty,)*)>, component_data: $comp_item< ($($component,)*)>) {
                #[allow(clippy::too_many_arguments)]
                fn call_inner<$input_ident, $($component),*>(
                    mut f: impl FnMut($input_ident, $($component),*),
                    $input_ident: $input_ident,
                    $($component: $component,)*
                ){
                    f($input_ident, $($component,)*)
                }
                let ($($component,)*) = component_data;
                call_inner(self, input, $($component),*)
            }
        }
    };
}

struct InputMarker {}

tuples!(
    impl_reactor_function,
    0,
    12,
    IterReactorFunction,
    InputMarker,
    IterInputItem,
    ReactorSlice
);

tuples!(
    impl_reactor_function,
    0,
    12,
    IterReactorFunction,
    InputMarker,
    IterInputItem,
    ReactorSlice,
    IterIdent: Iter<'static>
);

tuples!(
    impl_reactor_function,
    0,
    12,
    EachReactorFunction,
    InputMarker,
    EachInputItem,
    ReactorTuple
);

tuples!(
    impl_reactor_function,
    0,
    12,
    EachReactorFunction,
    InputMarker,
    EachInputItem,
    ReactorTuple,
    EntityIdent: Entity<'static>
);

tuples!(
    impl_reactor_function,
    0,
    12,
    EachReactorFunction,
    InputMarker,
    EachInputItem,
    ReactorTuple,
    IterIdent: Iter<'static>,
    IndexIdent: usize
);
