use crate::core::*;
use crate::sys;

pub trait ComponentOrPairId {
    const IS_ENUM: bool;
    const IS_PAIR: bool;
    const IS_TAGS: bool = Self::First::IS_TAG && Self::Second::IS_TAG;
    const IS_FIRST: bool;
    // These types are useful for merging functions in World class such ass add_pair<T,U> into add<T>.
    // When ComponentOrPairId is not a pair, First and Second will be same
    type CastType: ComponentId;
    type First: ComponentId;
    type Second: ComponentId;

    fn get_id<'a>(world: impl WorldProvider<'a>) -> sys::ecs_id_t;

    /// Get the symbol name of the component.
    ///
    /// # Safety
    ///
    /// Notice that this function for pairs (T, U) will return the type name of the tuple, not the individual components.
    /// This isn't a name stored in the ECS unlike a singular component.
    fn name() -> &'static str;
}

impl<T> ComponentOrPairId for T
where
    T: ComponentId + ComponentInfo,
{
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_PAIR: bool = false;
    const IS_FIRST: bool = true;
    type First = T;
    type Second = T;
    type CastType = T;

    #[inline]
    fn get_id<'a>(world: impl WorldProvider<'a>) -> sys::ecs_id_t {
        T::entity_id(world)
    }

    #[inline]
    fn name() -> &'static str {
        core::any::type_name::<T>()
    }
}

impl<T, U> ComponentOrPairId for (T, U)
where
    T: ComponentId + ComponentInfo,
    U: ComponentId + ComponentInfo,
    flecs_ecs::core::ConditionalTypePairSelector<<T as ComponentInfo>::TagType, T, U>:
        flecs_ecs::core::FlecsPairType,
{
    const IS_ENUM: bool = false;
    const IS_PAIR: bool = true;
    const IS_FIRST: bool =
        <ConditionalTypePairSelector<<T as ComponentInfo>::TagType, T,U> as FlecsPairType>::IS_FIRST;
    type First = T;
    type Second = U;
    type CastType =
        <ConditionalTypePairSelector<<T as ComponentInfo>::TagType, T, U> as FlecsPairType>::Type;
    #[inline]
    fn get_id<'a>(world: impl WorldProvider<'a>) -> sys::ecs_id_t {
        let world = world.world();
        ecs_pair(T::entity_id(world), U::entity_id(world))
    }

    #[inline]
    fn name() -> &'static str {
        core::any::type_name::<(T, U)>()
    }
}

impl<T> ComponentOrPairId for crate::core::utility::id::Id<T>
where
    T: ComponentId + ComponentInfo,
{
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_PAIR: bool = false;
    const IS_FIRST: bool = true;
    type First = T;
    type Second = T;
    type CastType = T;

    #[inline]
    fn get_id<'a>(world: impl WorldProvider<'a>) -> sys::ecs_id_t {
        T::entity_id(world)
    }

    #[inline]
    fn name() -> &'static str {
        core::any::type_name::<T>()
    }
}

impl<T, U> ComponentOrPairId
    for (
        crate::core::utility::id::Id<T>,
        crate::core::utility::id::Id<U>,
    )
where
    T: ComponentId + ComponentInfo,
    U: ComponentId + ComponentInfo,
    flecs_ecs::core::ConditionalTypePairSelector<<T as ComponentInfo>::TagType, T, U>:
        flecs_ecs::core::FlecsPairType,
{
    const IS_ENUM: bool = false;
    const IS_PAIR: bool = true;
    const IS_FIRST: bool =
        <ConditionalTypePairSelector<<T as ComponentInfo>::TagType, T,U> as FlecsPairType>::IS_FIRST;
    type First = T;
    type Second = U;
    type CastType =
        <ConditionalTypePairSelector<<T as ComponentInfo>::TagType, T, U> as FlecsPairType>::Type;
    #[inline]
    fn get_id<'a>(world: impl WorldProvider<'a>) -> sys::ecs_id_t {
        let world = world.world();
        ecs_pair(T::entity_id(world), U::entity_id(world))
    }

    #[inline]
    fn name() -> &'static str {
        core::any::type_name::<(T, U)>()
    }
}
