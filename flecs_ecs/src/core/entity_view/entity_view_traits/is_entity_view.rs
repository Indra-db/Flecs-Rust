use crate::core::WorldProvider;

use super::{
    const_entity_view::ConstEntityView, entity_id::EntityId, EventEntityView, MutEntityView,
};

pub trait IsEntityView<'w>: EntityId + WorldProvider<'w> {}

impl<'w, T: IsEntityView<'w>> ConstEntityView<'w> for T {}
impl<'w, T: IsEntityView<'w>> EventEntityView<'w> for T {}
impl<'w, T: IsEntityView<'w>> MutEntityView<'w> for T {}
