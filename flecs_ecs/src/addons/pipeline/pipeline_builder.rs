//! Pipeline builder used to configure and build Pipelines.

use super::Pipeline;
use crate::core::internals::*;
use crate::core::*;
use crate::sys;

/// Pipeline builder used to configure and build Pipelines.
/// Pipelines order and schedule systems for execution.
pub struct PipelineBuilder<'a, T>
where
    T: Iterable,
{
    desc: sys::ecs_pipeline_desc_t,
    term_builder: TermBuilder,
    world: WorldRef<'a>,
    is_instanced: bool,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> PipelineBuilder<'a, T>
where
    T: Iterable,
{
    /// Create a new pipeline builder
    pub fn new(world: &'a World) -> Self {
        let desc = Default::default();
        let mut obj = Self {
            desc,
            term_builder: TermBuilder::default(),
            world: world.into(),
            is_instanced: false,
            _phantom: std::marker::PhantomData,
        };

        obj.desc.entity =
            unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &Default::default()) };

        T::populate(&mut obj);
        obj
    }

    /// Create a new pipeline builder with an associated entity
    pub fn new_w_entity(world: &'a World, entity: impl Into<Entity>) -> Self {
        let mut obj = Self::new(world);
        obj.desc.entity = *entity.into();
        obj
    }

    pub(crate) fn new_from_desc(world: &'a World, desc: sys::ecs_pipeline_desc_t) -> Self {
        let mut obj = Self {
            desc,
            term_builder: TermBuilder::default(),
            world: world.into(),
            is_instanced: false,
            _phantom: std::marker::PhantomData,
        };

        if obj.desc.entity == 0 {
            obj.desc.entity =
                unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &Default::default()) };
        }

        T::populate(&mut obj);
        obj
    }

    pub(crate) fn new_from_desc_term_index(
        world: &'a World,
        desc: sys::ecs_pipeline_desc_t,
        term_index: i32,
    ) -> Self {
        let mut obj = Self {
            desc,
            term_builder: TermBuilder {
                current_term_index: term_index,
                next_term_index: term_index,
                ..Default::default()
            },

            world: world.into(),
            is_instanced: false,
            _phantom: std::marker::PhantomData,
        };

        if obj.desc.entity == 0 {
            obj.desc.entity =
                unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &Default::default()) };
        }

        T::populate(&mut obj);
        obj
    }

    /// Create a new pipeline builder with a name
    pub fn new_named(world: &'a World, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let mut obj = Self {
            desc: Default::default(),
            term_builder: TermBuilder::default(),
            world: world.into(),
            is_instanced: false,
            _phantom: std::marker::PhantomData,
        };

        let entity_desc: sys::ecs_entity_desc_t = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const i8,
            sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { sys::ecs_entity_init(obj.world_ptr_mut(), &entity_desc) };

        T::populate(&mut obj);
        obj
    }
}

#[doc(hidden)]
impl<'a, T: Iterable> internals::QueryConfig<'a> for PipelineBuilder<'a, T> {
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

impl<'a, T: Iterable> QueryBuilderImpl<'a> for PipelineBuilder<'a, T> {}
impl<'a, T: Iterable> TermBuilderImpl<'a> for PipelineBuilder<'a, T> {}

impl<'a, T> Builder<'a> for PipelineBuilder<'a, T>
where
    T: Iterable,
{
    type BuiltType = Pipeline<'a, T>;

    fn build(&mut self) -> Self::BuiltType {
        let pipeline = Pipeline::<T>::new(self.world(), self.desc);
        for string_parts in self.term_builder.str_ptrs_to_free.iter() {
            unsafe {
                String::from_raw_parts(
                    string_parts.ptr as *mut u8,
                    string_parts.len,
                    string_parts.capacity,
                );
            }
        }
        pipeline
    }
}

impl<'a, T: Iterable> IntoWorld<'a> for PipelineBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}
