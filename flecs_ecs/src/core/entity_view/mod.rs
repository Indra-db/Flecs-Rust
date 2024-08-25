//! `EntityViews` are wrappers around an [`Entity`][super::Entity] id with the world. It provides methods to build and interact with entities.

mod entity_view;
pub(super) mod entity_view_helper;
mod entity_view_impl;
mod entity_view_traits;

pub use entity_view::*;
use entity_view_helper::*;
pub use entity_view_traits::*;
