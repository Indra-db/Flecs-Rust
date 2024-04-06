#![allow(non_upper_case_globals)]

use crate::{
    core::component_registration::{ComponentId, ComponentType, IdComponent, Struct},
    sys::{
        ecs_entity_t, ecs_filter_t, ecs_flags32_t, ecs_id_t, ecs_inout_kind_t,
        ecs_inout_kind_t_EcsIn, ecs_inout_kind_t_EcsInOut, ecs_inout_kind_t_EcsInOutDefault,
        ecs_inout_kind_t_EcsInOutNone, ecs_inout_kind_t_EcsOut, ecs_iter_t, ecs_observer_t,
        ecs_oper_kind_t, ecs_oper_kind_t_EcsAnd, ecs_oper_kind_t_EcsAndFrom,
        ecs_oper_kind_t_EcsNot, ecs_oper_kind_t_EcsNotFrom, ecs_oper_kind_t_EcsOptional,
        ecs_oper_kind_t_EcsOr, ecs_oper_kind_t_EcsOrFrom, ecs_primitive_kind_t,
        ecs_query_group_info_t, ecs_query_t, ecs_ref_t, ecs_rule_t, ecs_table_t, ecs_term_id_t,
        ecs_term_t, ecs_type_hooks_t, ecs_type_info_t, ecs_type_kind_t, ecs_type_t,
        ecs_world_info_t, ecs_world_t, EcsComponent, EcsIdentifier, EcsPoly, EcsTarget,
        FLECS_IDEcsComponentID_,
    },
};

#[cfg(feature = "flecs_system")]
use crate::sys::EcsTickSource;

use std::{ffi::CStr, sync::OnceLock};

use super::{ComponentInfo, EntityId, IntoWorld, NoneEnum};

pub const RUST_ecs_id_FLAGS_MASK: u64 = 0xFF << 60;
pub const RUST_ECS_COMPONENT_MASK: u64 = !RUST_ecs_id_FLAGS_MASK;

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
pub type TypeKindT = ecs_type_kind_t;
pub type Flags32T = ecs_flags32_t;
pub type TermIdT = ecs_term_id_t;
pub type TermT = ecs_term_t;
pub type PrimitiveKindT = ecs_primitive_kind_t;
pub type FTimeT = f32;
#[cfg(feature = "flecs_system")]
pub type TickSource = EcsTickSource;

pub static SEPARATOR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"::\0") };

/// Specify read/write access for term
/// Specifies the access pattern of a system to a component term.
///
/// This enum is used to indicate how a system interacts with a component term during its execution,
/// differentiating between read-only access, write-only access, both, or neither.
///
/// Variants:
///
/// - `InOutDefault`: Default behavior, which is `InOut` for regular terms and `In` for shared terms.
/// - `InOutNone`: Indicates the term is neither read nor written by the system.
/// - `InOut`: The term is both read and written, implying a mutable access to the component data.
/// - `In`: The term is only read, implying an immutable access to the component data.
/// - `Out`: The term is only written, providing exclusive access to modify the component data.
#[allow(clippy::unnecessary_cast)]
#[repr(u32)]
pub enum InOutKind {
    InOutDefault = ecs_inout_kind_t_EcsInOutDefault as u32,
    InOutNone = ecs_inout_kind_t_EcsInOutNone as u32,
    InOut = ecs_inout_kind_t_EcsInOut as u32,
    In = ecs_inout_kind_t_EcsIn as u32,
    Out = ecs_inout_kind_t_EcsOut as u32,
}

impl InOutKind {
    pub fn is_read_only(&self) -> bool {
        matches!(self, Self::In)
    }
}

impl From<ecs_inout_kind_t> for InOutKind {
    fn from(value: ecs_inout_kind_t) -> Self {
        match value {
            ecs_inout_kind_t_EcsInOutDefault => InOutKind::InOutDefault,
            ecs_inout_kind_t_EcsInOutNone => InOutKind::InOutNone,
            ecs_inout_kind_t_EcsInOut => InOutKind::InOut,
            ecs_inout_kind_t_EcsIn => InOutKind::In,
            ecs_inout_kind_t_EcsOut => InOutKind::Out,
            _ => InOutKind::InOutDefault,
        }
    }
}

