use crate::core::{ecs_pair, ComponentId, ComponentType, IdT, Struct};

use super::IntoWorld;

pub trait IntoComponentId {
    const IS_ENUM: bool;
    const IS_PAIR: bool;
    // These types are useful for merging functions in World class such ass add_pair<T,U> into add<T>.
    // When IntoComponentId is not a pair, First and Second will be same
    type First: ComponentId;
    type Second: ComponentId;

    fn get_id<'a>(world: impl IntoWorld<'a>) -> IdT;

    /// Get the symbol name of the component.
    ///
    /// # Safety
    ///
    /// Notice that this function for pairs (T, U) will return the type name of the tuple, not the individual components.
    /// This isn't a name stored in the ECS unlike a singular component.
    fn name() -> &'static str;
}

impl<T> IntoComponentId for T
where
    T: ComponentId,
{
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_PAIR: bool = false;
    type First = T;
    type Second = T;

    #[inline]
    fn get_id<'a>(world: impl IntoWorld<'a>) -> IdT {
        T::get_id(world)
    }

    #[inline]
    fn name() -> &'static str {
        std::any::type_name::<T>()
    }
}

impl<T, U> IntoComponentId for (T, U)
where
    T: ComponentId,
    U: ComponentId + ComponentType<Struct>,
{
    const IS_ENUM: bool = false;
    const IS_PAIR: bool = true;
    type First = T;
    type Second = U;

    #[inline]
    fn get_id<'a>(world: impl IntoWorld<'a>) -> IdT {
        let world = world.world();
        ecs_pair(T::get_id(world), U::get_id(world))
    }

    #[inline]
    fn name() -> &'static str {
        std::any::type_name::<(T, U)>()
    }
}
