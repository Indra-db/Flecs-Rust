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
        let current_phase: EntityT = unsafe {
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
        self.kind_id(Phase::get_id(self.world()))
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
        Phase: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        let enum_id = phase.get_id_variant(self.world());
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
        self.desc.tick_source = Component::get_id(self.world());
        self
    }
}

impl<'a, T> Filterable<'a> for SystemBuilder<'a, T>
where
    T: Iterable,
{
    fn current_term(&mut self) -> &mut TermT {
        self.get_current_term_mut()
    }

    fn increment_current_term(&mut self) {
        self.query_builder.increment_current_term();
    }
}

impl<'a, T> TermBuilder<'a> for SystemBuilder<'a, T>
where
    T: Iterable,
{
    fn current_term_ref_mode(&self) -> TermRefMode {
        self.query_builder.current_term_ref_mode()
    }

    fn set_term_ref_mode(&mut self, mode: TermRefMode) {
        self.query_builder.set_term_ref_mode(mode);
    }

    fn get_term_mut_at(&mut self, index: i32) -> &mut TermT {
        &mut self.desc.query.terms[index as usize]
    }

    fn get_current_term_mut(&mut self) -> &mut TermT {
        let index = self.current_term_index();
        &mut self.desc.query.terms[index as usize]
    }

    fn get_current_term(&self) -> &TermT {
        let index = self.current_term_index();
        &self.desc.query.terms[index as usize]
    }

    fn term_ref_mut(&mut self) -> &mut TermRefT {
        let term_mode = self.current_term_ref_mode();
        let term = self.get_current_term_mut();

        match term_mode {
            TermRefMode::Src => &mut term.src,
            TermRefMode::First => &mut term.first,
            TermRefMode::Second => &mut term.second,
        }
    }
}

impl<'a, T> QueryBuilderImpl<'a> for SystemBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn query_desc_mut(&mut self) -> &mut sys::ecs_query_desc_t {
        print!("system desc");
        &mut self.desc.query
    }

    #[inline]
    fn expr_count_mut(&mut self) -> &mut i32 {
        &mut self.query_builder.expr_count
    }

    #[inline]
    fn current_term_index_mut(&mut self) -> &mut i32 {
        self.query_builder.current_term_index_mut()
    }

    fn query_desc(&self) -> &sys::ecs_query_desc_t {
        print!("system desc");
        &self.desc.query
    }

    fn current_term_index(&self) -> i32 {
        self.query_builder.current_term_index()
    }

    fn next_term_index(&self) -> i32 {
        self.query_builder.next_term_index()
    }

    fn next_term_index_mut(&mut self) -> &mut i32 {
        self.query_builder.next_term_index_mut()
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
        System::new(self.world(), self.desc, self.is_instanced)
    }
}

impl<'a, T: Iterable> IntoWorld<'a> for SystemBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.query_builder.world()
    }
}

implement_reactor_api!(SystemBuilder<'a, T>);
