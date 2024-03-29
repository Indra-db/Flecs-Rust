use crate::core::{ecs_pair, ComponentInfo, ComponentType, IdT, Struct};

use super::IntoWorld;

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
