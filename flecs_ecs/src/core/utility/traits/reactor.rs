use std::ffi::c_void;

use crate::core::*;

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

    //fn set_instanced(&mut self, instanced: bool);

    /// Set context
    ///
    /// # See also
    ///
    /// * C++ API: `observer_builder_i::ctx`
    /// * C++ API: `system_builder_i::ctx`
    #[doc(alias = "observer_builder_i::ctx")]
    #[doc(alias = "system_builder_i::ctx")]
    fn set_context(&mut self, context: *mut c_void) -> &mut Self;

    fn on_each<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(T::TupleType<'_>),
    {
        let binding_ctx = self.get_binding_context();

        let each_func = Box::new(func);
        let each_static_ref = Box::leak(each_func);

        binding_ctx.each = Some(each_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each = Some(Self::on_free_each);

        self.set_desc_callback(Some(Self::run_each::<Func> as unsafe extern "C" fn(_)));

        //self.set_instanced(true);

        self.build()
    }

    fn on_each_entity<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(&mut EntityView, T::TupleType<'_>),
    {
        let binding_ctx = self.get_binding_context();

        let each_entity_func = Box::new(func);
        let each_entity_static_ref = Box::leak(each_entity_func);

        binding_ctx.each_entity = Some(each_entity_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each_entity = Some(Self::on_free_each_entity);

        self.set_desc_callback(Some(
            Self::run_each_entity::<Func> as unsafe extern "C" fn(_),
        ));

        //self.set_instanced(true);

        self.build()
    }

    fn on_each_iter<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(&mut Iter, usize, T::TupleType<'_>),
    {
        let binding_ctx = self.get_binding_context();

        let each_iter_func = Box::new(func);
        let each_iter_static_ref = Box::leak(each_iter_func);

        binding_ctx.each_iter = Some(each_iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each_iter = Some(Self::on_free_each_iter);

        self.set_desc_callback(Some(Self::run_each_iter::<Func> as unsafe extern "C" fn(_)));

        //self.set_instanced(true);

        self.build()
    }

    fn on_iter_only<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(&mut Iter),
    {
        let binding_ctx = self.get_binding_context();
        let iter_func = Box::new(func);
        let iter_static_ref = Box::leak(iter_func);
        binding_ctx.iter_only = Some(iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_iter_only = Some(Self::on_free_iter_only);

        self.set_desc_callback(Some(Self::run_iter_only::<Func> as unsafe extern "C" fn(_)));

        //TODO are we sure this shouldn't be instanced?

        self.build()
    }

    fn on_iter<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(&mut Iter, T::TupleSliceType<'_>),
    {
        let binding_ctx = self.get_binding_context();

        let iter_func = Box::new(func);
        let iter_static_ref = Box::leak(iter_func);

        binding_ctx.iter = Some(iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_iter = Some(Self::on_free_iter);

        self.set_desc_callback(Some(Self::run_iter::<Func> as unsafe extern "C" fn(_)));

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
            fn set_run_callback(
                &mut self,
                callback: flecs_ecs::sys::ecs_iter_action_t,
            ) -> &mut Self {
                self.desc.run = callback;
                self
            }

            // fn set_instanced(&mut self, instanced: bool) {
            //     self.is_instanced = instanced;
            // }

            fn set_context(&mut self, context: *mut c_void) -> &mut Self {
                self.desc.ctx = context;
                self
            }
        }
    };
}

use flecs_ecs_sys::ecs_iter_action_t;
pub(crate) use implement_reactor_api;
