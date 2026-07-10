use crate::core::*;

/// Extracts the Ecs ID a type.
/// Extension trait [`Into<Entity>`] for tuples that implement `Into<Entity>`.
/// These types can be [`Id`], [`IdView`], [`Entity`], [`EntityView`], [`Component`], [`UntypedComponent`].
pub trait IntoId: InternalIntoEntity
where
    Self: Sized,
{
    /// # Safety
    /// This is used to determine if the type is a pair or not.
    /// not if the underlying id represents a pair.
    /// This should generally not be used by the user.
    const IS_PAIR: bool;
    const IS_ENUM: bool;
    const IS_TYPE_TAG: bool;
    const IS_TYPED_REF: bool;
    const IS_TYPED_MUT_REF: bool;
    type CastType;

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn into_id<'a>(self, _world: impl WorldProvider<'a>) -> Id {
        Id(0)
    }

    /// This will return the id of the first part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id.
    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first<'a>(self, _world: impl WorldProvider<'a>) -> Entity {
        Entity(0)
    }

    /// This will return the id of the second part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id.
    #[doc(hidden)]
    #[inline]
    fn get_id_second<'a>(self, _world: impl WorldProvider<'a>) -> Entity {
        Entity::new(0)
    }

    /// This will return the id of the first and second part of a pair.
    /// If this is called on a non_pair, it will return the same id for both first and second.
    #[doc(hidden)]
    #[inline]
    fn get_id_first_second<'a>(&self, _world: impl WorldProvider<'a>) -> (Entity, Entity) {
        (Entity(0), Entity(0))
    }
}

impl<T: InternalIntoEntity> IntoId for T {
    const IS_PAIR: bool = T::IS_TYPED_PAIR;
    const IS_ENUM: bool = <T as InternalIntoEntity>::IS_ENUM;
    const IS_TYPE_TAG: bool = <T as InternalIntoEntity>::IS_TYPE_TAG;
    const IS_TYPED_REF: bool = <T as InternalIntoEntity>::IS_TYPED_REF;
    const IS_TYPED_MUT_REF: bool = <T as InternalIntoEntity>::IS_TYPED_MUT_REF;
    type CastType = T::CastType;

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn into_id<'a>(self, world: impl WorldProvider<'a>) -> Id {
        Id(*(self.into_entity(world)))
    }

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first<'a>(self, world: impl WorldProvider<'a>) -> Entity {
        let world = world.world();
        ecs_first(self.into_entity(world), world)
    }
    fn get_id_second<'a>(self, world: impl WorldProvider<'a>) -> Entity {
        let world = world.world();
        ecs_second(self.into_entity(world), world)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    None,
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessSingleTarget {
    /// A single `Entity`
    Entity(Entity),
    /// A single name
    Name(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessTarget<'s> {
    /// A single `Entity`
    Entity(Entity),
    /// A single name
    Name(&'s str),
    /// Two Entities
    Pair(Entity, Entity),
    /// Two names
    PairName(&'s str, &'s str),
    /// Name + Entity
    PairNameEntity(&'s str, Entity),
    /// Entity + Name
    PairEntityName(Entity, &'s str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access<'s> {
    pub mode: AccessMode,
    pub target: AccessTarget<'s>,
}

impl From<Access<'_>> for Id {
    fn from(access: Access<'_>) -> Id {
        match access.target {
            AccessTarget::Entity(entity) => entity.into(),
            AccessTarget::Pair(first, second) => Id(ecs_pair(*first, *second)),
            _ => panic!("AccessTarget {:?} not convertible to Id", access.target),
        }
    }
}

pub trait FromAccessArg<T>: Sized {
    fn from_access_arg<'a>(value: T, world: impl WorldProvider<'a>) -> Self;
}

/// “Only single‐target inputs” (Entity, &str, `Id<T>`, …)
pub trait SingleAccessArg<'s>: Sized
where
    Access<'s>: FromAccessArg<Self>,
{
}

/// for single or pair inputs
pub trait AccessArg<'s>: Sized
where
    Access<'s>: FromAccessArg<Self>,
{
}

impl<'s> SingleAccessArg<'s> for &'s str {}
impl<'s, T: IntoEntity> SingleAccessArg<'s> for T where Access<'s>: FromAccessArg<T> {}

impl<'s> AccessArg<'s> for &'s str {}
impl<'s, T: IntoId> AccessArg<'s> for T where Access<'s>: FromAccessArg<T> {}
impl<'s> AccessArg<'s> for (&'s str, &'s str) {}
impl<'s, T: IntoId> AccessArg<'s> for (&'s str, T) {}
impl<'s, T: IntoId> AccessArg<'s> for (T, &'s str) {}

impl<'s, T: ComponentId> FromAccessArg<&crate::core::utility::id::Id<T>> for Access<'s> {
    fn from_access_arg<'a>(
        id: &crate::core::utility::id::Id<T>,
        world: impl WorldProvider<'a>,
    ) -> Access<'s> {
        let entity = IntoId::into_id(*id, world);
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::Entity(Entity(*entity)),
        }
    }
}

impl<'s, T: ComponentId> FromAccessArg<&mut crate::core::utility::id::Id<T>> for Access<'s> {
    fn from_access_arg<'a>(
        id: &mut crate::core::utility::id::Id<T>,
        world: impl WorldProvider<'a>,
    ) -> Access<'s> {
        let entity = IntoId::into_id(*id, world);
        Access {
            mode: AccessMode::ReadWrite,
            target: AccessTarget::Entity(Entity(*entity)),
        }
    }
}

impl<'s> FromAccessArg<&'s str> for Access<'s> {
    fn from_access_arg<'a>(name: &'s str, _world: impl WorldProvider<'a>) -> Access<'s> {
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::Name(name),
        }
    }
}

