use crate::core::Entity;

pub trait EntityId {
    fn entity_id(&self) -> Entity;
}
