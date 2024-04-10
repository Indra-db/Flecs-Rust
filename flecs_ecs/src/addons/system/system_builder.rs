//! Systems are a query + function that can be ran manually or by a pipeline.

use std::ops::DerefMut;

use crate::addons::system::*;
use crate::core::private::internal_ReactorAPI;
use crate::core::*;

pub struct SystemBuilder<'a, T>
where
    T: Iterable,
{
    query_builder: QueryBuilder<'a, T>,
    desc: sys::ecs_system_desc_t,
    is_instanced: bool,
}

/// Deref to `QueryBuilder` to allow access to `QueryBuilder` methods without having to access `QueryBuilder` through `SystemBuilder`
impl<'a, T> Deref for SystemBuilder<'a, T>
where
    T: Iterable,
{
    type Target = QueryBuilder<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.query_builder
    }
}

impl<'a, T> DerefMut for SystemBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query_builder
    }
}

impl<'a, T> SystemBuilder<'a, T>
where
    T: Iterable,
{
    pub fn new(world: &'a World) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::<T>::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };

        let entity_desc: sys::ecs_entity_desc_t = sys::ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };
        obj.desc.entity = unsafe { sys::ecs_entity_init(obj.world.world_ptr_mut(), &entity_desc) };

        T::populate(&mut obj);

        #[cfg(feature = "flecs_pipeline")]
        unsafe {
            sys::ecs_add_id(
                world.world_ptr_mut(),
                obj.desc.entity,
                ecs_dependson(ECS_ON_UPDATE),
            );
            sys::ecs_add_id(world.world_ptr_mut(), obj.desc.entity, ECS_ON_UPDATE);
        }

        obj
    }

    pub fn new_from_desc(world: &'a World, mut desc: sys::ecs_system_desc_t) -> Self {
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::<T>::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };
        let entity_desc: sys::ecs_entity_desc_t = sys::ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };
        obj.desc.entity = unsafe { sys::ecs_entity_init(obj.world.world_ptr_mut(), &entity_desc) };

        T::populate(&mut obj);

        #[cfg(feature = "flecs_pipeline")]
        unsafe {
            sys::ecs_add_id(
                world.world_ptr_mut(),
                obj.desc.entity,
                ecs_dependson(ECS_ON_UPDATE),
            );
            sys::ecs_add_id(world.world_ptr_mut(), obj.desc.entity, ECS_ON_UPDATE);
        }

        obj
    }

    pub fn new_named(world: &'a World, name: &CStr) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::<T>::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };
        let entity_desc: sys::ecs_entity_desc_t = sys::ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };
        obj.desc.entity = unsafe { sys::ecs_entity_init(obj.world.world_ptr_mut(), &entity_desc) };
        T::populate(&mut obj);

        #[cfg(feature = "flecs_pipeline")]
        unsafe {
            sys::ecs_add_id(
                world.world_ptr_mut(),
                obj.desc.entity,
                ecs_dependson(ECS_ON_UPDATE),
            );
            sys::ecs_add_id(world.world_ptr_mut(), obj.desc.entity, ECS_ON_UPDATE);
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
    pub fn kind_id(&mut self, phase: impl IntoEntity) -> &mut Self {
        let phase = phase.get_id();
        let current_phase: EntityT = unsafe {
            sys::ecs_get_target(
                self.world.world_ptr_mut(),
                self.desc.entity,
                ECS_DEPENDS_ON,
                0,
            )
        };
        unsafe {
            if current_phase != 0 {
                sys::ecs_remove_id(
                    self.world.world_ptr_mut(),
                    self.desc.entity,
                    ecs_dependson(current_phase),
                );
                sys::ecs_remove_id(self.world.world_ptr_mut(), self.desc.entity, current_phase);
            }
            if phase != 0 {
                sys::ecs_add_id(
                    self.world.world_ptr_mut(),
                    self.desc.entity,
                    ecs_dependson(phase),
                );
                sys::ecs_add_id(self.world.world_ptr_mut(), self.desc.entity, phase);
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
        Phase: ComponentId,
    {
        self.kind_id(Phase::get_id(self.world))
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
    pub fn rate_w_tick_source(&mut self, tick_source: impl IntoEntity, rate: i32) -> &mut Self {
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
    pub fn tick_source_id(&mut self, tick_source: impl IntoEntity) -> &mut Self {
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
        Component: ComponentId,
    {
        self.desc.tick_source = Component::get_id(self.world);
        self
    }
}

impl<'a, T> Filterable<'a> for SystemBuilder<'a, T>
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

impl<'a, T> FilterBuilderImpl<'a> for SystemBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn desc_filter_mut(&mut self) -> &mut sys::ecs_filter_desc_t {
        &mut self.desc.query.filter
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

impl<'a, T> TermBuilder<'a> for SystemBuilder<'a, T>
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
    fn term_id_ptr_mut(&mut self) -> *mut TermIdT {
        self.filter_builder.term_id_ptr_mut()
    }
}

impl<'a, T> QueryBuilderImpl<'a> for SystemBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn desc_query_mut(&mut self) -> &mut sys::ecs_query_desc_t {
        &mut self.desc.query
    }
}

impl<'a, T> Builder<'a> for SystemBuilder<'a, T>
where
    T: Iterable,
{
    type BuiltType = System<'a>;

    /// Build the `system_builder` into an system
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        System::new(self.world, self.desc, self.is_instanced)
    }
}

impl<'a, T: Iterable> IntoWorld<'a> for SystemBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.query_builder.world()
    }
}

implement_reactor_api!(SystemBuilder<'a, T>);
