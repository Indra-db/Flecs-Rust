use std::borrow::Borrow;

use crate::core::{
    c_types::{InOutKind, OperKind},
    component_registration::ComponentInfo,
    Entity, EntityView, Id, IdT, World, WorldT,
};

use super::{ecs_pair, EntityId};

/// Represents the input/output type of a component in an ECS system.
///
/// This trait defines the kind of access (input, output, or both) that an ECS system has
/// to a component. Implementing this trait allows specifying whether a component is read,
/// written, or both by a system. This categorization helps in optimizing access patterns
/// and maintaining data consistency within the ECS framework.
///
/// # Associated Constants
///
/// * `IN_OUT`: The kind of access (`InOutKind`) the system has to the component.
///
/// # Associated Types
///
/// * `Type`: The type of the component data. Must implement `ComponentInfo`.
pub trait InOutType {
    const IN_OUT: InOutKind;
    type Type: ComponentInfo;
}

/// Represents the operation type of a system in an ECS framework.
///
/// This trait is used to specify the kind of operation a system performs on a component,
/// such as adding, removing, or setting a component. Implementing this trait allows the ECS
/// framework to understand and optimize the execution of systems based on their operational
/// characteristics.
///
/// # Associated Constants
///
/// * `OPER`: The kind of operation (`OperKind`) the system performs.
///
/// # Associated Types
///
/// * `Type`: The type of the component data. Must implement `ComponentInfo`.
pub trait OperType {
    const OPER: OperKind;
    type Type: ComponentInfo;
}

/// Extracts the entity id from a type.
pub trait IntoEntityId {
    fn get_id(&self) -> u64;
}

impl IntoEntityId for IdT {
    #[inline]
    fn get_id(&self) -> IdT {
        *self
    }
}

impl IntoEntityId for EntityId {
    #[inline]
    fn get_id(&self) -> u64 {
        self.0
    }
}

impl IntoEntityId for Id {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl IntoEntityId for EntityView {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl IntoEntityId for Entity {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl<T> IntoEntityId for &T
where
    T: IntoEntityId,
{
    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

impl<T> IntoEntityId for &mut T
where
    T: IntoEntityId,
{
    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

/// Extension trait for tuples that implement `IntoEntityId`.
/// This extension is useful for when some function only expect one entity id, but not pairs of them.
/// so you only accept `IntoEntityId`. Where both pairs and a single id are accepted, you can use `IntoEntityIdExt`.
pub trait IntoEntityIdExt {
    const IS_PAIR: bool;

    fn get_id_ext(&self) -> u64;
}

impl<T, U> IntoEntityIdExt for (T, U)
where
    T: IntoEntityId,
    U: IntoEntityId,
{
    const IS_PAIR: bool = true;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        ecs_pair(self.0.get_id(), self.1.get_id())
    }
}

// We can not implement for T where T : IntoEntityId, because it would essentially extend the trait, which we don't want
// so we have to implement for each type that implements IntoEntityId separately.

impl IntoEntityIdExt for IdT {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for EntityId {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for Id {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for EntityView {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for Entity {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl<T> IntoEntityIdExt for &T
where
    T: IntoEntityIdExt,
{
    const IS_PAIR: bool = T::IS_PAIR;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        T::get_id_ext(*self)
    }
}

impl<T> IntoEntityIdExt for &mut T
where
    T: IntoEntityIdExt,
{
    const IS_PAIR: bool = T::IS_PAIR;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        T::get_id_ext(*self)
    }
}

pub trait IntoWorld {
    fn get_world_raw_mut(&self) -> *mut WorldT;
    #[inline]
    fn get_world_raw(&self) -> *const WorldT {
        self.get_world_raw_mut() as *const WorldT
    }
}

impl IntoWorld for *mut WorldT {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        *self
    }
}

impl IntoWorld for *const WorldT {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        *self as *mut WorldT
    }
}

impl IntoWorld for World {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.raw_world
    }
}

impl IntoWorld for Id {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl IntoWorld for Entity {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl IntoWorld for EntityView {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl<T> IntoWorld for &T
where
    T: IntoWorld,
{
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        T::get_world_raw_mut(*self)
    }
}

impl<T> IntoWorld for &mut T
where
    T: IntoWorld,
{
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        T::get_world_raw_mut(*self)
    }
}

impl<T> IntoWorld for Option<T>
where
    T: IntoWorld,
{
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        match self {
            Some(t) => t.get_world_raw_mut(),
            None => std::ptr::null_mut(),
        }
    }
}

// set_override_pair_second
