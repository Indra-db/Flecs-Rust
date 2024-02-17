use std::{ops::Deref, os::raw::c_void};

use super::{
    c_binding::bindings::{ecs_filter_desc_t, ecs_iter_action_t, ecs_observer_desc_t},
    c_types::{EntityT, TermT, WorldT, SEPARATOR},
    component_registration::CachedComponentData,
    filter_builder::{FilterBuilder, FilterBuilderImpl},
    iterable::{Filterable, Iterable},
    term::TermBuilder,
    world::World,
};

pub struct ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    pub filter_builder: FilterBuilder<'a, 'w, T>,
    pub desc: ecs_observer_desc_t,
    pub event_count: i32,
    pub world: &'w World,
}

impl<'a, 'w, T> Deref for ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    type Target = FilterBuilder<'a, 'w, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.filter_builder
    }
}

impl<'a, 'w, T> ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    pub fn new(world: &'w World) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::new_with_desc(world, &mut desc.filter, 0),
            world,
            event_count: 0,
        };
        T::populate(&mut obj);
        obj
    }

    pub fn new_named(world: &'w World, name: &str) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            filter_builder: FilterBuilder::new(world),
            world,
            event_count: 0,
        };
        T::populate(&mut obj);
        obj
    }
}

impl<'a, 'w, T> Filterable for ObserverBuilder<'a, 'w, T>
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

impl<'a, 'w, T> FilterBuilderImpl for ObserverBuilder<'a, 'w, T>
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

impl<'a, 'w, T> TermBuilder for ObserverBuilder<'a, 'w, T>
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
    fn get_desc_observer(&self) -> &mut ecs_observer_desc_t;

    fn get_event_count(&self) -> i32;

    fn increment_event_count(&mut self);

    fn add_event(&mut self, event: EntityT) -> &mut Self {
        let desc = self.get_desc_observer();
        let event_count = self.get_event_count() as usize;
        desc.events[event_count] = event;
        self.increment_event_count();
        self
    }

    //todo!()
    fn add_event_of_type<T>(&mut self) -> &mut Self
    where
        T: CachedComponentData,
    {
        let desc = self.get_desc_observer();
        let event_count = self.get_event_count() as usize;
        let id = T::get_id(self.get_world());
        desc.events[event_count] = id;
        self.increment_event_count();
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
