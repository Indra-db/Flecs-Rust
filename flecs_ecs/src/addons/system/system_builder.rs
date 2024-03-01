use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    os::raw::c_void,
    ptr,
};

use crate::core::{
    c_binding::bindings::{
        ecs_add_id, ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t, ecs_get_target,
        ecs_iter_action_t, ecs_iter_next, ecs_query_desc_t, ecs_remove_id, ecs_system_desc_t,
        ecs_table_lock, ecs_table_unlock,
    },
    c_types::{EntityT, FTimeT, IterT, TermIdT, TermT, WorldT, ECS_DEPENDS_ON, SEPARATOR},
    component_registration::CachedComponentData,
    entity::Entity,
    filter_builder::FilterBuilderImpl,
    iter::Iter,
    iterable::{Filterable, Iterable},
    query_builder::{QueryBuilder, QueryBuilderImpl},
    term::{Term, TermBuilder},
    utility::{functions::ecs_dependson, types::ObserverSystemBindingCtx},
    world::World,
    ECS_ON_UPDATE,
};

use super::System;

pub struct SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    query_builder: QueryBuilder<'a, T>,
    desc: ecs_system_desc_t,
    is_instanced: bool,
}

/// Deref to QueryBuilder to allow access to QueryBuilder methods without having to access QueryBuilder through SystemBuilder
impl<'a, T> Deref for SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type Target = QueryBuilder<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.query_builder
    }
}

impl<'a, T> DerefMut for SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query_builder
    }
}

