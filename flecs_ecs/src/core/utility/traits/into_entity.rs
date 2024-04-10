use crate::core::{
    ecs_pair, Component, ComponentId, Entity, EntityId, EntityView, Id, IdT, UntypedComponent,
};

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

impl<'a> IntoEntityId for Id<'a> {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl<'a> IntoEntityId for EntityView<'a> {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl<'a> IntoEntityId for Entity<'a> {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl<'a, T> IntoEntityId for Component<'a, T>
where
    T: ComponentId,
{
    #[inline]
    fn get_id(&self) -> u64 {
        self.base.entity.raw_id
    }
}

impl<'a> IntoEntityId for UntypedComponent<'a> {
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

    fn get_id(&self) -> u64;

    /// This will return the id of the first part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id.
    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first(&self) -> u64 {
        self.get_id()
    }

    /// This will return the id of the second part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id.
    #[doc(hidden)]
    #[inline]
    fn get_id_second(&self) -> u64 {
        self.get_id()
    }
}

impl<T, U> IntoEntityIdExt for (T, U)
where
    T: IntoEntityId,
    U: IntoEntityId,
{
    const IS_PAIR: bool = true;

    #[inline]
    fn get_id(&self) -> u64 {
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
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for EntityId {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl<'a> IntoEntityIdExt for Id<'a> {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl<'a> IntoEntityIdExt for EntityView<'a> {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl<'a> IntoEntityIdExt for Entity<'a> {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl<'a, T> IntoEntityIdExt for Component<'a, T>
where
    T: ComponentId,
{
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        self.base.entity.raw_id
    }
}

impl<'a> IntoEntityIdExt for UntypedComponent<'a> {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        self.entity.raw_id
    }
}

impl<T> IntoEntityIdExt for &T
where
    T: IntoEntityIdExt,
{
    const IS_PAIR: bool = T::IS_PAIR;

    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

impl<T> IntoEntityIdExt for &mut T
where
    T: IntoEntityIdExt,
{
    const IS_PAIR: bool = T::IS_PAIR;

    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}