/// Specify operator for term
/// Represents the logical operation applied to a term within a query.
///
/// This enum defines how a term within a query is matched against entities in the ECS,
/// supporting complex query compositions through logical operations.
///
/// Variants:
///
/// - `And`: The term must be present for an entity to match.
/// - `Or`: At least one of the terms in an `Or` chain must be present for an entity to match.
/// - `Not`: The term must not be present for an entity to match.
/// - `Optional`: The presence or absence of the term does not affect matching.
/// - `AndFrom`: All components identified by the term's ID must be present for an entity to match.
/// - `OrFrom`: At least one component identified by the term's ID must be present for an entity to match.
/// - `NotFrom`: None of the components identified by the term's ID should be present for an entity to match.
///
/// These operations allow for flexible and powerful queries within an ECS framework, enabling
/// systems to precisely specify the conditions under which entities are selected for processing.
#[allow(clippy::unnecessary_cast)]
#[repr(u32)]
pub enum OperKind {
    And = ecs_oper_kind_t_EcsAnd as u32,
    Or = ecs_oper_kind_t_EcsOr as u32,
    Not = ecs_oper_kind_t_EcsNot as u32,
    Optional = ecs_oper_kind_t_EcsOptional as u32,
    AndFrom = ecs_oper_kind_t_EcsAndFrom as u32,
    OrFrom = ecs_oper_kind_t_EcsOrFrom as u32,
    NotFrom = ecs_oper_kind_t_EcsNotFrom as u32,
}

impl OperKind {
    pub fn is_negation(&self) -> bool {
        matches!(self, Self::Not | Self::NotFrom)
    }
}

impl From<ecs_oper_kind_t> for OperKind {
    fn from(value: ecs_oper_kind_t) -> Self {
        match value {
            ecs_oper_kind_t_EcsAnd => OperKind::And,
            ecs_oper_kind_t_EcsOr => OperKind::Or,
            ecs_oper_kind_t_EcsNot => OperKind::Not,
            ecs_oper_kind_t_EcsOptional => OperKind::Optional,
            ecs_oper_kind_t_EcsAndFrom => OperKind::AndFrom,
            ecs_oper_kind_t_EcsOrFrom => OperKind::OrFrom,
            ecs_oper_kind_t_EcsNotFrom => OperKind::NotFrom,
            _ => OperKind::And,
        }
    }
}

// Id flags

/// Indicates that the id is a pair.
pub const ECS_PAIR: u64 = 1 << 63;
/// Automatically override component when it is inherited
pub const ECS_OVERRIDE: u64 = 1 << 62;
/// Adds bitset to storage which allows component to be enabled/disabled
pub const ECS_TOGGLE: u64 = 1 << 61;
/// Include all components from entity to which AND is applied
pub const ECS_AND: u64 = 1 << 60;

// Builtin component ids
pub const ECS_COMPONENT: u64 = 1;
pub const ecs_field_idENTIFIER: u64 = 2;
pub const ECS_ITERABLE: u64 = 3;
pub const ECS_POLY: u64 = 4;

// Poly target components
pub const ECS_QUERY: u64 = 5;
pub const ECS_OBSERVER: u64 = 6;
pub const ECS_SYSTEM: u64 = 7;

///Term flags

///  The base ID, equivalent to the C #define
pub const FLECS_HI_COMPONENT_ID: u64 = 256;

/// Match on self
pub const ECS_SELF: u32 = 1 << 1;

/// Match by traversing upwards
pub const ECS_UP: u32 = 1 << 2;

/// Match by traversing downwards (derived, cannot be set)
pub const ECS_DOWN: u32 = 1 << 3;

/// Match all entities encountered through traversal
pub const ECS_TRAVERSE_ALL: u32 = 1 << 4;

/// Sort results breadth first
pub const ECS_CASCADE: u32 = 1 << 5;

/// Iterate groups in descending order (used for ordering)
pub const ECS_DESC: u32 = 1 << 6;

/// Short for up(ChildOf)
pub const ECS_PARENT: u32 = 1 << 7;

/// Term id is a variable
pub const ECS_IS_VARIABLE: u32 = 1 << 8;

/// Term id is an entity
pub const ECS_IS_ENTITY: u32 = 1 << 9;

/// Term id is a name (don't attempt to lookup as entity)
pub const ECS_IS_NAME: u32 = 1 << 10;

/// Prevent observer from triggering on term
pub const ECS_FILTER: u32 = 1 << 11;

