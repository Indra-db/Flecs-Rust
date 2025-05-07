use super::{super::id::Id, WorldProvider};
use crate::core::{ComponentId, Entity};

pub trait IntoEntity {
    fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity;
}

impl<T: ComponentId> IntoEntity for Id<T> {
    fn into_entity<'a>(self, world: impl WorldProvider<'a>) -> Entity {
        world.world().component_id::<T>()
    }
}

impl<T: Into<Entity>> IntoEntity for T {
    fn into_entity<'a>(self, _world: impl WorldProvider<'a>) -> Entity {
        self.into()
    }
}
