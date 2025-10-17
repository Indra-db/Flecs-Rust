/*
using from_json_desc_t = ecs_from_json_desc_t;
using entity_to_json_desc_t = ecs_entity_to_json_desc_t;
using iter_to_json_desc_t = ecs_iter_to_json_desc_t;
*/

use crate::sys;

use super::meta::FetchedId;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub type FromJsonDesc = sys::ecs_from_json_desc_t;
pub type WorldToJsonDesc = sys::ecs_world_to_json_desc_t;
pub type EntityToJsonDesc = sys::ecs_entity_to_json_desc_t;
pub type IterToJsonDesc = sys::ecs_iter_to_json_desc_t;

mod entity_view;
mod world;
