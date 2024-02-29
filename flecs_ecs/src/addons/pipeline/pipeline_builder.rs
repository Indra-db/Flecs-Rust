use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
};

use crate::core::{
    ecs_entity_desc_t, ecs_entity_init, ecs_pipeline_desc_t, Builder, EntityT, FilterBuilderImpl,
    Filterable, Iterable, QueryBuilder, QueryBuilderImpl, Term, TermBuilder, TermIdT, TermT, World,
    WorldT, SEPARATOR,
};

use super::Pipeline;

pub struct PipelineBuilder<'a, T>
where
    T: Iterable<'a>,
{
    query_builder: QueryBuilder<'a, T>,
    desc: ecs_pipeline_desc_t,
    is_instanced: bool,
}

/// Deref to QueryBuilder to allow access to QueryBuilder methods without having to access QueryBuilder through PipelineBuilder
impl<'a, T> Deref for PipelineBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type Target = QueryBuilder<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.query_builder
    }
}

impl<'a, T> DerefMut for PipelineBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query_builder
    }
}

impl<'a, T> PipelineBuilder<'a, T>
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
        obj.desc.query = *obj.query_builder.get_desc_query();
        obj.desc.query.filter = *obj.filter_builder.get_desc_filter();
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
            desc: desc,
            query_builder: QueryBuilder::<T>::new_from_desc(world, &mut desc.query),
            is_instanced: false,
        };
        obj.desc.query = *obj.query_builder.get_desc_query();
        obj.desc.query.filter = *obj.filter_builder.get_desc_filter();
        T::populate(&mut obj);
        obj
    }

    pub fn new_from_desc_term_index(
        world: &World,
        mut desc: ecs_pipeline_desc_t,
        term_index: i32,
    ) -> Self {
        let mut obj = Self {
            desc: desc,
            query_builder: QueryBuilder::<T>::new_from_desc_term_index(
                world,
                &mut desc.query,
                term_index,
            ),
            is_instanced: false,
        };
        obj.desc.query = *obj.query_builder.get_desc_query();
        obj.desc.query.filter = *obj.filter_builder.get_desc_filter();
        T::populate(&mut obj);
        obj
    }

    pub fn new_named(world: &World, name: &CStr) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            query_builder: QueryBuilder::new_named(world, name),
            is_instanced: false,
        };
        T::populate(&mut obj);
        obj.desc.query = *obj.query_builder.get_desc_query();
        obj.desc.query.filter = *obj.filter_builder.get_desc_filter();
        let mut entity_desc: ecs_entity_desc_t = Default::default();
        entity_desc.name = name.as_ptr();
        entity_desc.sep = SEPARATOR.as_ptr() as *const i8;
        obj.desc.entity = unsafe { ecs_entity_init(obj.world.raw_world, &entity_desc) };
        obj
    }
}
impl<'a, T> Filterable for PipelineBuilder<'a, T>
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

impl<'a, T> TermBuilder for PipelineBuilder<'a, T>
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

impl<'a, T> Builder for PipelineBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type BuiltType = Pipeline<'a, T>;

    fn build(&mut self) -> Self::BuiltType {
        Pipeline::<T>::new(&self.world, self.desc)
    }
}
/*

impl<'a, T> FilterBuilderImpl for PipelineBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_filter(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.filter_builder.desc
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

impl<'a, T> QueryBuilderImpl for PipelineBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_query(&mut self) -> &mut ecs_query_desc_t {
        &mut self.desc.query
    }
}
*/
