//! Systems are a query + function that can be ran manually or by a pipeline.
use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

use crate::{
    core::{
        c_types::{EntityT, FTimeT, TermIdT, TermT, WorldT, ECS_DEPENDS_ON, SEPARATOR},
        component_registration::ComponentInfo,
        ecs_dependson,
        filter_builder::FilterBuilderImpl,
        implement_reactor_api,
        iterable::{Filterable, Iterable},
        private::internal_ReactorAPI,
        query_builder::{QueryBuilder, QueryBuilderImpl},
        term::{Term, TermBuilder},
        world::World,
        Builder, IntoEntityId, ReactorAPI, ECS_ON_UPDATE,
    },
    sys::{
        ecs_add_id, ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t, ecs_get_target,
        ecs_iter_action_t, ecs_query_desc_t, ecs_remove_id, ecs_system_desc_t,
    },
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

/// Deref to `QueryBuilder` to allow access to `QueryBuilder` methods without having to access `QueryBuilder` through `SystemBuilder`
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

        let entity_desc: ecs_entity_desc_t = ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
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

    pub fn new_from_desc(world: &World, mut desc: ecs_system_desc_t) -> Self {
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::<T>::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };
        let entity_desc: ecs_entity_desc_t = ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
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

    pub fn new_named(world: &World, name: &CStr) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::<T>::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };
        let entity_desc: ecs_entity_desc_t = ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
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
    pub fn kind_id(&mut self, phase: impl IntoEntityId) -> &mut Self {
        let phase = phase.get_id();
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
        Phase: ComponentInfo,
    {
        self.kind_id(Phase::get_id(self.world.raw_world))
    }

    /// Specify whether system can run on multiple threads.
    ///
    /// # Arguments
    ///
    /// * `value` - if false, the system will always run on a single thread.
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::multi_threaded`
    #[doc(alias = "system_builder_i::multi_threaded")]
    pub fn multi_threaded(&mut self, value: bool) -> &mut Self {
        self.desc.multi_threaded = value;
        self
    }

    /// Specify whether system should be ran in staged context.
    ///
    /// # Arguments
    ///
    /// * `value` - If false,  system will always run staged.
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::no_readonly`
    #[doc(alias = "system_builder_i::no_readonly")]
    pub fn no_readonly(&mut self, value: bool) -> &mut Self {
        self.desc.no_readonly = value;
        self
    }

    /// Set system interval. This operation will cause the system to be ran at the specified interval.
    /// The timer is synchronous, and is incremented each frame by `delta_time`.
    ///
    /// # Arguments
    ///
    /// * `interval` - The interval value.
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::interval`
    #[doc(alias = "system_builder_i::interval")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::rate`
    #[doc(alias = "system_builder_i::rate")]
    pub fn rate_w_tick_source(&mut self, tick_source: impl IntoEntityId, rate: i32) -> &mut Self {
        self.desc.rate = rate;
        self.desc.tick_source = tick_source.get_id();
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
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::rate`
    #[doc(alias = "system_builder_i::rate")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::tick_source`
    #[doc(alias = "system_builder_i::tick_source")]
    pub fn tick_source_id(&mut self, tick_source: impl IntoEntityId) -> &mut Self {
        self.desc.tick_source = tick_source.get_id();
        self
    }

    /// Set tick source.
    /// This operation sets a shared tick source for the system.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type associated with the singleton tick source to use for the system
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::tick_source`
    #[doc(alias = "system_builder_i::tick_source")]
    pub fn tick_source<Component>(&mut self) -> &mut Self
    where
        Component: ComponentInfo,
    {
        self.desc.tick_source = Component::get_id(self.world.raw_world);
        self
    }
}

impl<'a, T> Filterable for SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    fn current_term(&mut self) -> &mut TermT {
        let next_term_index = self.next_term_index;
        &mut self.get_desc_filter().terms[next_term_index as usize]
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
        &mut self.desc.query.filter
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

impl<'a, T> Builder for SystemBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type BuiltType = System;

    /// Build the `system_builder` into an system
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        System::new(&self.world, self.desc, self.is_instanced)
    }
}

implement_reactor_api!(SystemBuilder<'a, T>);
