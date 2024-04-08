use std::{
    default,
    ffi::{c_void, CStr},
    ops::Deref,
};

use crate::sys::{
    ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t, ecs_iter_action_t, ecs_observer_desc_t,
};

use super::{
    c_types::{TermT, SEPARATOR},
    component_registration::ComponentId,
    filter_builder::{FilterBuilder, FilterBuilderImpl},
    implement_reactor_api,
    iterable::{Filterable, Iterable},
    observer::Observer,
    private::internal_ReactorAPI,
    term::TermBuilder,
    world::World,
    Builder, IntoEntityId, IntoWorld, ReactorAPI, Term,
};

pub struct ObserverBuilder<'a, T>
where
    T: Iterable,
{
    filter_builder: FilterBuilder<'a, T>,
    desc: ecs_observer_desc_t,
    event_count: i32,
    is_instanced: bool,
}

/// Deref to `FilterBuilder` to allow access to `FilterBuilder` methods without having to access `FilterBuilder` through `ObserverBuilder`
impl<'a, T> Deref for ObserverBuilder<'a, T>
where
    T: Iterable,
{
    type Target = FilterBuilder<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.filter_builder
    }
}

impl<'a, T> ObserverBuilder<'a, T>
where
    T: Iterable,
{
    /// Create a new observer builder
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    ///
    /// See also
    ///
    /// * C++ API: `observer_builder::observer_builder`
    #[doc(alias = "observer_builder::observer_builder")]
    pub fn new(world: &'a World) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::<T>::new_from_desc(world, &mut desc.filter, 0),
            event_count: 0,
            is_instanced: false,
        };

        let entity_desc: ecs_entity_desc_t = ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..default::Default::default()
        };

        obj.desc.entity = unsafe { ecs_entity_init(obj.world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }

    /// Create a new observer builder with a name
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `name` - The name of the observer
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::node_builder`
    #[doc(alias = "node_builder::node_builder")]
    pub fn new_named(world: &'a World, name: &CStr) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::<T>::new_from_desc(world, &mut desc.filter, 0),
            event_count: 0,
            is_instanced: false,
        };
        let entity_desc: ecs_entity_desc_t = ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..default::Default::default()
        };

        obj.desc.entity = unsafe { ecs_entity_init(obj.world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }

    /// Create a new observer builder from an existing descriptor
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `desc` - The descriptor to create the observer from
    ///
    /// See also
    ///
    /// * C++ API: `observer_builder::observer_builder`
    #[doc(alias = "observer_builder::observer_builder")]
    pub fn new_from_desc(world: &'a World, mut desc: ecs_observer_desc_t) -> Self {
        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::new_from_desc(world, &mut desc.filter, 0),
            event_count: 0,
            is_instanced: false,
        };

        let entity_desc: ecs_entity_desc_t = ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..default::Default::default()
        };

        obj.desc.entity = unsafe { ecs_entity_init(obj.world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }

    pub fn event_count(&self) -> i32 {
        self.event_count
    }

    /// Specify the event(s) for when the observer should run.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to add
    ///
    /// # See also
    ///
    /// * C++ API: `observer_builder_i::event`
    #[doc(alias = "observer_builder_i::event")]
    pub fn add_event_id(&mut self, event: impl IntoEntityId) -> &mut Self {
        let event = event.get_id();
        let event_count = self.event_count as usize;
        self.event_count += 1;
        self.desc.events[event_count] = event;
        self
    }

    /// Specify the event(s) for when the observer should run.
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of the event
    ///
    /// # See also
    ///
    /// * C++ API: `observer_builder_i::event`
    #[doc(alias = "observer_builder_i::event")]
    pub fn add_event<E>(&mut self) -> &mut Self
    where
        E: ComponentId,
    {
        let event_count = self.event_count as usize;
        self.event_count += 1;
        let id = E::get_id(self.world);
        self.desc.events[event_count] = id;
        self
    }

    /// Invoke observer for anything that matches its filter on creation
    ///
    /// # Arguments
    ///
    /// * `should_yield` - If true, the observer will be invoked for all existing entities that match its filter
    ///
    /// # See also
    ///
    /// * C++ API: `observer_builder_i::yield_existing`
    #[doc(alias = "observer_builder_i::yield_existing")]
    pub fn yield_existing(&mut self, should_yield: bool) -> &mut Self {
        self.desc.yield_existing = should_yield;
        self
    }
}

impl<'a, T> Filterable<'a> for ObserverBuilder<'a, T>
where
    T: Iterable,
{
    fn current_term(&mut self) -> &mut TermT {
        unsafe { &mut *self.filter_builder.term.term_ptr }
    }

    fn next_term(&mut self) {
        self.filter_builder.next_term();
    }
}

impl<'a, T> FilterBuilderImpl<'a> for ObserverBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn desc_filter_mut(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.desc.filter
    }

    #[inline]
    fn expr_count_mut(&mut self) -> &mut i32 {
        self.filter_builder.expr_count_mut()
    }

    #[inline]
    fn term_index_mut(&mut self) -> &mut i32 {
        self.filter_builder.term_index_mut()
    }
}

impl<'a, T> TermBuilder<'a> for ObserverBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn term_mut(&mut self) -> &mut Term<'a> {
        self.filter_builder.term_mut()
    }

    #[inline]
    fn term_ptr_mut(&mut self) -> *mut TermT {
        self.filter_builder.term_ptr_mut()
    }

    #[inline]
    fn term_id_ptr_mut(&mut self) -> *mut super::c_types::TermIdT {
        self.filter_builder.term_id_ptr_mut()
    }
}

impl<'a, T> Builder<'a> for ObserverBuilder<'a, T>
where
    T: Iterable,
{
    type BuiltType = Observer<'a>;

    /// Build the `observer_builder` into an `observer`
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        Observer::new(&self.world, self.desc, self.is_instanced)
    }
}

impl<'a, T: Iterable> IntoWorld<'a> for ObserverBuilder<'a, T> {
    fn get_world(&self) -> Option<&'a World> {
        self.filter_builder.get_world()
    }
}

implement_reactor_api!(ObserverBuilder<'a, T>);
