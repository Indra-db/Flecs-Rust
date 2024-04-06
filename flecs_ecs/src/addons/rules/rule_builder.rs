use std::{ffi::CStr, ops::Deref};

use flecs_ecs_sys::{ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t};

use crate::core::{
    Builder, FilterBuilder, FilterBuilderImpl, Filterable, Iterable, Term, TermBuilder, TermIdT,
    TermT, World, WorldT, SEPARATOR,
};

use super::Rule;

pub struct RuleBuilder<'a, T>
where
    T: Iterable<'a>,
{
    pub filter_builder: FilterBuilder<'a, T>,
}

impl<'a, T> Deref for RuleBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type Target = FilterBuilder<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.filter_builder
    }
}

impl<'a, T> RuleBuilder<'a, T>
where
    T: Iterable<'a>,
{
    /// Create a new query builder
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    ///
    /// See also
    ///
    /// * C++ API: `builder::builder`
    #[doc(alias = "builder::builder")]
    pub fn new(world: &World) -> Self {
        let mut obj = Self {
            filter_builder: FilterBuilder::new(world),
        };

        let entity_desc = ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.filter_builder.desc.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }

    /// Create a new query builder with a name
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `name` - The name of the observer
    ///
    /// See also
    ///
    /// * C++ API: `query_builder::query_builder`
    #[doc(alias = "query_builder::query_builder")]
    pub fn new_named(world: &World, name: &CStr) -> Self {
        let mut obj = Self {
            filter_builder: FilterBuilder::new_named(world, name),
        };

        let entity_desc = ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.filter_builder.desc.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }
}

impl<'a, T> Filterable for RuleBuilder<'a, T>
where
    T: Iterable<'a>,
{
    fn current_term(&mut self) -> &mut TermT {
        unsafe { &mut *self.filter_builder.term.term_ptr }
    }

    fn next_term(&mut self) {
        self.filter_builder.next_term();
    }
}

impl<'a, T> FilterBuilderImpl for RuleBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn desc_filter_mut(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.filter_builder.desc
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

impl<'a, T> TermBuilder for RuleBuilder<'a, T>
where
    T: Iterable<'a>,
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

impl<'a, T> Builder for RuleBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type BuiltType = Rule<'a, T>;

    /// Build the `observer_builder` into an query
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        let world = &self.filter_builder.world;
        Rule::<T>::new_from_desc(world, &mut self.filter_builder.desc)
    }
}
