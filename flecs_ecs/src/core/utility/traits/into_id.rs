use crate::core::*;

/// Extracts the Ecs ID from a type.
/// Extension trait from [`Into<Entity>`] for tuples that implement `Into<Entity>`.
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

// impl<T, U> From<(T, U)> for Id
// where
//     T: IntoEntity,
//     U: IntoEntity,
// {
//     fn from(pair: (T, U)) -> Self {
//         ecs_pair(*pair.0.into_entity(), *pair.1.into()).into()
//     }
// }

// impl<T, U> IntoId for (T, U)
// where
//     T: IntoEntity + Copy,
//     U: IntoEntity + Copy,
// {
//     const IS_PAIR: bool = true;

//     #[doc(hidden)] // not meant to be used by the use
//     #[inline]
//     fn into_id<'a>(self, world: impl WorldProvider<'a>) -> Id {
//         let world = world.world();
//         Id(ecs_pair(
//             *(self.0.into_entity(world)),
//             *(self.1.into_entity(world)),
//         ))
//     }

//     #[doc(hidden)] // not meant to be used by the user
//     #[inline]
//     fn get_id_first<'a>(self, world: impl WorldProvider<'a>) -> Entity {
//         self.0.into_entity(world)
//     }

//     #[doc(hidden)] // not meant to be used by the user
//     #[inline]
//     fn get_id_second<'a>(self, world: impl WorldProvider<'a>) -> Entity {
//         self.1.into_entity(world)
//     }
// }

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

// impl IntoId for Entity {
//     const IS_PAIR: bool = false;
// }

// impl IntoId for Id {
//     const IS_PAIR: bool = false;

//     #[doc(hidden)] // not meant to be used by the user
//     #[inline]
//     fn into_id<'a>(self, _world: impl WorldProvider<'a>) -> Id {
//         self
//     }
//     #[doc(hidden)] // not meant to be used by the user
//     #[inline]
//     fn get_id_first<'a>(self, world: impl WorldProvider<'a>) -> Entity {
//         ecs_first(*self, world)
//     }

//     #[doc(hidden)] // not meant to be used by the user
//     #[inline]
//     fn get_id_second<'a>(self, world: impl WorldProvider<'a>) -> Entity {
//         ecs_second(*self, world)
//     }
// }

// impl IntoId for IdView<'_> {
//     const IS_PAIR: bool = false;

//     #[doc(hidden)] // not meant to be used by the user
//     #[inline]
//     fn into_id<'a>(self, _world: impl WorldProvider<'a>) -> Id {
//         self.id
//     }

//     #[doc(hidden)] // not meant to be used by the user
//     #[inline]
//     fn get_id_first<'a>(self, world: impl WorldProvider<'a>) -> Entity {
//         ecs_first(self.id, world)
//     }
//     #[doc(hidden)] // not meant to be used by the user
//     #[inline]
//     fn get_id_second<'a>(self, world: impl WorldProvider<'a>) -> Entity {
//         ecs_second(self.id, world)
//     }
// }

// impl IntoId for EntityView<'_> {
//     const IS_PAIR: bool = false;
// }

// impl<T> IntoId for Component<'_, T>
// where
//     T: ComponentId,
// {
//     const IS_PAIR: bool = false;
// }

// impl IntoId for UntypedComponent<'_> {
//     const IS_PAIR: bool = false;
// }
