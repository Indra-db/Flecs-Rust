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
    _phantom: core::marker::PhantomData<&'a T>,
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
            _phantom: core::marker::PhantomData,
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
            _phantom: core::marker::PhantomData,
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
            _phantom: core::marker::PhantomData,
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
    pub fn kind(&mut self, phase: impl IntoEntity) -> &mut Self {
        let phase = *phase.into_entity(self.world);
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

    /// Specify in which enum phase the system should run
    ///
    /// # Arguments
    ///
    /// * `phase` - the phase
    pub fn kind_enum<Phase>(&mut self, phase: Phase) -> &mut Self
    where
        Phase: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let enum_id = phase.id_variant(self.world());
        self.kind(enum_id)
    }

    /// Specify whether system can run on multiple threads.
    pub fn multi_threaded(&mut self) -> &mut Self {
        self.desc.multi_threaded = true;
        self
    }

    /// Set the system to not run on multiple threads.
    ///
    /// This is the default behavior. If not previously set through [`SystemBuilder::multi_threaded()`],
    /// then there is no need to call this method.
    pub fn disable_multi_threaded(&mut self) -> &mut Self {
        self.desc.multi_threaded = false;
        self
    }

    /// Specify whether system should be ran in staged context.
    ///
    /// # Arguments
    ///
    /// * `value` - If false,  system will always run staged.
    pub fn immediate(&mut self, value: bool) -> &mut Self {
        self.desc.immediate = value;
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
    fn build(&mut self) -> Self::BuiltType {
        let system = System::new(self.world(), self.desc);
        for s in self.term_builder.str_ptrs_to_free.iter_mut() {
            unsafe { core::mem::ManuallyDrop::drop(s) };
        }
        self.term_builder.str_ptrs_to_free.clear();
        system
    }
}

impl<'a, T: QueryTuple> WorldProvider<'a> for SystemBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

implement_reactor_api!((), SystemBuilder<'a, T>);
