//! Script pre-registered component

use super::*;
use crate::sys::FLECS_IDEcsScriptID_;
pub type Script = crate::sys::EcsScript;
impl_component_traits_binding_type_w_static_id!(Script, FLECS_IDEcsScriptID_);
