use crate::core::WorldProvider;

use super::{
    entity_id::EntityId, entity_view_const::EntityViewConst, EntityViewEvent, EntityViewMut,
};

pub trait IsEntityView<'w>: EntityId + WorldProvider<'w> {}

impl<'w, T: IsEntityView<'w>> EntityViewConst<'w> for T {}
impl<'w, T: IsEntityView<'w>> EntityViewEvent<'w> for T {}
impl<'w, T: IsEntityView<'w>> EntityViewMut<'w> for T {}
