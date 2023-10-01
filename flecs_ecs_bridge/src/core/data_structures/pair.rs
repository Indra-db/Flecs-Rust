//prototype, temporary probably

use crate::core::{
    c_types::{IdT, WorldT},
    component_registration::CachedComponentData,
    utility::functions::ecs_pair,
};

#[derive(Default)]
pub struct PairT<T: CachedComponentData, U: CachedComponentData> {
    first: T,
    second: U,
}

pub trait PairTT {
    type First: CachedComponentData;
    type Second: CachedComponentData;
    fn get_first_component_id(world: *mut WorldT) -> IdT;
    fn get_first_component(&self) -> &Self::First;
    fn get_first_component_mut(&mut self) -> &mut Self::First;
    fn get_second_component_id(world: *mut WorldT) -> IdT;
    fn get_second_component(&self) -> &Self::Second;
    fn get_second_component_mut(&mut self) -> &mut Self::Second;
    fn get_pair_id(world: *mut WorldT) -> IdT {
        ecs_pair(Self::First::get_id(world), Self::Second::get_id(world))
    }
}

impl<T: CachedComponentData, U: CachedComponentData> PairTT for PairT<T, U> {
    type First = T;
    type Second = U;
    fn get_first_component_id(world: *mut WorldT) -> IdT {
        T::get_id(world)
    }
    fn get_first_component(&self) -> &T {
        &self.first
    }
    fn get_second_component_id(world: *mut WorldT) -> IdT {
        U::get_id(world)
    }
    fn get_second_component(&self) -> &U {
        &self.second
    }

    fn get_first_component_mut(&mut self) -> &mut T {
        &mut self.first
    }

    fn get_second_component_mut(&mut self) -> &mut U {
        &mut self.second
    }
}