/// Union of flags used for traversing (EcsUp|EcsDown|EcsTraverseAll|EcsSelf|EcsCascade|EcsParent)
pub const ECS_TRAVERSE_FLAGS: u32 =
    ECS_UP | ECS_DOWN | ECS_TRAVERSE_ALL | ECS_SELF | ECS_CASCADE | ECS_DESC | ECS_PARENT;

// Core scopes & entities
pub const ECS_WORLD: u64 = FLECS_HI_COMPONENT_ID;
pub const ECS_FLECS: u64 = FLECS_HI_COMPONENT_ID + 1;
pub const ECS_FLECS_CORE: u64 = FLECS_HI_COMPONENT_ID + 2;
pub const ECS_FLECS_INTERNALS: u64 = FLECS_HI_COMPONENT_ID + 3;
pub const ECS_MODULE: u64 = FLECS_HI_COMPONENT_ID + 4;
pub const ECS_PRIVATE: u64 = FLECS_HI_COMPONENT_ID + 5;
pub const ECS_PREFAB: u64 = FLECS_HI_COMPONENT_ID + 6;
pub const ECS_DISABLED: u64 = FLECS_HI_COMPONENT_ID + 7;
pub const ECS_SLOT_OF: u64 = FLECS_HI_COMPONENT_ID + 8;
pub const ECS_FLAG: u64 = FLECS_HI_COMPONENT_ID + 9;

// Relationship properties
pub const ECS_WILDCARD: u64 = FLECS_HI_COMPONENT_ID + 10;
pub const ECS_ANY: u64 = FLECS_HI_COMPONENT_ID + 11;
pub const ECS_THIS: u64 = FLECS_HI_COMPONENT_ID + 12;
pub const ECS_VARIABLE: u64 = FLECS_HI_COMPONENT_ID + 13;
pub const ECS_TRANSITIVE: u64 = FLECS_HI_COMPONENT_ID + 14;
pub const ECS_REFLEXIVE: u64 = FLECS_HI_COMPONENT_ID + 15;
pub const ECS_SYMMETRIC: u64 = FLECS_HI_COMPONENT_ID + 16;
pub const ECS_FINAL: u64 = FLECS_HI_COMPONENT_ID + 17;
pub const ECS_DONT_INHERIT: u64 = FLECS_HI_COMPONENT_ID + 18;
pub const ECS_ALWAYS_OVERRIDE: u64 = FLECS_HI_COMPONENT_ID + 19;
pub const ECS_TAG: u64 = FLECS_HI_COMPONENT_ID + 20;
pub const ECS_UNION: u64 = FLECS_HI_COMPONENT_ID + 21;
pub const ECS_EXCLUSIVE: u64 = FLECS_HI_COMPONENT_ID + 22;
pub const ECS_ACYCLIC: u64 = FLECS_HI_COMPONENT_ID + 23;
pub const ECS_TRAVERSABLE: u64 = FLECS_HI_COMPONENT_ID + 24;
pub const ECS_WITH: u64 = FLECS_HI_COMPONENT_ID + 25;
pub const ECS_ONE_OF: u64 = FLECS_HI_COMPONENT_ID + 26;

// Builtin relationships
pub const ECS_CHILD_OF: u64 = FLECS_HI_COMPONENT_ID + 27;
pub const ECS_IS_A: u64 = FLECS_HI_COMPONENT_ID + 28;
pub const ECS_DEPENDS_ON: u64 = FLECS_HI_COMPONENT_ID + 29;

// Identifier tags
pub const ECS_NAME: u64 = FLECS_HI_COMPONENT_ID + 30;
pub const ECS_SYMBOL: u64 = FLECS_HI_COMPONENT_ID + 31;
pub const ECS_ALIAS: u64 = FLECS_HI_COMPONENT_ID + 32;

