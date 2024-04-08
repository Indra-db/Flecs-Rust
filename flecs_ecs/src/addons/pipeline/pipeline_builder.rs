use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
};

use flecs_ecs_sys::{ecs_filter_desc_t, ecs_query_desc_t};

use crate::{
    core::{
        Builder, EntityT, FilterBuilderImpl, Filterable, Iterable, QueryBuilder, QueryBuilderImpl,
        Term, TermBuilder, TermIdT, TermT, World, WorldT, SEPARATOR,
    },
    sys::{ecs_entity_desc_t, ecs_entity_init, ecs_pipeline_desc_t},
};

use super::Pipeline;

pub struct PipelineBuilder<T>
where
    T: Iterable,
{
    query_builder: QueryBuilder<T>,
    desc: ecs_pipeline_desc_t,
    is_instanced: bool,
}

/// Deref to `QueryBuilder` to allow access to `QueryBuilder` methods without having to access `QueryBuilder` through `PipelineBuilder`
impl<T> Deref for PipelineBuilder<T>
where
    T: Iterable,
{
    type Target = QueryBuilder<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.query_builder
    }
}

impl<T> DerefMut for PipelineBuilder<T>
where
    T: Iterable,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query_builder
    }
}

impl<T> PipelineBuilder<T>
where
    T: Iterable,
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
        obj
    }

    pub fn new_entity(world: &World, entity: EntityT) -> Self {
        let mut obj = Self::new(world);
        obj.desc.entity = entity;
        obj
    }

    pub fn new_from_desc(world: &World, mut desc: ecs_pipeline_desc_t) -> Self {
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
        obj
    }

    pub fn new_from_desc_term_index(
        world: &World,
        mut desc: ecs_pipeline_desc_t,
        term_index: i32,
    ) -> Self {
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::<T>::new_from_desc_term_index(
                world,
                &mut desc.query,
                term_index,
            ),
            is_instanced: false,
        };
        T::populate(&mut obj);
        obj
    }

    //TODO fix this - not working as intended most likely
    pub fn new_named(world: &World, name: &CStr) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            query_builder: QueryBuilder::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };

        let entity_desc: ecs_entity_desc_t = ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { ecs_entity_init(obj.world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }
}
impl<T> Filterable for PipelineBuilder<T>
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

impl<T> TermBuilder for PipelineBuilder<T>
where
    T: Iterable,
{
    #[inline]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.filter_builder.world.raw_world
    }

    #[inline]
    fn term_mut(&mut self) -> &mut Term {
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

impl<T> Builder for PipelineBuilder<T>
where
    T: Iterable,
{
    type BuiltType = Pipeline<T>;

    fn build(&mut self) -> Self::BuiltType {
        Pipeline::<T>::new(&self.world, self.desc)
    }
}

impl<T> FilterBuilderImpl for PipelineBuilder<T>
where
    T: Iterable,
{
    #[inline]
    fn desc_filter_mut(&mut self) -> &mut ecs_filter_desc_t {
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

impl<T> QueryBuilderImpl for PipelineBuilder<T>
where
    T: Iterable,
{
    #[inline]
    fn desc_query_mut(&mut self) -> &mut ecs_query_desc_t {
        &mut self.desc.query
    }
}