impl<'s> FromAccessArg<(&'s str, &'s str)> for Access<'s> {
    fn from_access_arg<'a>(
        names: (&'s str, &'s str),
        _world: impl WorldProvider<'a>,
    ) -> Access<'s> {
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::PairName(names.0, names.1),
        }
    }
}

impl<'s, T> FromAccessArg<(&'s str, T)> for Access<'s>
where
    T: IntoId,
{
    fn from_access_arg<'a>(names: (&'s str, T), world: impl WorldProvider<'a>) -> Access<'s> {
        let id = names.1.into_id(world);
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::PairNameEntity(names.0, Entity(*id)),
        }
    }
}

impl<'s, T> FromAccessArg<(T, &'s str)> for Access<'s>
where
    T: IntoId,
{
    fn from_access_arg<'a>(names: (T, &'s str), world: impl WorldProvider<'a>) -> Access<'s> {
        let id = names.0.into_id(world);
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::PairEntityName(Entity(*id), names.1),
        }
    }
}

impl<'s, T: IntoId> FromAccessArg<T> for Access<'s> {
    fn from_access_arg<'a>(id: T, world: impl WorldProvider<'a>) -> Access<'s> {
        let world = world.world();
        let id = id.into_id(world);
        if T::IS_PAIR {
            let first = id.get_id_first(world);
            let second = id.get_id_second(world);
            if <T as IntoId>::IS_TYPED_REF {
                Access {
                    mode: AccessMode::Read,
                    target: AccessTarget::Pair(first, second),
                }
            } else if <T as IntoId>::IS_TYPED_MUT_REF {
                Access {
                    mode: AccessMode::ReadWrite,
                    target: AccessTarget::Pair(first, second),
                }
            } else {
                Access {
                    mode: AccessMode::None,
                    target: AccessTarget::Pair(first, second),
                }
            }
        } else if <T as IntoId>::IS_TYPED_REF {
            Access {
                mode: AccessMode::Read,
                target: AccessTarget::Entity(Entity(*id)),
            }
        } else if <T as IntoId>::IS_TYPED_MUT_REF {
            Access {
                mode: AccessMode::ReadWrite,
                target: AccessTarget::Entity(Entity(*id)),
            }
        } else {
            Access {
                mode: AccessMode::None,
                target: AccessTarget::Entity(Entity(*id)),
            }
        }
    }
}
