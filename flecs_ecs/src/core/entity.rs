use std::fmt::Display;
use std::ops::{BitAnd, BitOr};
use std::sync::OnceLock;
use std::{ffi::CStr, ops::Deref};

use crate::core::*;
use crate::sys;

/// An Identifier for what represents an entity.
/// An `Entity` is an id that represents either an entity, component, query, observer or system in the world.
/// an `Entity` does not represent a pair. See [`Id`] for that.
/// Entity ids consist out of a number unique to the entity in the lower 32 bits,
/// and a counter used to track entity liveliness in the upper 32 bits. When an
/// id is recycled, its generation count is increased. This causes recycled ids
/// to be very large (>4 billion), which is normal.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Entity(pub u64);

impl Entity {
    #[inline]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Convert the entity id to an [`EntityView`] with the given world.
    ///
    /// # Safety
    ///
    /// This entity is safe to do operations on if the entity belongs to the world
    ///
    /// # Arguments
    ///
    /// * `world` - The world the entity belongs to
    pub fn entity_view<'a>(&self, world: impl IntoWorld<'a>) -> EntityView<'a> {
        EntityView::new_from(world, *self)
    }

    /// Convert the entity id to an [`IdView`] with the given world.
    ///
    /// # Safety
    ///
    /// This entity is safe to do operations on if the entity belongs to the world
    ///
    /// # Arguments
    ///
    /// * `world` - The world the entity belongs to
    pub fn id_view<'a>(&self, world: impl IntoWorld<'a>) -> IdView<'a> {
        IdView::new_from(world, *self)
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

impl Deref for Entity {
    type Target = u64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BitOr for Entity {
    type Output = Entity;

    fn bitor(self, rhs: Self) -> Self::Output {
        Entity(self.0 | rhs.0)
    }
}

impl BitOr<u64> for Entity {
    type Output = Entity;

    fn bitor(self, rhs: u64) -> Self::Output {
        Entity(self.0 | rhs)
    }
}

impl BitOr<Id> for Entity {
    type Output = Entity;

    fn bitor(self, rhs: Id) -> Self::Output {
        Entity(self.0 | *rhs)
    }
}

impl BitOr<Entity> for u64 {
    type Output = Entity;

    fn bitor(self, rhs: Entity) -> Self::Output {
        Entity(self | rhs.0)
    }
}

impl BitAnd for Entity {
    type Output = Entity;

    fn bitand(self, rhs: Self) -> Self::Output {
        Entity(self.0 & rhs.0)
    }
}

impl BitAnd<u64> for Entity {
    type Output = Entity;

    fn bitand(self, rhs: u64) -> Self::Output {
        Entity(self.0 & rhs)
    }
}

impl BitAnd<Id> for Entity {
    type Output = Entity;

    fn bitand(self, rhs: Id) -> Self::Output {
        Entity(self.0 & *rhs)
    }
}

impl From<u64> for Entity {
    #[inline]
    fn from(id: u64) -> Self {
        Entity::new(id)
    }
}

impl<'a> From<EntityView<'a>> for Entity {
    #[inline]
    fn from(view: EntityView<'a>) -> Self {
        view.id
    }
}

impl<'a, T> From<Component<'a, T>> for Entity
where
    T: ComponentId,
{
    #[inline]
    fn from(component: Component<'a, T>) -> Self {
        component.base.entity.id
    }
}

impl<'a> From<UntypedComponent<'a>> for Entity {
    #[inline]
    fn from(component: UntypedComponent<'a>) -> Self {
        component.entity.id
    }
}

impl From<Entity> for u64 {
    fn from(id: Entity) -> Self {
        id.0
    }
}

impl PartialEq<Entity> for u64 {
    fn eq(&self, other: &Entity) -> bool {
        self == &other.0
    }
}

impl PartialEq<u64> for Entity {
    fn eq(&self, other: &u64) -> bool {
        &self.0 == other
    }
}

impl PartialEq<Id> for Entity {
    fn eq(&self, other: &Id) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<EntityView<'a>> for Entity {
    fn eq(&self, other: &EntityView<'a>) -> bool {
        self.0 == other.id.0
    }
}

impl<'a> PartialEq<IdView<'a>> for Entity {
    fn eq(&self, other: &IdView<'a>) -> bool {
        self.0 == other.id.0
    }
}

impl<'a, T> PartialEq<Component<'a, T>> for Entity
where
    T: ComponentId,
{
    fn eq(&self, other: &Component<'a, T>) -> bool {
        self.0 == other.base.entity.id.0
    }
}

impl<'a> PartialEq<UntypedComponent<'a>> for Entity {
    fn eq(&self, other: &UntypedComponent<'a>) -> bool {
        self.0 == other.entity.id.0
    }
}

impl PartialOrd<Entity> for u64 {
    fn partial_cmp(&self, other: &Entity) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.0))
    }
}

impl PartialOrd<u64> for Entity {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(other))
    }
}

impl PartialOrd<Id> for Entity {
    fn partial_cmp(&self, other: &Id) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<'a> PartialOrd<EntityView<'a>> for Entity {
    fn partial_cmp(&self, other: &EntityView<'a>) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.id.0))
    }
}

impl<'a> PartialOrd<IdView<'a>> for Entity {
    fn partial_cmp(&self, other: &IdView<'a>) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.id.0))
    }
}

impl<'a, T> PartialOrd<Component<'a, T>> for Entity
where
    T: ComponentId,
{
    fn partial_cmp(&self, other: &Component<'a, T>) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.base.entity.id.0))
    }
}

impl<'a> PartialOrd<UntypedComponent<'a>> for Entity {
    fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.entity.id.0))
    }
}

impl Display for Entity {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
