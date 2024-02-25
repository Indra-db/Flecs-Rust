use std::{default, ops::Deref, os::raw::c_void, ptr};

use super::{
    c_binding::bindings::{
        ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t, ecs_filter_next, ecs_iter_action_t,
        ecs_iter_next, ecs_iter_t, ecs_observer_desc_t, ecs_table_lock, ecs_table_unlock,
    },
    c_types::{EntityT, IterT, TermT, WorldT, SEPARATOR},
    component_registration::CachedComponentData,
    entity::Entity,
    filter_builder::{FilterBuilder, FilterBuilderImpl},
    iter::Iter,
    iterable::{Filterable, Iterable},
    observer::Observer,
    term::TermBuilder,
    utility::types::ObserverSystemBindingCtx,
    world::World,
};

pub struct ObserverBuilder<'a, T>
where
    T: Iterable<'a>,
{
    filter_builder: FilterBuilder<'a, T>,
    desc: ecs_observer_desc_t,
    event_count: i32,
    /// non-owning world reference
    world: World,
    is_instanced: bool,
}

/// Deref to FilterBuilder to allow access to FilterBuilder methods without having to access FilterBuilder through ObserverBuilder
impl<'a, T> Deref for ObserverBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type Target = FilterBuilder<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.filter_builder
    }
}

impl<'a, T> ObserverBuilder<'a, T>
where
    T: Iterable<'a>,
{
    pub fn new(world: &World) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::<T>::new_with_desc(world, &mut desc.filter, 0),
            world: world.clone(),
            event_count: 0,
            is_instanced: false,
        };
        T::populate(&mut obj);
        obj.desc.filter = *obj.filter_builder.get_desc_filter();
        let mut entity_desc: ecs_entity_desc_t = Default::default();
        entity_desc.name = std::ptr::null();
        entity_desc.sep = SEPARATOR.as_ptr() as *const i8;
        obj.desc.entity = unsafe { ecs_entity_init(obj.world.raw_world, &entity_desc) };
        obj
    }

    pub fn new_named(world: &World, name: &str) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            filter_builder: FilterBuilder::new(world),
            world: world.clone(),
            event_count: 0,
            is_instanced: false,
        };
        T::populate(&mut obj);
        obj.desc.filter = *obj.filter_builder.get_desc_filter();
        let mut entity_desc: ecs_entity_desc_t = Default::default();
        let c_name = std::ffi::CString::new(name).expect("Failed to convert to CString");
        entity_desc.name = c_name.as_ptr() as *const i8;
        entity_desc.sep = SEPARATOR.as_ptr() as *const i8;
        obj.desc.entity = unsafe { ecs_entity_init(obj.world.raw_world, &entity_desc) };
        obj
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

    pub fn build(&mut self) -> Observer {
        Observer::new(&self.world, self.desc, self.is_instanced)
    }

    pub fn on_each<Func>(&mut self, func: Func) -> Observer
    where
        Func: FnMut(T::TupleType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let each_func = Box::new(func);
        let each_static_ref = Box::leak(each_func);

        binding_ctx.each = Some(each_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each = Some(Self::on_free_each);

        self.desc.callback = Some(Self::run_each::<Func> as unsafe extern "C" fn(_));

        self.build()
    }

    pub fn on_each_entity<Func>(&mut self, func: Func) -> Observer
    where
        Func: FnMut(&mut Entity, T::TupleType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let each_entity_func = Box::new(func);
        let each_entity_static_ref = Box::leak(each_entity_func);

        binding_ctx.each_entity = Some(each_entity_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each_entity = Some(Self::on_free_each);

        self.desc.callback = Some(Self::run_each_entity::<Func> as unsafe extern "C" fn(_));

        self.build()
    }

    pub fn on_iter_only<Func>(&mut self, func: Func) -> Observer
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

    pub fn on_iter<Func>(&mut self, func: Func) -> Observer
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

    unsafe extern "C" fn run_each_entity<Func>(iter: *mut IterT)
    where
        Func: FnMut(&mut Entity, T::TupleType),
    {
        let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
        let each_entity = (*ctx).each_entity.unwrap();
        let each_entity = &mut *(each_entity as *mut Func);

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
    }

    unsafe extern "C" fn run_iter_only<Func>(iter: *mut IterT)
    where
        Func: FnMut(&Iter),
    {
        unsafe {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let iter_only = (*ctx).iter_only.unwrap();
            let iter_only = &mut *(iter_only as *mut Func);

            let iter_count = (*iter).count as usize;

            for i in 0..iter_count {
                let iterT = Iter::new(&mut *iter);
                iter_only(&iterT);
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
            let iterT = Iter::new(&mut *iter);
            iter_func(&iterT, tuple);
        }

        ecs_table_unlock((*iter).world, (*iter).table);
    }
}

impl<'a, T> Filterable for ObserverBuilder<'a, T>
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
        self.filter_builder.next_term()
    }
}

impl<'a, T> FilterBuilderImpl for ObserverBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_filter(&mut self) -> &mut ecs_filter_desc_t {
        self.filter_builder.get_desc_filter()
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

impl<'a, T> TermBuilder for ObserverBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_world(&self) -> *mut WorldT {
        self.filter_builder.world.raw_world
    }

    #[inline]
    fn get_term(&mut self) -> &mut super::term::Term {
        self.filter_builder.get_term()
    }

    #[inline]
    fn get_raw_term(&mut self) -> *mut TermT {
        self.filter_builder.get_raw_term()
    }

    #[inline]
    fn get_term_id(&mut self) -> *mut super::c_types::TermIdT {
        self.filter_builder.get_term_id()
    }
}

pub trait ObserverBuilderImpl: FilterBuilderImpl {
    fn get_desc_observer(&mut self) -> &mut ecs_observer_desc_t;

    fn get_event_count(&self) -> i32;

    fn increment_event_count(&mut self);

    fn add_event(&mut self, event: EntityT) -> &mut Self {
        let event_count = self.get_event_count() as usize;
        self.increment_event_count();
        let desc = self.get_desc_observer();
        desc.events[event_count] = event;
        self
    }

    //todo!() function name
    fn add_event_of_type<T>(&mut self) -> &mut Self
    where
        T: CachedComponentData,
    {
        let event_count = self.get_event_count() as usize;
        self.increment_event_count();
        let id = T::get_id(self.get_world());
        let desc = self.get_desc_observer();
        desc.events[event_count] = id;
        self
    }

    //todo!() better function name
    fn yield_existing(&mut self, should_yield: bool) -> &mut Self {
        self.get_desc_observer().yield_existing = should_yield;
        self
    }

    fn set_context(&mut self, context: *mut c_void) -> &mut Self {
        self.get_desc_observer().ctx = context;
        self
    }

    fn set_run_callback(&mut self, callback: ecs_iter_action_t) -> &mut Self {
        self.get_desc_observer().run = callback;
        self
    }
}

impl<'a, T> ObserverBuilderImpl for ObserverBuilder<'a, T>
where
    T: Iterable<'a>,
{
    fn get_desc_observer(&mut self) -> &mut ecs_observer_desc_t {
        &mut self.desc
    }

    fn get_event_count(&self) -> i32 {
        self.event_count
    }

    fn increment_event_count(&mut self) {
        self.event_count += 1;
    }
}
