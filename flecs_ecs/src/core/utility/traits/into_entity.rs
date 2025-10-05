use core::ffi::c_void;

use super::{super::id::Id, WorldProvider};
use crate::core::{
    ComponentId, ComponentInfo, ConditionalCachedRefTypeSelector, Entity, FlecsCachedRefPairType,
    FlecsIsATag, FlecsIsNotTyped, FlecsIsTyped,
};

pub trait IntoEntity {
    const IS_TYPED_PAIR: bool;
    const IS_TYPED: bool;
    const IF_ID_IS_DEFAULT: bool;
    const IS_TYPED_SECOND: bool;
    const IF_ID_IS_DEFAULT_SECOND: bool;
    const IS_ENUM: bool;
    const IS_TYPE_TAG: bool;
    const IS_TYPED_REF: bool;
    const IS_TYPED_MUT_REF: bool;
    type CastType;
    type IsTyped;
    type IsTag;
    fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity;
}

impl<T: ComponentId> IntoEntity for Id<T> {
    const IS_TYPED_PAIR: bool = false;
    const IS_TYPED: bool = true;
    const IF_ID_IS_DEFAULT: bool = T::IMPLS_DEFAULT;
    const IS_TYPED_SECOND: bool = true;
    const IF_ID_IS_DEFAULT_SECOND: bool = false;
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TYPE_TAG: bool = T::IS_TAG;
    const IS_TYPED_REF: bool = <T as ComponentInfo>::IS_REF;
    const IS_TYPED_MUT_REF: bool = <T as ComponentInfo>::IS_MUT;
    type CastType = T;
    type IsTyped = FlecsIsTyped;
    type IsTag = T::TagType;

    #[inline(always)]
    fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity {
        world.world().component_id::<T>()
    }
}

impl<T: Into<Entity>> IntoEntity for T {
    const IS_TYPED_PAIR: bool = false;
    const IS_TYPED: bool = false;
    const IF_ID_IS_DEFAULT: bool = false; //we don't know if the id is default or not
    const IS_TYPED_SECOND: bool = true;
    const IF_ID_IS_DEFAULT_SECOND: bool = false;
    const IS_ENUM: bool = false;
    const IS_TYPE_TAG: bool = false;
    const IS_TYPED_REF: bool = false;
    const IS_TYPED_MUT_REF: bool = false;
    type CastType = c_void;
    type IsTyped = FlecsIsNotTyped;
    type IsTag = FlecsIsATag;

    #[inline(always)]
    fn into_entity<'a>(self, _world: impl WorldProvider<'a>) -> Entity {
        self.into()
    }
}

#[doc(hidden)]
pub trait InternalIntoEntity {
    const IS_TYPED_PAIR: bool;
    const IS_TYPED: bool;
    const IF_ID_IS_DEFAULT: bool;
    const IS_TYPED_SECOND: bool;
    const IF_ID_IS_DEFAULT_SECOND: bool;
    const IS_ENUM: bool;
    const IS_TYPE_TAG: bool;
    const IS_TYPED_REF: bool;
    const IS_TYPED_MUT_REF: bool;
    type CastType;
    type IsFirstTyped;
    type IsSecondTyped;
    type IsFirstATag;
    type IsSecondATag;

    fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity;
}

#[doc(hidden)]
impl<T: IntoEntity> InternalIntoEntity for T {
    const IS_TYPED_PAIR: bool = T::IS_TYPED_PAIR;
    const IS_TYPED: bool = T::IS_TYPED;
    const IF_ID_IS_DEFAULT: bool = T::IF_ID_IS_DEFAULT;
    const IS_TYPED_SECOND: bool = T::IS_TYPED_SECOND;
    const IF_ID_IS_DEFAULT_SECOND: bool = T::IF_ID_IS_DEFAULT_SECOND;
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_TYPE_TAG: bool = <T as IntoEntity>::IS_TYPE_TAG;
    const IS_TYPED_REF: bool = T::IS_TYPED_REF;
    const IS_TYPED_MUT_REF: bool = T::IS_TYPED_MUT_REF;
    type CastType = T::CastType;
    type IsFirstTyped = T::IsTyped;
    type IsSecondTyped = T::IsTyped;
    type IsFirstATag = T::IsTag;
    type IsSecondATag = T::IsTag;

    #[inline(always)]
    fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity {
        self.into_entity(world)
    }
}

// we implement this to optimize the case where we add a component id<T> to add
// normally we shouldn't implement IntoEntity for Id
#[doc(hidden)]
impl InternalIntoEntity for crate::core::Id {
    const IS_TYPED_PAIR: bool = false;
    const IS_TYPED: bool = false;
    const IF_ID_IS_DEFAULT: bool = false; //we don't know if the id is default or not
    const IS_TYPED_SECOND: bool = true;
    const IF_ID_IS_DEFAULT_SECOND: bool = false;
    const IS_ENUM: bool = false;
    const IS_TYPE_TAG: bool = false;
    const IS_TYPED_REF: bool = false;
    const IS_TYPED_MUT_REF: bool = false;
    type CastType = core::ffi::c_void;
    type IsFirstTyped = FlecsIsNotTyped;
    type IsSecondTyped = FlecsIsNotTyped;
    type IsFirstATag = FlecsIsATag;
    type IsSecondATag = FlecsIsATag;

    #[inline(always)]
    fn into_entity<'a>(self, _world: impl WorldProvider<'a>) -> Entity {
        Entity(self.0)
    }
}

