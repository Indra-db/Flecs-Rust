//prototype, temporary probably

use crate::core::component::CachedComponentData;

struct PairT<T: CachedComponentData, U: CachedComponentData> {
    first: T,
    second: U,
}

trait Pair<T: CachedComponentData, U: CachedComponentData> {
    fn get_first_component_id(&self) -> IdT;
    fn get_first_component(&self) -> &T;
    fn get_first_component_mut(&self) -> &mut T;
    fn get_second_component_id(&self) -> IdT;
    fn get_second_component(&self) -> &U;
    fn get_second_component_mut(&self) -> &mut U;
}

impl<T: CachedComponentData, U: CachedComponentData> Pair<T, U> for PairT<T, U> {
    fn get_first_component_id(&self, world: WorldT) -> IdT {
        T::get_id(world)
    }
    fn get_first_component(&self) -> &T {
        &self.first
    }
    fn get_second_component_id(&self, world: WorldT) -> IdT {
        U::get_id(world)
    }
    fn get_second_component(&self) -> &U {
        &self.second
    }

    fn get_first_component_mut(&self) -> &mut T {
        &mut self.first
    }

    fn get_second_component_mut(&self) -> &mut U {
        &mut self.second
    }
}