// Events
pub const ECS_ON_ADD: u64 = FLECS_HI_COMPONENT_ID + 33;
pub const ECS_ON_REMOVE: u64 = FLECS_HI_COMPONENT_ID + 34;
pub const ECS_ON_SET: u64 = FLECS_HI_COMPONENT_ID + 35;
pub const ECS_UNSET: u64 = FLECS_HI_COMPONENT_ID + 36;
pub const ECS_ON_DELETE: u64 = FLECS_HI_COMPONENT_ID + 37;
pub const ECS_ON_TABLE_CREATE: u64 = FLECS_HI_COMPONENT_ID + 38;
pub const ECS_ON_TABLE_DELETE: u64 = FLECS_HI_COMPONENT_ID + 39;
pub const ECS_ON_TABLE_EMPTY: u64 = FLECS_HI_COMPONENT_ID + 40;
pub const ECS_ON_TABLE_FILL: u64 = FLECS_HI_COMPONENT_ID + 41;
pub const ECS_ON_CREATE_TRIGGER: u64 = FLECS_HI_COMPONENT_ID + 42;
pub const ECS_ON_DELETE_TRIGGER: u64 = FLECS_HI_COMPONENT_ID + 43;
pub const ECS_ON_DELETE_OBSERVABLE: u64 = FLECS_HI_COMPONENT_ID + 44;
pub const ECS_ON_COMPONENT_HOOKS: u64 = FLECS_HI_COMPONENT_ID + 45;
pub const ECS_ON_DELETE_TARGET: u64 = FLECS_HI_COMPONENT_ID + 46;

// Timers
pub const ECS_TICK_SOURCE: u64 = FLECS_HI_COMPONENT_ID + 47;
pub const ECS_TIMER: u64 = FLECS_HI_COMPONENT_ID + 48;
pub const ECS_RATE_FILTER: u64 = FLECS_HI_COMPONENT_ID + 49;

// Actions
pub const ECS_REMOVE: u64 = FLECS_HI_COMPONENT_ID + 50;
pub const ECS_DELETE: u64 = FLECS_HI_COMPONENT_ID + 51;
pub const ECS_PANIC: u64 = FLECS_HI_COMPONENT_ID + 52;

// Misc
pub const ECS_TARGET: u64 = FLECS_HI_COMPONENT_ID + 53;
pub const ECS_FLATTEN: u64 = FLECS_HI_COMPONENT_ID + 54;
pub const ECS_DEFAULT_CHILD_COMPONENT: u64 = FLECS_HI_COMPONENT_ID + 55;

// Builtin predicate ids (used by rule engine)
pub const ECS_PRED_EQ: u64 = FLECS_HI_COMPONENT_ID + 56;
pub const ECS_PRED_MATCH: u64 = FLECS_HI_COMPONENT_ID + 57;
pub const ECS_PRED_LOOKUP: u64 = FLECS_HI_COMPONENT_ID + 58;
pub const ECS_SCOPE_OPEN: u64 = FLECS_HI_COMPONENT_ID + 59;
pub const ECS_SCOPE_CLOSE: u64 = FLECS_HI_COMPONENT_ID + 60;

// Systems
pub const ECS_MONITOR: u64 = FLECS_HI_COMPONENT_ID + 61;
pub const ECS_EMPTY: u64 = FLECS_HI_COMPONENT_ID + 62;
pub const ECS_PIPELINE: u64 = FLECS_HI_COMPONENT_ID + 63;
pub const ECS_ON_START: u64 = FLECS_HI_COMPONENT_ID + 64;
pub const ECS_PRE_FRAME: u64 = FLECS_HI_COMPONENT_ID + 65;
pub const ECS_ON_LOAD: u64 = FLECS_HI_COMPONENT_ID + 66;
pub const ECS_POST_LOAD: u64 = FLECS_HI_COMPONENT_ID + 67;
pub const ECS_PRE_UPDATE: u64 = FLECS_HI_COMPONENT_ID + 68;
pub const ECS_ON_UPDATE: u64 = FLECS_HI_COMPONENT_ID + 69;
pub const ECS_ON_VALIDATE: u64 = FLECS_HI_COMPONENT_ID + 70;
pub const ECS_POST_UPDATE: u64 = FLECS_HI_COMPONENT_ID + 71;
pub const ECS_PRE_STORE: u64 = FLECS_HI_COMPONENT_ID + 72;
pub const ECS_ON_STORE: u64 = FLECS_HI_COMPONENT_ID + 73;
pub const ECS_POST_FRAME: u64 = FLECS_HI_COMPONENT_ID + 74;
pub const ECS_PHASE: u64 = FLECS_HI_COMPONENT_ID + 75;