// we implement this to optimize the case where we add a component id<T> to add
// normally we shouldn't implement IntoEntity for Id
#[doc(hidden)]
impl InternalIntoEntity for crate::core::IdView<'_> {
    const IS_TYPED_PAIR: bool = false;
    const IS_TYPED: bool = false;
    const IF_ID_IS_DEFAULT: bool = false; //we don't know if the id is default or not
    const IS_TYPED_SECOND: bool = true;
    const IF_ID_IS_DEFAULT_SECOND: bool = false;
    const IS_ENUM: bool = false;
    const IS_TYPE_TAG: bool = false;
    const IS_TYPED_REF: bool = false;
    const IS_TYPED_MUT_REF: bool = false;
    type CastType = core::ffi::c_void;
    type IsFirstTyped = FlecsIsNotTyped;
    type IsSecondTyped = FlecsIsNotTyped;
    type IsFirstATag = FlecsIsATag;
    type IsSecondATag = FlecsIsATag;

    #[inline(always)]
    fn into_entity<'a>(self, _world: impl WorldProvider<'a>) -> Entity {
        Entity(*self.id)
    }
}

#[doc(hidden)]
impl<T, U> InternalIntoEntity for (T, U)
where
    T: InternalIntoEntity,
    U: InternalIntoEntity,
    ConditionalCachedRefTypeSelector<
        <T as InternalIntoEntity>::IsFirstTyped,
        <U as InternalIntoEntity>::IsSecondTyped,
        <T as InternalIntoEntity>::IsFirstATag,
        <U as InternalIntoEntity>::IsSecondATag,
        T,
        U,
    >: FlecsCachedRefPairType,
{
    const IS_TYPED_PAIR: bool = true;
    const IS_TYPED: bool = T::IS_TYPED | U::IS_TYPED;
    const IF_ID_IS_DEFAULT: bool = T::IF_ID_IS_DEFAULT; //we don't know if the id is default or not
    const IS_TYPED_SECOND: bool = U::IS_TYPED;
    const IF_ID_IS_DEFAULT_SECOND: bool = U::IF_ID_IS_DEFAULT; //we don't know if the id is default or not
    const IS_ENUM: bool = false;
    const IS_TYPE_TAG: bool = T::IS_TYPE_TAG & U::IS_TYPE_TAG;
    const IS_TYPED_REF: bool = false;
    const IS_TYPED_MUT_REF: bool = false;
    type IsFirstTyped = T::IsFirstTyped;
    type IsSecondTyped = U::IsFirstTyped;
    type IsFirstATag = T::IsFirstATag;
    type IsSecondATag = U::IsFirstATag;
    type CastType = <ConditionalCachedRefTypeSelector<
        T::IsFirstTyped,
        U::IsSecondTyped,
        T::IsFirstATag,
        U::IsSecondATag,
        T,
        U,
    > as FlecsCachedRefPairType>::Type;
    #[inline(always)]
    fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity {
        let world = world.world();
        Entity(crate::core::ecs_pair(
            *(self.0.into_entity(world)),
            *(self.1.into_entity(world)),
        ))
    }
}

// #[doc(hidden)]
// impl<T, U> InternalIntoEntity for &(T, U)
// where
//     T: InternalIntoEntity + Copy,
//     U: InternalIntoEntity + Copy,
// {
//     const IS_TYPED_PAIR: bool = true;
//     const IS_TYPED: bool = T::IS_TYPED;
//     const IF_ID_IS_DEFAULT: bool = T::IF_ID_IS_DEFAULT; //we don't know if the id is default or not
//     const IS_TYPED_SECOND: bool = U::IS_TYPED;
//     const IF_ID_IS_DEFAULT_SECOND: bool = U::IF_ID_IS_DEFAULT; //we don't know if the id is default or not
//     const IS_ENUM: bool = false;
//     const IS_TYPE_TAG: bool = T::IS_TYPE_TAG & U::IS_TYPE_TAG;
//     const IS_TYPED_REF: bool = true;
//     const IS_TYPED_MUT_REF: bool = false;
//     fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity {
//         let world = world.world();
//         Entity(crate::core::ecs_pair(
//             *(self.0.into_entity(world)),
//             *(self.1.into_entity(world)),
//         ))
//     }
// }

// #[doc(hidden)]
// impl<T, U> InternalIntoEntity for &mut (T, U)
// where
//     T: InternalIntoEntity + Copy,
//     U: InternalIntoEntity + Copy,
// {
//     const IS_TYPED_PAIR: bool = true;
//     const IS_TYPED: bool = T::IS_TYPED;
//     const IF_ID_IS_DEFAULT: bool = T::IF_ID_IS_DEFAULT; //we don't know if the id is default or not
//     const IS_TYPED_SECOND: bool = U::IS_TYPED;
//     const IF_ID_IS_DEFAULT_SECOND: bool = U::IF_ID_IS_DEFAULT; //we don't know if the id is default or not
//     const IS_ENUM: bool = false;
//     const IS_TYPE_TAG: bool = T::IS_TYPE_TAG & U::IS_TYPE_TAG;
//     const IS_TYPED_REF: bool = false;
//     const IS_TYPED_MUT_REF: bool = true;
//     fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity {
//         let world = world.world();
//         Entity(crate::core::ecs_pair(
//             *(self.0.into_entity(world)),
//             *(self.1.into_entity(world)),
//         ))
//     }
// }
