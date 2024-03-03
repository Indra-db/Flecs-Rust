//! Pipelines order and schedule systems for execution.

mod pipeline_builder;

pub use pipeline_builder::*;

use std::ops::{Deref, DerefMut};

use crate::{
    core::{
        c_binding::{ecs_os_api, ecs_pipeline_desc_t, ecs_pipeline_init},
        Entity, FlecsErrorCode, Iterable, World,
    },
    ecs_abort,
};

/// Pipelines order and schedule systems for execution.
pub struct Pipeline<'a, T>
where
    T: Iterable<'a>,
{
    pub entity: Entity,
    phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Deref for Pipeline<'a, T>
where
    T: Iterable<'a>,
{
    type Target = Entity;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a, T> DerefMut for Pipeline<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<'a, T> Pipeline<'a, T>
where
    T: Iterable<'a>,
{
    /// Create a new pipeline.
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the pipeline in.
    /// * `desc` - The pipeline description.
    ///
    /// # See also
    ///
    /// * C++ API: `pipeline::pipeline`
    #[doc(alias = "pipeline::pipeline")]
    pub fn new(world: &World, desc: ecs_pipeline_desc_t) -> Self {
        let entity = Entity::new(world);
        let mut pipeline = Self {
            entity,
            phantom: Default::default(),
        };
        pipeline.raw_id = unsafe { ecs_pipeline_init(world.raw_world, &desc) };

        if pipeline.raw_id == 0 {
            ecs_abort!(FlecsErrorCode::InvalidParameter);
        }

        if !desc.query.filter.terms_buffer.is_null() {
            unsafe {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func(desc.query.filter.terms_buffer as *mut _);
                }
            };
        }
        pipeline
    }
}
