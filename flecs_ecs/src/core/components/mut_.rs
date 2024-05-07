use std::ops::{Deref, DerefMut};

use flecs_ecs::core::*;
use flecs_ecs_sys as sys;
use flecs_ecs_sys::ecs_record_t;

#[derive(Debug)]
pub struct Mut<'a, T: ComponentId> {
    ref_: &'a mut T::UnderlyingType,
    record: *const ecs_record_t,
    world: WorldRef<'a>,
}

impl<'a, T: ComponentId> Mut<'a, T> {
    #[inline(always)]
    pub(crate) fn new(
        ref_: &'a mut T::UnderlyingType,
        record: *const ecs_record_t,
        world: WorldRef<'a>,
    ) -> Self {
        Self {
            ref_,
            record,
            world,
        }
    }
}

impl<'a, T: ComponentId> Deref for Mut<'a, T> {
    type Target = T::UnderlyingType;

    #[inline(always)]
    fn deref(&self) -> &'_ Self::Target {
        self.ref_
    }
}

impl<'a, T: ComponentId> DerefMut for Mut<'a, T> {
    #[inline(always)]
    fn deref_mut(&'_ mut self) -> &'_ mut Self::Target {
        self.ref_
    }
}

impl<'a, T: ComponentId> Drop for Mut<'a, T> {
    #[inline(always)]
    fn drop(&mut self) {
        if !self.record.is_null() {
            unsafe { sys::ecs_table_unlock(self.world.world_ptr_mut(), (*self.record).table) }
        }
    }
}
