//! `SystemBuilder` is a builder pattern for creating systems.

use crate::addons::system::*;
use crate::core::internals::*;
use crate::core::private::internal_SystemAPI;
use crate::core::*;

/// `SystemBuilder` is a builder pattern for creating systems.
pub struct SystemBuilder<'a, T>
where
    T: QueryTuple,
{
    pub(crate) desc: sys::ecs_system_desc_t,
    term_builder: TermBuilder,
    world: WorldRef<'a>,
    is_instanced: bool,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> SystemBuilder<'a, T>
where
    T: QueryTuple,
{
    /// Create a new system builder
    pub(crate) fn new(world: &'a World) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            term_builder: TermBuilder::default(),
            world: world.into(),
            _phantom: std::marker::PhantomData,
            is_instanced: false,
        };

        obj.desc.entity = unsafe { sys::ecs_entity_init(obj.world_ptr_mut(), &Default::default()) };

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

    pub(crate) fn new_from_desc(world: &'a World, desc: sys::ecs_system_desc_t) -> Self {
        let mut obj = Self {
            desc,
            term_builder: TermBuilder::default(),
            world: world.into(),
            _phantom: std::marker::PhantomData,
            is_instanced: false,
        };

        if obj.desc.entity == 0 {
            obj.desc.entity =
                unsafe { sys::ecs_entity_init(obj.world_ptr_mut(), &Default::default()) };
        }

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

    /// Create a new system builder with a name
    pub(crate) fn new_named(world: &'a World, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let mut obj = Self {
            desc: Default::default(),
            term_builder: TermBuilder::default(),
            world: world.into(),
            _phantom: std::marker::PhantomData,
            is_instanced: false,
        };

        let entity_desc: sys::ecs_entity_desc_t = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };
        obj.desc.entity = unsafe { sys::ecs_entity_init(obj.world_ptr_mut(), &entity_desc) };

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
    pub fn kind_id(&mut self, phase: impl Into<Entity>) -> &mut Self {
        let phase = *phase.into();
        let current_phase: sys::ecs_entity_t = unsafe {
            sys::ecs_get_target(self.world_ptr_mut(), self.desc.entity, ECS_DEPENDS_ON, 0)
        };
        unsafe {
            if current_phase != 0 {
                sys::ecs_remove_id(
                    self.world_ptr_mut(),
                    self.desc.entity,
                    ecs_dependson(current_phase),
                );
                sys::ecs_remove_id(self.world_ptr_mut(), self.desc.entity, current_phase);
            }
            if phase != 0 {
                sys::ecs_add_id(self.world_ptr_mut(), self.desc.entity, ecs_dependson(phase));
                sys::ecs_add_id(self.world_ptr_mut(), self.desc.entity, phase);
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
        Phase: ComponentId + ComponentType<Struct>,
    {
        self.kind_id(Phase::id(self.world()))
    }

    /// Specify in which enum phase the system should run
    ///
    /// # Arguments
    ///
    /// * `phase` - the phase
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::kind`
    #[doc(alias = "system_builder_i::kind")]
    pub fn kind_enum<Phase>(&mut self, phase: Phase) -> &mut Self
    where
        Phase: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let enum_id = phase.id_variant(self.world());
        self.kind_id(enum_id)
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
    pub fn multi_threaded(&mut self) -> &mut Self {
        self.desc.multi_threaded = true;
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
    /// * C++ API: `system_builder_i::immediate`
    #[doc(alias = "system_builder_i::immediate")]
    pub fn immediate(&mut self, value: bool) -> &mut Self {
        self.desc.immediate = value;
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
    pub fn rate_w_tick_source(&mut self, tick_source: impl Into<Entity>, rate: i32) -> &mut Self {
        self.desc.rate = rate;
        self.desc.tick_source = *tick_source.into();
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
    pub fn tick_source_id(&mut self, tick_source: impl Into<Entity>) -> &mut Self {
        self.desc.tick_source = *tick_source.into();
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
        self.desc.tick_source = Component::id(self.world());
        self
    }
}

#[doc(hidden)]
impl<'a, T: QueryTuple> internals::QueryConfig<'a> for SystemBuilder<'a, T> {
    #[inline(always)]
    fn term_builder(&self) -> &TermBuilder {
        &self.term_builder
    }

    #[inline(always)]
    fn term_builder_mut(&mut self) -> &mut TermBuilder {
        &mut self.term_builder
    }

    #[inline(always)]
    fn query_desc(&self) -> &sys::ecs_query_desc_t {
        &self.desc.query
    }

    #[inline(always)]
    fn query_desc_mut(&mut self) -> &mut sys::ecs_query_desc_t {
        &mut self.desc.query
    }
    #[inline(always)]
    fn count_generic_terms(&self) -> i32 {
        T::COUNT
    }
}

impl<'a, T: QueryTuple> TermBuilderImpl<'a> for SystemBuilder<'a, T> {}

impl<'a, T: QueryTuple> QueryBuilderImpl<'a> for SystemBuilder<'a, T> {}

impl<'a, T> Builder<'a> for SystemBuilder<'a, T>
where
    T: QueryTuple,
{
    type BuiltType = System<'a>;

    /// Build the `system_builder` into an system
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        let system = System::new(self.world(), self.desc, self.is_instanced);
        for string_parts in self.term_builder.str_ptrs_to_free.iter() {
            unsafe {
                String::from_raw_parts(
                    string_parts.ptr as *mut u8,
                    string_parts.len,
                    string_parts.capacity,
                );
            }
        }
        system
    }
}

impl<'a, T: QueryTuple> WorldProvider<'a> for SystemBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

implement_reactor_api!((), SystemBuilder<'a, T>);