// Meta primitive components (don't use low ids to save id space)
pub const ECS_BOOL_T: u64 = FLECS_HI_COMPONENT_ID + 80;
pub const ECS_CHAR_T: u64 = FLECS_HI_COMPONENT_ID + 81;
pub const ECS_BYTE_T: u64 = FLECS_HI_COMPONENT_ID + 82;
pub const ECS_U8_T: u64 = FLECS_HI_COMPONENT_ID + 83;
pub const ECS_U16_T: u64 = FLECS_HI_COMPONENT_ID + 84;
pub const ECS_U32_T: u64 = FLECS_HI_COMPONENT_ID + 85;
pub const ECS_U64_T: u64 = FLECS_HI_COMPONENT_ID + 86;
pub const ECS_UPTR_T: u64 = FLECS_HI_COMPONENT_ID + 87;
pub const ECS_I8_T: u64 = FLECS_HI_COMPONENT_ID + 88;
pub const ECS_I16_T: u64 = FLECS_HI_COMPONENT_ID + 89;
pub const ECS_I32_T: u64 = FLECS_HI_COMPONENT_ID + 90;
pub const ECS_I64_T: u64 = FLECS_HI_COMPONENT_ID + 91;
pub const ECS_IPTR_T: u64 = FLECS_HI_COMPONENT_ID + 92;
pub const ECS_F32_T: u64 = FLECS_HI_COMPONENT_ID + 93;
pub const ECS_F64_T: u64 = FLECS_HI_COMPONENT_ID + 94;
pub const ECS_STRING_T: u64 = FLECS_HI_COMPONENT_ID + 95;
pub const ECS_ENTITY_T: u64 = FLECS_HI_COMPONENT_ID + 96;

// Meta module component ids
pub const ECS_META_TYPE: u64 = FLECS_HI_COMPONENT_ID + 97;
pub const ECS_META_TYPE_SERIALIZED: u64 = FLECS_HI_COMPONENT_ID + 98;
pub const ECS_PRIMITIVE: u64 = FLECS_HI_COMPONENT_ID + 99;
pub const ECS_ENUM: u64 = FLECS_HI_COMPONENT_ID + 100;
pub const ECS_BITMASK: u64 = FLECS_HI_COMPONENT_ID + 101;
pub const ECS_MEMBER: u64 = FLECS_HI_COMPONENT_ID + 102;
pub const ECS_STRUCT: u64 = FLECS_HI_COMPONENT_ID + 103;
pub const ECS_ARRAY: u64 = FLECS_HI_COMPONENT_ID + 104;
pub const ECS_VECTOR: u64 = FLECS_HI_COMPONENT_ID + 105;
pub const ECS_OPAQUE: u64 = FLECS_HI_COMPONENT_ID + 106;
pub const ECS_UNIT: u64 = FLECS_HI_COMPONENT_ID + 107;
pub const ECS_UNIT_PREFIX: u64 = FLECS_HI_COMPONENT_ID + 108;
pub const ECS_CONSTANT: u64 = FLECS_HI_COMPONENT_ID + 109;
pub const ECS_QUANTITY: u64 = FLECS_HI_COMPONENT_ID + 110;

// Doc module components
pub const ECS_DOC_DESCRIPTION: u64 = FLECS_HI_COMPONENT_ID + 111;
pub const ECS_DOC_BRIEF: u64 = FLECS_HI_COMPONENT_ID + 112;
pub const ECS_DOC_DETAIL: u64 = FLECS_HI_COMPONENT_ID + 113;
pub const ECS_DOC_LINK: u64 = FLECS_HI_COMPONENT_ID + 114;
pub const ECS_DOC_COLOR: u64 = FLECS_HI_COMPONENT_ID + 115;

// REST module components
pub const ECS_REST: u64 = FLECS_HI_COMPONENT_ID + 116;

pub type Identifier = EcsIdentifier;
pub type Poly = EcsPoly;
pub type Target = EcsTarget;

fn ecs_component_data() -> IdComponent {
    IdComponent {
        id: unsafe { FLECS_IDEcsComponentID_ },
    }
}

fn ecs_poly_data() -> IdComponent {
    IdComponent { id: ECS_POLY }
}

impl ComponentInfo for EcsComponent {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
}

impl ComponentType<Struct> for EcsComponent {}

impl ComponentId for EcsComponent {
    type UnderlyingType = EcsComponent;
    type UnderlyingEnumType = NoneEnum;

    fn register_explicit(_world: impl IntoWorld) {
        //this is already registered in the world inside C
    }

    fn register_explicit_named(_world: impl IntoWorld, _name: &CStr) -> EntityT {
        //this is already registered in the world inside C
        unsafe { FLECS_IDEcsComponentID_ }
    }

