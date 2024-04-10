use std::sync::OnceLock;
use std::{ffi::CStr, ops::Deref};

use crate::core::*;
use crate::sys;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy, Hash)]
pub struct Entity {
    id: u64,
}

impl Entity {
    #[inline]
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Convert the entity id to an entity with the given world.
    ///
    /// # Safety
    ///
    /// This entity is safe to do operations on if the entity belongs to the world
    ///
    /// # Arguments
    ///
    /// * `world` - The world the entity belongs to
    pub fn to_entity<'a>(&self, world: impl IntoWorld<'a>) -> EntityView<'a> {
        EntityView::new_from(world, self.id)
    }
}

impl Deref for Entity {
    type Target = IdT;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl ComponentInfo for Entity {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = false;
}

impl ComponentId for Entity {
    type UnderlyingType = Entity;
    type UnderlyingEnumType = NoneEnum;

    fn register_explicit<'a>(_world: impl IntoWorld<'a>) {
        // already registered by flecs in World
    }

    fn register_explicit_named<'a>(_world: impl IntoWorld<'a>, _name: &CStr) -> EntityT {
        // already registered by flecs in World
        unsafe { sys::FLECS_IDecs_entity_tID_ }
    }

    fn is_registered() -> bool {
        true
    }

    fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
        //because this is always registered in the c world
        true
    }

    unsafe fn get_id_unchecked() -> IdT {
        //this is safe because it's already registered in flecs_c / world
        sys::FLECS_IDecs_entity_tID_
    }

    fn get_id<'a>(_world: impl IntoWorld<'a>) -> IdT {
        //this is safe because it's already registered in flecs_c / world
        unsafe { sys::FLECS_IDecs_entity_tID_ }
    }

    fn __get_once_lock_data() -> &'static OnceLock<IdComponent> {
        static ONCE_LOCK: OnceLock<IdComponent> = OnceLock::new();
        &ONCE_LOCK
    }
}

pub struct Id {
    id: u64,
}

impl Id {
    #[inline]
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Convert the entity id to an entity with the given world.
    ///
    /// # Safety
    ///
    /// This entity is safe to do operations on if the entity belongs to the world
    ///
    /// # Arguments
    ///
    /// * `world` - The world the entity belongs to
    pub fn to_entity<'a>(&self, world: impl IntoWorld<'a>) -> EntityView<'a> {
        EntityView::new_from(world, self.id)
    }
}

impl Deref for Id {
    type Target = IdT;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}
