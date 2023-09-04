use std::ops::Deref;

use super::c_binding::bindings::*;
use lazy_static::lazy_static;

pub const RUST_ECS_ID_FLAGS_MASK: u64 = 0xFF << 60;
pub const RUST_ECS_COMPONENT_MASK: u64 = !RUST_ECS_ID_FLAGS_MASK;

pub type WorldT = ecs_world_t;
pub type WorldInfoT = ecs_world_info_t;
pub type QueryGroupInfoT = ecs_query_group_info_t;
pub type IdT = ecs_id_t;
pub type EntityT = ecs_entity_t;
pub type TypeT = ecs_type_t;
pub type TableT = ecs_table_t;
pub type FilterT = ecs_filter_t;
pub type ObserverT = ecs_observer_t;
pub type QueryT = ecs_query_t;
pub type RuleT = ecs_rule_t;
pub type RefT = ecs_ref_t;
pub type IterT = ecs_iter_t;
pub type TypeInfoT = ecs_type_info_t;
pub type TypeHooksT = ecs_type_hooks_t;
pub type Flags32T = ecs_flags32_t;

#[repr(C)]
pub enum InOutKindT {
    InOutDefault = 0,
    InOutNone,
    InOut,
    In,
    Out,
}

//TODO: this is a test
impl InOutKindT {
    pub fn is_read_only(&self) -> bool {
        match self {
            Self::In => true,
            _ => false,
        }
    }
}

#[repr(C)]
pub enum OperKindT {
    And = 0,
    Or,
    Not,
    Optional,
    AndFrom,
    OrFrom,
    NotFrom,
}

//TODO: this is a test
impl OperKindT {
    pub fn is_negation(&self) -> bool {
        match self {
            Self::Not | Self::NotFrom => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Pair(pub ecs_id_t);

lazy_static! {
    pub static ref PAIR: Pair = Pair(unsafe { ECS_PAIR });
}

impl Deref for Pair {
    type Target = ecs_id_t;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