impl<'a, T> SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    pub fn new(world: &World) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::<T>::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };
        obj.desc.query = *obj.query_builder.get_desc_query();
        obj.desc.query.filter = *obj.filter_builder.get_desc_filter();
        T::populate(&mut obj);
        obj
    }

    pub fn new_from_desc(world: &World, mut desc: ecs_system_desc_t) -> Self {
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::<T>::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };
        obj.desc.query = *obj.query_builder.get_desc_query();
        obj.desc.query.filter = *obj.filter_builder.get_desc_filter();
        T::populate(&mut obj);
        obj
    }

    pub fn new_named(world: &World, name: &CStr) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            query_builder: QueryBuilder::new_named(world, name),
            is_instanced: false,
        };
        obj.desc.query = *obj.query_builder.get_desc_query();
        obj.desc.query.filter = *obj.filter_builder.get_desc_filter();
        let entity_desc: ecs_entity_desc_t = ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };
        obj.desc.entity = unsafe { ecs_entity_init(obj.world.raw_world, &entity_desc) };
        T::populate(&mut obj);

        #[cfg(feature = "flecs_pipeline")]
        unsafe {
            ecs_add_id(
                world.raw_world,
                obj.desc.entity,
                ecs_dependson(ECS_ON_UPDATE),
            );
            ecs_add_id(world.raw_world, obj.desc.entity, ECS_ON_UPDATE);
        }
        obj
    }

    /// Specify in which phase the system should run
    ///
    /// # Arguments
    ///
    /// * `phase` - the phase
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::kind`
    #[doc(alias = "system_builder_i::kind")]
    pub fn kind_id(&mut self, phase: EntityT) -> &mut Self {
        let current_phase: EntityT =
            unsafe { ecs_get_target(self.world.raw_world, self.desc.entity, ECS_DEPENDS_ON, 0) };
        unsafe {
            if current_phase != 0 {
                ecs_remove_id(
                    self.world.raw_world,
                    self.desc.entity,
                    ecs_dependson(current_phase),
                );
                ecs_remove_id(self.world.raw_world, self.desc.entity, current_phase);
            }
            if phase != 0 {
                ecs_add_id(self.world.raw_world, self.desc.entity, ecs_dependson(phase));
                ecs_add_id(self.world.raw_world, self.desc.entity, phase);
            }
        };
        self
    }

    /// Specify in which phase the system should run
    ///
    /// # Type Parameters
    ///
    /// * `Phase` - the phase
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::kind`
    #[doc(alias = "system_builder_i::kind")]
    pub fn kind<Phase>(&mut self) -> &mut Self
    where
        Phase: CachedComponentData,
    {
        self.kind_id(Phase::get_id(self.world.raw_world))
    }

    /// Specify whether system can run on multiple threads.
    ///
    /// # Arguments
    ///
    /// * `value` - if false, the system will always run on a single thread.
    pub fn multi_threaded(&mut self, value: bool) -> &mut Self {
        self.desc.multi_threaded = value;
        self
    }

    /// Specify whether system should be ran in staged context.
    ///
    /// # Arguments
    ///
    /// * `value` - If false,  system will always run staged.
    pub fn no_readonly(&mut self, value: bool) -> &mut Self {
        self.desc.no_readonly = value;
        self
    }

    /// Set system interval. This operation will cause the system to be ran at the specified interval.
    /// The timer is synchronous, and is incremented each frame by delta_time.
    ///
    /// # Arguments
    ///
    /// * `interval` - The interval value.
    pub fn interval(&mut self, interval: FTimeT) -> &mut Self {
        self.desc.interval = interval;
        self
    }

    /// Set system rate.
    /// This operation will cause the system to be ran at a multiple of the
    /// provided tick source. The tick source may be any entity, including
    /// another system.
    ///
    /// # Arguments
    ///
    /// * `tick_source` - The tick source.
    /// * `rate` - The multiple at which to run the system
    pub fn rate_w_tick_source(&mut self, tick_source: EntityT, rate: i32) -> &mut Self {
        self.desc.rate = rate;
        self.desc.tick_source = tick_source;
        self
    }

    /// Set system rate.
    /// This operation will cause the system to be ran at a multiple of the
    /// frame tick frequency. If a tick source was provided, this just updates
    /// the rate of the system.
    ///
    /// # Arguments
    ///
    /// * `rate` - The multiple at which to run the system
    pub fn rate(&mut self, rate: i32) -> &mut Self {
        self.desc.rate = rate;
        self
    }

    /// Set tick source.
    /// This operation sets a shared tick source for the system.
    ///
    /// # Arguments
    ///
    /// * `tick_source` - The tick source.
    pub fn tick_source(&mut self, tick_source: EntityT) -> &mut Self {
        self.desc.tick_source = tick_source;
        self
    }

    fn get_binding_ctx(&mut self) -> &mut ObserverSystemBindingCtx {
        let mut binding_ctx: *mut ObserverSystemBindingCtx = self.desc.binding_ctx as *mut _;

        if binding_ctx.is_null() {
            let new_binding_ctx = Box::<ObserverSystemBindingCtx>::default();
            let static_ref = Box::leak(new_binding_ctx);
            binding_ctx = static_ref;
            self.desc.binding_ctx = binding_ctx as *mut c_void;
            self.desc.binding_ctx_free = Some(Self::binding_ctx_drop);
        }
        unsafe { &mut *binding_ctx }
    }

    extern "C" fn binding_ctx_drop(ptr: *mut c_void) {
        let ptr_struct: *mut ObserverSystemBindingCtx = ptr as *mut ObserverSystemBindingCtx;
        unsafe {
            ptr::drop_in_place(ptr_struct);
        }
    }

    /// Set system context
    ///
    /// # Arguments
    ///
    /// * `context` - the context to set
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::ctx`
    #[doc(alias = "system_builder_i::ctx")]
    fn set_context(&mut self, context: *mut c_void) -> &mut Self {
        self.desc.ctx = context;
        self
    }

    /// Set system action
    ///
    /// # Arguments
    ///
    /// * `callback` - the callback to set
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::run`
    #[doc(alias = "system_builder_i::run")]
    fn set_run_callback(&mut self, callback: ecs_iter_action_t) -> &mut Self {
        self.desc.run = callback;
        self
    }

    pub fn build(&mut self) -> System {
        System::new(&self.world, self.desc, self.is_instanced)
    }

    pub fn on_each<Func>(&mut self, func: Func) -> System
    where
        Func: FnMut(T::TupleType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let each_func = Box::new(func);
        let each_static_ref = Box::leak(each_func);

        binding_ctx.each = Some(each_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each = Some(Self::on_free_each);

        self.desc.callback = Some(Self::run_each::<Func> as unsafe extern "C" fn(_));

        self.is_instanced = true;

        self.build()
    }

    pub fn on_each_entity<Func>(&mut self, func: Func) -> System
    where
        Func: FnMut(&mut Entity, T::TupleType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let each_entity_func = Box::new(func);
        let each_entity_static_ref = Box::leak(each_entity_func);

        binding_ctx.each_entity = Some(each_entity_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each_entity = Some(Self::on_free_each);

        self.desc.callback = Some(Self::run_each_entity::<Func> as unsafe extern "C" fn(_));

        self.is_instanced = true;

        self.build()
    }

    pub fn on_iter_only<Func>(&mut self, func: Func) -> System
    where
        Func: FnMut(&Iter) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();
        let iter_func = Box::new(func);
        let iter_static_ref = Box::leak(iter_func);
        binding_ctx.iter_only = Some(iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_iter_only = Some(Self::on_free_iter_only);

        self.desc.callback = Some(Self::run_iter_only::<Func> as unsafe extern "C" fn(_));

        self.build()
    }

    pub fn on_iter<Func>(&mut self, func: Func) -> System
    where
        Func: FnMut(&Iter, T::TupleSliceType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let iter_func = Box::new(func);
        let iter_static_ref = Box::leak(iter_func);

        binding_ctx.iter = Some(iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_iter = Some(Self::on_free_iter);

        self.desc.callback = Some(Self::run_iter::<Func> as unsafe extern "C" fn(_));

        self.build()
    }

    extern "C" fn on_free_each(ptr: *mut c_void) {
        let ptr_func: *mut fn(T::TupleType) = ptr as *mut fn(T::TupleType);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    extern "C" fn on_free_each_entity(ptr: *mut c_void) {
        let ptr_func: *mut fn(&mut Entity, T::TupleType) =
            ptr as *mut fn(&mut Entity, T::TupleType);
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
        let ptr_func: *mut fn(&Iter, T::TupleSliceType) = ptr as *mut fn(&Iter, T::TupleSliceType);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    unsafe extern "C" fn run_each<Func>(iter: *mut IterT)
    where
        Func: FnMut(T::TupleType),
    {
        let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
        let each = (*ctx).each.unwrap();
        let each = &mut *(each as *mut Func);

        while ecs_iter_next(iter) {
            let components_data = T::get_array_ptrs_of_components(&*iter);
            let iter_count = (*iter).count as usize;
            let array_components = &components_data.array_components;

            ecs_table_lock((*iter).world, (*iter).table);

            for i in 0..iter_count {
                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                } else {
                    T::get_tuple(array_components, i)
                };
                each(tuple);
            }

            ecs_table_unlock((*iter).world, (*iter).table);
        }
    }

    unsafe extern "C" fn run_each_entity<Func>(iter: *mut IterT)
    where
        Func: FnMut(&mut Entity, T::TupleType),
    {
        let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
        let each_entity = (*ctx).each_entity.unwrap();
        let each_entity = &mut *(each_entity as *mut Func);

        //while ecs_iter_next(iter) {
        let components_data = T::get_array_ptrs_of_components(&*iter);
        let array_components = &components_data.array_components;
        let iter_count = (*iter).count as usize;

        ecs_table_lock((*iter).world, (*iter).table);

        for i in 0..iter_count {
            let mut entity = Entity::new_from_existing_raw((*iter).world, *(*iter).entities.add(i));
            let tuple = if components_data.is_any_array_a_ref {
                let is_ref_array_components = &components_data.is_ref_array_components;
                T::get_tuple_with_ref(array_components, is_ref_array_components, i)
            } else {
                T::get_tuple(array_components, i)
            };

            each_entity(&mut entity, tuple);
        }
        ecs_table_unlock((*iter).world, (*iter).table);
        //}
    }

    unsafe extern "C" fn run_iter_only<Func>(iter: *mut IterT)
    where
        Func: FnMut(&Iter),
    {
        unsafe {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let iter_only = (*ctx).iter_only.unwrap();
            let iter_only = &mut *(iter_only as *mut Func);

            while ecs_iter_next(iter) {
                let iter_t = Iter::new(&mut *iter);
                iter_only(&iter_t);
            }
        }
    }

    unsafe extern "C" fn run_iter<Func>(iter: *mut IterT)
    where
        Func: FnMut(&Iter, T::TupleSliceType),
    {
        let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
        let iter_func = (*ctx).iter.unwrap();
        let iter_func = &mut *(iter_func as *mut Func);
        while ecs_iter_next(iter) {
            let components_data = T::get_array_ptrs_of_components(&*iter);
            let array_components = &components_data.array_components;
            let iter_count = (*iter).count as usize;

            ecs_table_lock((*iter).world, (*iter).table);

            for i in 0..iter_count {
                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_slices_with_ref(array_components, is_ref_array_components, i)
                } else {
                    T::get_tuple_slices(array_components, i)
                };
                let iter_t = Iter::new(&mut *iter);
                iter_func(&iter_t, tuple);
            }

            ecs_table_unlock((*iter).world, (*iter).table);
        }
    }
}

impl<'a, T> Filterable for SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    fn get_world(&self) -> *mut WorldT {
        self.filter_builder.world.raw_world
    }

    fn current_term(&mut self) -> &mut TermT {
        self.filter_builder.current_term()
    }

    fn next_term(&mut self) {
        self.filter_builder.next_term();
    }
}

impl<'a, T> FilterBuilderImpl for SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_filter(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.filter_builder.desc
    }

    #[inline]
    fn get_expr_count(&mut self) -> &mut i32 {
        self.filter_builder.get_expr_count()
    }

    #[inline]
    fn get_term_index(&mut self) -> &mut i32 {
        self.filter_builder.get_term_index()
    }
}

impl<'a, T> TermBuilder for SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_world(&self) -> *mut WorldT {
        self.filter_builder.world.raw_world
    }

    #[inline]
    fn get_term(&mut self) -> &mut Term {
        self.filter_builder.get_term()
    }

    #[inline]
    fn get_raw_term(&mut self) -> *mut TermT {
        self.filter_builder.get_raw_term()
    }

    #[inline]
    fn get_term_id(&mut self) -> *mut TermIdT {
        self.filter_builder.get_term_id()
    }
}

impl<'a, T> QueryBuilderImpl for SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_query(&mut self) -> &mut ecs_query_desc_t {
        &mut self.desc.query
    }
}