    fn is_registered() -> bool {
        //this is already registered in the world inside C
        true
    }

    fn is_registered_with_world(_: impl IntoWorld) -> bool {
        //this is already registered in the world inside C
        true
    }

    fn get_id(_world: impl IntoWorld) -> IdT {
        unsafe { FLECS_IDEcsComponentID_ }
    }

    unsafe fn get_id_unchecked() -> IdT {
        FLECS_IDEcsComponentID_
    }

    fn __get_once_lock_data() -> &'static OnceLock<IdComponent> {
        static ONCE_LOCK: OnceLock<IdComponent> = OnceLock::new();
        &ONCE_LOCK
    }
}

impl ComponentInfo for Poly {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
}

impl ComponentType<Struct> for Poly {}

impl ComponentId for Poly {
    type UnderlyingType = Poly;
    type UnderlyingEnumType = NoneEnum;

    fn register_explicit(_world: impl IntoWorld) {
        //this is already registered in the world inside C
    }

    fn register_explicit_named(_world: impl IntoWorld, _name: &CStr) -> EntityT {
        //this is already registered in the world inside C
        ECS_POLY
    }

    fn is_registered() -> bool {
        //this is already registered in the world inside C
        true
    }

    fn is_registered_with_world(_: impl IntoWorld) -> bool {
        //this is already registered in the world inside C
        true
    }

    fn get_id(_world: impl IntoWorld) -> IdT {
        ECS_POLY
    }

    unsafe fn get_id_unchecked() -> IdT {
        ECS_POLY
    }

    fn __get_once_lock_data() -> &'static OnceLock<IdComponent> {
        static ONCE_LOCK: OnceLock<IdComponent> = OnceLock::new();
        &ONCE_LOCK
    }
}

#[cfg(feature = "flecs_system")]
impl ComponentInfo for TickSource {
    const IS_TAG: bool = false;
    const IS_ENUM: bool = false;
}

#[cfg(feature = "flecs_system")]
impl ComponentId for TickSource {
    type UnderlyingType = TickSource;
    type UnderlyingEnumType = NoneEnum;

    fn register_explicit(_world: impl IntoWorld) {
        //this is already registered in the world inside C
    }

    fn register_explicit_named(_world: impl IntoWorld, _name: &CStr) -> EntityT {
        //this is already registered in the world inside C
        ECS_TICK_SOURCE
    }

    fn is_registered() -> bool {
        //this is already registered in the world inside C
        true
    }

    fn is_registered_with_world(_: impl IntoWorld) -> bool {
        //this is already registered in the world inside C
        true
    }

    unsafe fn get_id_unchecked() -> IdT {
        ECS_TICK_SOURCE
    }

    fn get_id(_: impl IntoWorld) -> IdT {
        ECS_TICK_SOURCE
    }

    fn __get_once_lock_data() -> &'static OnceLock<IdComponent> {
        static ONCE_LOCK: OnceLock<IdComponent> = OnceLock::new();
        &ONCE_LOCK
    }
}

impl ComponentInfo for EntityId {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
}

impl ComponentId for EntityId {
    type UnderlyingType = EntityId;
    type UnderlyingEnumType = NoneEnum;

    fn register_explicit(_world: impl IntoWorld) {
        // already registered by flecs in World
    }

    fn register_explicit_named(_world: impl IntoWorld, _name: &CStr) -> EntityT {
        // already registered by flecs in World
        unsafe { flecs_ecs_sys::FLECS_IDecs_entity_tID_ }
    }

    fn is_registered() -> bool {
        true
    }

    fn is_registered_with_world(_: impl IntoWorld) -> bool {
        //because this is always registered in the c world
        true
    }

    unsafe fn get_id_unchecked() -> IdT {
        //this is safe because it's already registered in flecs_c / world
        flecs_ecs_sys::FLECS_IDecs_entity_tID_
    }

    fn get_id(_world: impl IntoWorld) -> IdT {
        //this is safe because it's already registered in flecs_c / world
        unsafe { flecs_ecs_sys::FLECS_IDecs_entity_tID_ }
    }

    fn __get_once_lock_data() -> &'static OnceLock<IdComponent> {
        static ONCE_LOCK: OnceLock<IdComponent> = OnceLock::new();
        &ONCE_LOCK
    }
}
