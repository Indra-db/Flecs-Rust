//! Pipelines order and schedule systems for execution.

mod pipeline_builder;
pub use pipeline_builder::*;

use std::ops::{Deref, DerefMut};

use crate::core::*;
use crate::sys;

/// Pipelines order and schedule systems for execution.
pub struct Pipeline<'a, T>
where
    T: Iterable,
{
    pub entity: EntityView<'a>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> Deref for Pipeline<'a, T>
where
    T: Iterable,
{
    type Target = EntityView<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a, T> DerefMut for Pipeline<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<'a, T> From<Pipeline<'a, T>> for EntityView<'a>
where
    T: Iterable,
{
    fn from(pipeline: Pipeline<'a, T>) -> Self {
        pipeline.entity
    }
}

impl<'a, T> Pipeline<'a, T>
where
    T: Iterable,
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
    pub fn new(world: impl IntoWorld<'a>, desc: sys::ecs_pipeline_desc_t) -> Self {
        let entity = EntityView::new(world.world());
        let mut pipeline = Self {
            entity,
            phantom: Default::default(),
        };
        pipeline.id = Entity(unsafe { sys::ecs_pipeline_init(world.world_ptr_mut(), &desc) });

        if pipeline.id == 0 {
            ecs_abort!(FlecsErrorCode::InvalidParameter);
        }

        if !desc.query.filter.terms_buffer.is_null() {
            unsafe {
                if let Some(free_func) = sys::ecs_os_api.free_ {
                    free_func(desc.query.filter.terms_buffer as *mut _);
                }
            };
        }
        pipeline
    }
}
