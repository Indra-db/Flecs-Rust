use crate::core::{
    c_types::{InOutKind, OperKind},
    component_registration::ComponentInfo,
    Component, ComponentType, Entity, EntityView, Id, IdT, Struct, Table, TableRange, TableT,
    UntypedComponent, World, WorldT,
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
    type Type: IntoComponentId;
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

impl<T> IntoEntityId for Component<T>
where
    T: ComponentInfo,
{
    #[inline]
    fn get_id(&self) -> u64 {
        self.base.entity.raw_id
    }
}

impl IntoEntityId for UntypedComponent {
    #[inline]
    fn get_id(&self) -> u64 {
        self.entity.raw_id
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

    /// This will return the id of the first part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id_ext.
    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first(&self) -> u64 {
        self.get_id_ext()
    }

    /// This will return the id of the second part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id_ext.
    #[doc(hidden)]
    #[inline]
    fn get_id_second(&self) -> u64 {
        self.get_id_ext()
    }
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

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first(&self) -> u64 {
        self.0.get_id()
    }

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_second(&self) -> u64 {
        self.1.get_id()
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

impl<T> IntoEntityIdExt for Component<T>
where
    T: ComponentInfo,
{
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        self.base.entity.raw_id
    }
}

impl IntoEntityIdExt for UntypedComponent {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id_ext(&self) -> u64 {
        self.entity.raw_id
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

pub trait IntoComponentId {
    const IS_ENUM: bool;
    const IS_PAIR: bool;
    // These types are useful for merging functions in World class such ass add_pair<T,U> into add<T>.
    // When IntoComponentId is not a pair, First and Second will be same
    type First: ComponentInfo;
    type Second: ComponentInfo;

    fn get_id(world: impl IntoWorld) -> IdT;

    /// Get the symbol name of the component.
    ///
    /// # Safety
    ///
    /// Notice that this function for pairs (T, U) will return the type name of the tuple, not the individual components.
    /// This isn't a name stored in the ECS unlike a singular component.
    fn get_name() -> &'static str;
}

impl<T> IntoComponentId for T
where
    T: ComponentInfo,
{
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_PAIR: bool = false;
    type First = T;
    type Second = T;

    #[inline]
    fn get_id(world: impl IntoWorld) -> IdT {
        T::get_id(world.get_world_raw_mut())
    }

    #[inline]
    fn get_name() -> &'static str {
        T::get_symbol_name()
    }
}

impl<T, U> IntoComponentId for (T, U)
where
    T: ComponentInfo,
    U: ComponentInfo + ComponentType<Struct>,
{
    const IS_ENUM: bool = false;
    const IS_PAIR: bool = true;
    type First = T;
    type Second = U;

    #[inline]
    fn get_id(world: impl IntoWorld) -> IdT {
        ecs_pair(
            T::get_id(world.get_world_raw_mut()),
            U::get_id(world.get_world_raw_mut()),
        )
    }

    #[inline]
    fn get_name() -> &'static str {
        std::any::type_name::<(T, U)>()
    }
}

pub trait IntoTable {
    fn get_table(&self) -> *mut TableT;
}

impl IntoTable for *mut TableT {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        *self
    }
}

impl IntoTable for *const TableT {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        *self as *mut TableT
    }
}

impl IntoTable for Table {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        self.get_raw_table()
    }
}

impl IntoTable for TableRange {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        self.table.get_raw_table()
    }
}
