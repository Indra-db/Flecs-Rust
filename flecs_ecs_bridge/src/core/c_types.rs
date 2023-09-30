use std::{ffi::CStr, ops::Deref};

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

pub static SEPARATOR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"::\0") };

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
        matches!(self, Self::In)
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
        matches!(self, Self::Not | Self::NotFrom)
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

// The base ID, equivalent to the C #define
const FLECS_HI_COMPONENT_ID: u64 = 256;

// Core scopes & entities
const ECS_WORLD: u64 = FLECS_HI_COMPONENT_ID + 0;
const ECS_FLECS: u64 = FLECS_HI_COMPONENT_ID + 1;
const ECS_FLECS_CORE: u64 = FLECS_HI_COMPONENT_ID + 2;
const ECS_FLECS_INTERNALS: u64 = FLECS_HI_COMPONENT_ID + 3;
const ECS_MODULE: u64 = FLECS_HI_COMPONENT_ID + 4;
const ECS_PRIVATE: u64 = FLECS_HI_COMPONENT_ID + 5;
const ECS_PREFAB: u64 = FLECS_HI_COMPONENT_ID + 6;
const ECS_DISABLED: u64 = FLECS_HI_COMPONENT_ID + 7;
const ECS_SLOT_OF: u64 = FLECS_HI_COMPONENT_ID + 8;
const ECS_FLAG: u64 = FLECS_HI_COMPONENT_ID + 9;

// Relationship properties
const ECS_WILDCARD: u64 = FLECS_HI_COMPONENT_ID + 10;
const ECS_ANY: u64 = FLECS_HI_COMPONENT_ID + 11;
const ECS_THIS: u64 = FLECS_HI_COMPONENT_ID + 12;
const ECS_VARIABLE: u64 = FLECS_HI_COMPONENT_ID + 13;
const ECS_TRANSITIVE: u64 = FLECS_HI_COMPONENT_ID + 14;
const ECS_REFLEXIVE: u64 = FLECS_HI_COMPONENT_ID + 15;
const ECS_SYMMETRIC: u64 = FLECS_HI_COMPONENT_ID + 16;
const ECS_FINAL: u64 = FLECS_HI_COMPONENT_ID + 17;
const ECS_DONT_INHERIT: u64 = FLECS_HI_COMPONENT_ID + 18;
const ECS_ALWAYS_OVERRIDE: u64 = FLECS_HI_COMPONENT_ID + 19;
const ECS_TAG: u64 = FLECS_HI_COMPONENT_ID + 20;
const ECS_UNION: u64 = FLECS_HI_COMPONENT_ID + 21;
const ECS_EXCLUSIVE: u64 = FLECS_HI_COMPONENT_ID + 22;
const ECS_ACYCLIC: u64 = FLECS_HI_COMPONENT_ID + 23;
const ECS_TRAVERSABLE: u64 = FLECS_HI_COMPONENT_ID + 24;
const ECS_WITH: u64 = FLECS_HI_COMPONENT_ID + 25;
const ECS_ONE_OF: u64 = FLECS_HI_COMPONENT_ID + 26;

// Builtin relationships
const ECS_CHILD_OF: u64 = FLECS_HI_COMPONENT_ID + 27;
const ECS_IS_A: u64 = FLECS_HI_COMPONENT_ID + 28;
const ECS_DEPENDS_ON: u64 = FLECS_HI_COMPONENT_ID + 29;

// Identifier tags
const ECS_NAME: u64 = FLECS_HI_COMPONENT_ID + 30;
const ECS_SYMBOL: u64 = FLECS_HI_COMPONENT_ID + 31;
const ECS_ALIAS: u64 = FLECS_HI_COMPONENT_ID + 32;

// Events
const ECS_ON_ADD: u64 = FLECS_HI_COMPONENT_ID + 33;
const ECS_ON_REMOVE: u64 = FLECS_HI_COMPONENT_ID + 34;
const ECS_ON_SET: u64 = FLECS_HI_COMPONENT_ID + 35;
const ECS_UNSET: u64 = FLECS_HI_COMPONENT_ID + 36;
const ECS_ON_DELETE: u64 = FLECS_HI_COMPONENT_ID + 37;
const ECS_ON_TABLE_CREATE: u64 = FLECS_HI_COMPONENT_ID + 38;
const ECS_ON_TABLE_DELETE: u64 = FLECS_HI_COMPONENT_ID + 39;
const ECS_ON_TABLE_EMPTY: u64 = FLECS_HI_COMPONENT_ID + 40;
const ECS_ON_TABLE_FILL: u64 = FLECS_HI_COMPONENT_ID + 41;
const ECS_ON_CREATE_TRIGGER: u64 = FLECS_HI_COMPONENT_ID + 42;
const ECS_ON_DELETE_TRIGGER: u64 = FLECS_HI_COMPONENT_ID + 43;
const ECS_ON_DELETE_OBSERVABLE: u64 = FLECS_HI_COMPONENT_ID + 44;
const ECS_ON_COMPONENT_HOOKS: u64 = FLECS_HI_COMPONENT_ID + 45;
const ECS_ON_DELETE_TARGET: u64 = FLECS_HI_COMPONENT_ID + 46;
