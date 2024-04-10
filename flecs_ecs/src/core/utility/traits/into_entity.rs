use crate::core::*;

/// Extracts the Ecs ID from a type.
/// `IntoEntity` encapsultes the logic of extracting the entity id from a type.
/// These types can be `EntityView`, `Entity`, `Component`, `UntypedComponent`.
/// `IdView` is not part of that list, because it can represent a pair as well.
/// This is to allow a more safe API, where you can't accidentally pass a pair where a single id is expected.
///
/// See also: [`IntoId`]
pub trait IntoEntity {
    fn get_id(&self) -> u64;
}

impl IntoEntity for IdT {
    #[inline]
    fn get_id(&self) -> IdT {
        *self
    }
}

impl<'a> IntoEntity for EntityView<'a> {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl<'a, T> IntoEntity for Component<'a, T>
where
    T: ComponentId,
{
    #[inline]
    fn get_id(&self) -> u64 {
        self.base.entity.raw_id
    }
}

impl<'a> IntoEntity for UntypedComponent<'a> {
    #[inline]
    fn get_id(&self) -> u64 {
        self.entity.raw_id
    }
}

impl<T> IntoEntity for &T
where
    T: IntoEntity,
{
    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

impl<T> IntoEntity for &mut T
where
    T: IntoEntity,
{
    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

/// Extracts the Ecs ID from a type.
/// Extension trait from [`IntoEntity`] for tuples that implement `IntoEntity`.
/// These types can be `IdView`, `EntityView`, `Entity`, `Component`, `UntypedComponent`.
pub trait IntoId {
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

impl<T, U> IntoId for (T, U)
where
    T: IntoEntity,
    U: IntoEntity,
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

// We can not implement for T where T : `IntoEntity`, because it would essentially extend the trait, which we don't want
// so we have to implement for each type that implements `IntoEntity` separately.

impl IntoId for IdT {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        *self
    }
}

impl<'a> IntoId for IdView<'a> {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl<'a> IntoId for EntityView<'a> {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl<'a, T> IntoId for Component<'a, T>
where
    T: ComponentId,
{
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        self.base.entity.raw_id
    }
}

impl<'a> IntoId for UntypedComponent<'a> {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        self.entity.raw_id
    }
}

impl<T> IntoId for &T
where
    T: IntoId,
{
    const IS_PAIR: bool = T::IS_PAIR;

    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

impl<T> IntoId for &mut T
where
    T: IntoId,
{
    const IS_PAIR: bool = T::IS_PAIR;

    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}
