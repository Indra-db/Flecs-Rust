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
}

impl<T: InternalIntoEntity> IntoId for T {
    const IS_PAIR: bool = T::IS_TYPED_PAIR;
    const IS_ENUM: bool = <T as InternalIntoEntity>::IS_ENUM;
    const IS_TYPE_TAG: bool = <T as InternalIntoEntity>::IS_TYPE_TAG;
    const IS_TYPED_REF: bool = <T as InternalIntoEntity>::IS_TYPED_REF;
    const IS_TYPED_MUT_REF: bool = <T as InternalIntoEntity>::IS_TYPED_MUT_REF;

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
pub enum AccessTarget {
    /// A single `Entity`
    Entity(Entity),
    /// A single name
    Name(&'static str),
    /// Two Entities
    Pair(Entity, Entity),
    /// Two names
    PairName(&'static str, &'static str),
    /// Name + Entity
    PairNameEntity(&'static str, Entity),
    /// Entity + Name
    PairEntityName(Entity, &'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access {
    pub mode: AccessMode,
    pub target: AccessTarget,
}

impl From<Access> for Id {
    fn from(access: Access) -> Id {
        match access.target {
            AccessTarget::Entity(entity) => entity.into(),
            AccessTarget::Pair(first, second) => Id(ecs_pair(*first, *second)),
            _ => panic!("AccessTarget {:?} not convertible to Id", access.target),
        }
    }
}

pub trait FromAccessArg<T> {
    fn from_access_arg<'a>(value: T, world: impl WorldProvider<'a>) -> Access;
}

/// “Only single‐target inputs” (Entity, &str, `Id<T>`, …)
pub trait SingleAccessArg: Sized
where
    Access: FromAccessArg<Self>,
{
}

/// for single or pair inputs
pub trait AccessArg: Sized
where
    Access: FromAccessArg<Self>,
{
}

impl SingleAccessArg for &'static str {}
impl<T: IntoEntity> SingleAccessArg for T where Access: FromAccessArg<T> {}

impl AccessArg for &'static str {}
impl<T: IntoId> AccessArg for T where Access: FromAccessArg<T> {}
impl AccessArg for (&'static str, &'static str) {}
impl<T: IntoId> AccessArg for (&'static str, T) {}
impl<T: IntoId> AccessArg for (T, &'static str) {}

impl<T: ComponentId> FromAccessArg<&crate::core::utility::id::Id<T>> for Access {
    fn from_access_arg<'a>(
        id: &crate::core::utility::id::Id<T>,
        world: impl WorldProvider<'a>,
    ) -> Access {
        let entity = IntoId::into_id(*id, world);
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::Entity(Entity(*entity)),
        }
    }
}

impl<T: ComponentId> FromAccessArg<&mut crate::core::utility::id::Id<T>> for Access {
    fn from_access_arg<'a>(
        id: &mut crate::core::utility::id::Id<T>,
        world: impl WorldProvider<'a>,
    ) -> Access {
        let entity = IntoId::into_id(*id, world);
        Access {
            mode: AccessMode::ReadWrite,
            target: AccessTarget::Entity(Entity(*entity)),
        }
    }
}

impl FromAccessArg<&'static str> for Access {
    fn from_access_arg<'a>(name: &'static str, _world: impl WorldProvider<'a>) -> Access {
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::Name(name),
        }
    }
}

impl FromAccessArg<(&'static str, &'static str)> for Access {
    fn from_access_arg<'a>(
        names: (&'static str, &'static str),
        _world: impl WorldProvider<'a>,
    ) -> Access {
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::PairName(names.0, names.1),
        }
    }
}

impl<T> FromAccessArg<(&'static str, T)> for Access
where
    T: IntoId,
{
    fn from_access_arg<'a>(names: (&'static str, T), world: impl WorldProvider<'a>) -> Access {
        let id = names.1.into_id(world);
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::PairNameEntity(names.0, Entity(*id)),
        }
    }
}

impl<T> FromAccessArg<(T, &'static str)> for Access
where
    T: IntoId,
{
    fn from_access_arg<'a>(names: (T, &'static str), world: impl WorldProvider<'a>) -> Access {
        let id = names.0.into_id(world);
        Access {
            mode: AccessMode::Read,
            target: AccessTarget::PairEntityName(Entity(*id), names.1),
        }
    }
}

impl<T: IntoId> FromAccessArg<T> for Access {
    fn from_access_arg<'a>(id: T, world: impl WorldProvider<'a>) -> Access {
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
