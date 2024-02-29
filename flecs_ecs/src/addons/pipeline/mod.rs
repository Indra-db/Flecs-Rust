pub mod pipeline_builder;

pub use pipeline_builder::*;

use std::ops::{Deref, DerefMut};

use crate::{
    core::{
        ecs_pipeline_desc_t, ecs_pipeline_init, Entity, Iterable, World, ecs_os_api,
        FlecsErrorCode,
    },
    ecs_abort,
};

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
                    free_func(desc.query.filter.terms_buffer as *mut _)
                }
            };
        }
        pipeline
    }
}
