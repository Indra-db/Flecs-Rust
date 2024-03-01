use super::c_binding::bindings::*;
use super::component_registration::{
    try_register_struct_component, try_register_struct_component_named, ComponentType, Struct,
};

use crate::core::component_registration::{CachedComponentData, ComponentData};

use std::ffi::CStr;
use std::sync::OnceLock;

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
pub type TypeKindT = ecs_type_kind_t;
pub type Flags32T = ecs_flags32_t;
pub type TermIdT = ecs_term_id_t;
pub type TermT = ecs_term_t;
pub type PrimitiveKindT = ecs_primitive_kind_t;
pub type FTimeT = f32;
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
#[repr(C)]
pub enum InOutKind {
    InOutDefault = 0,
    InOutNone = 1,
    InOut = 2,
    In = 3,
    Out = 4,
}

impl InOutKind {
    pub fn is_read_only(&self) -> bool {
        matches!(self, Self::In)
    }
}

impl From<::std::os::raw::c_uint> for InOutKind {
    fn from(value: ::std::os::raw::c_uint) -> Self {
        match value {
            0 => InOutKind::InOutDefault,
            1 => InOutKind::InOutNone,
            2 => InOutKind::InOut,
            3 => InOutKind::In,
            4 => InOutKind::Out,
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
#[repr(C)]
pub enum OperKind {
    And,
    Or,
    Not,
    Optional,
    AndFrom,
    OrFrom,
    NotFrom,
}

impl OperKind {
    pub fn is_negation(&self) -> bool {
        matches!(self, Self::Not | Self::NotFrom)
    }
}

impl From<::std::os::raw::c_uint> for OperKind {
    fn from(value: ::std::os::raw::c_uint) -> Self {
        match value {
            0 => OperKind::And,
            1 => OperKind::Or,
            2 => OperKind::Not,
            3 => OperKind::Optional,
            4 => OperKind::AndFrom,
            5 => OperKind::OrFrom,
            6 => OperKind::NotFrom,
            _ => OperKind::And,
        }
    }
}

impl Default for TypeHooksT {
    fn default() -> Self {
        TypeHooksT {
            ctor: None,
            dtor: None,
            copy: None,
            move_: None,
            copy_ctor: None,
            move_ctor: None,
            ctor_move_dtor: None,
            move_dtor: None,
            on_add: None,
            on_set: None,
            on_remove: None,
            ctx: std::ptr::null_mut(),
            binding_ctx: std::ptr::null_mut(),
            ctx_free: None,
            binding_ctx_free: None,
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
pub const ECS_IDENTIFIER: u64 = 2;
pub const ECS_ITERABLE: u64 = 3;
pub const ECS_POLY: u64 = 4;

// Poly target components
pub const ECS_QUERY: u64 = 5;
pub const ECS_OBSERVER: u64 = 6;
pub const ECS_SYSTEM: u64 = 7;

// The base ID, equivalent to the C #define
pub const FLECS_HI_COMPONENT_ID: u64 = 256;

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

#[allow(clippy::derivable_impls)]
impl Default for EcsComponent {
    fn default() -> Self {
        Self {
            size: Default::default(),
            alignment: Default::default(),
        }
    }
}

fn get_ecs_component_data() -> ComponentData {
    ComponentData {
        id: unsafe { FLECS__EEcsComponent },
        size: std::mem::size_of::<EcsComponent>(),
        alignment: std::mem::align_of::<EcsComponent>(),
        allow_tag: true,
    }
}

fn get_ecs_poly_data() -> ComponentData {
    ComponentData {
        id: ECS_POLY,
        size: std::mem::size_of::<Poly>(),
        alignment: std::mem::align_of::<Poly>(),
        allow_tag: true,
    }
}

impl ComponentType<Struct> for EcsComponent {}

impl CachedComponentData for EcsComponent {
    fn register_explicit(_world: *mut WorldT) {
        //this is already registered as FLECS__EEcsComponent
        Self::__get_once_lock_data().get_or_init(get_ecs_component_data);
    }

    fn register_explicit_named(_world: *mut WorldT, _name: &CStr) {
        //this is already registered as FLECS__EEcsComponent
        Self::__get_once_lock_data().get_or_init(get_ecs_component_data);
    }

    fn is_registered() -> bool {
        Self::__get_once_lock_data().get().is_some()
    }

    fn is_registered_with_world(world: *mut WorldT) -> bool {
        if Self::is_registered() {
            //because this is always registered in the c world
            true
        } else {
            Self::register_explicit(world);
            true
        }
    }

    fn get_data(_world: *mut WorldT) -> &'static ComponentData {
        Self::__get_once_lock_data().get_or_init(get_ecs_component_data)
    }

    fn get_id(_world: *mut WorldT) -> IdT {
        Self::__get_once_lock_data()
            .get_or_init(get_ecs_component_data)
            .id
    }

    fn get_size(_world: *mut WorldT) -> usize {
        Self::__get_once_lock_data()
            .get_or_init(get_ecs_component_data)
            .size
    }

    fn get_alignment(_world: *mut WorldT) -> usize {
        Self::__get_once_lock_data()
            .get_or_init(get_ecs_component_data)
            .alignment
    }

    fn get_allow_tag(_world: *mut WorldT) -> bool {
        Self::__get_once_lock_data()
            .get_or_init(get_ecs_component_data)
            .allow_tag
    }

    fn __get_once_lock_data() -> &'static OnceLock<ComponentData> {
        static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
        &ONCE_LOCK
    }

    fn get_symbol_name_c() -> &'static str {
        static SYMBOL_NAME_C: OnceLock<String> = OnceLock::new();
        SYMBOL_NAME_C.get_or_init(|| String::from("EcsComponent\0"))
    }

    fn get_symbol_name() -> &'static str {
        let name = Self::get_symbol_name_c();
        &name[..name.len() - 1]
    }
}

impl Default for Poly {
    fn default() -> Self {
        Self {
            poly: std::ptr::null_mut(),
        }
    }
}

impl ComponentType<Struct> for Poly {}

impl CachedComponentData for Poly {
    fn register_explicit(_world: *mut WorldT) {
        //this is already registered as FLECS__EEcsComponent
        Self::__get_once_lock_data().get_or_init(get_ecs_poly_data);
    }

    fn register_explicit_named(_world: *mut WorldT, _name: &CStr) {
        //this is already registered as FLECS__EEcsComponent
        Self::__get_once_lock_data().get_or_init(get_ecs_poly_data);
    }

    fn is_registered() -> bool {
        Self::__get_once_lock_data().get().is_some()
    }

    fn is_registered_with_world(world: *mut WorldT) -> bool {
        if Self::is_registered() {
            //because this is always registered in the c world
            true
        } else {
            Self::register_explicit(world);
            true
        }
    }

    fn get_data(_world: *mut WorldT) -> &'static ComponentData {
        Self::__get_once_lock_data().get_or_init(get_ecs_poly_data)
    }

    fn get_id(_world: *mut WorldT) -> IdT {
        Self::__get_once_lock_data()
            .get_or_init(get_ecs_poly_data)
            .id
    }

    fn get_size(_world: *mut WorldT) -> usize {
        Self::__get_once_lock_data()
            .get_or_init(get_ecs_poly_data)
            .size
    }

    fn get_alignment(_world: *mut WorldT) -> usize {
        Self::__get_once_lock_data()
            .get_or_init(get_ecs_poly_data)
            .alignment
    }

    fn get_allow_tag(_world: *mut WorldT) -> bool {
        Self::__get_once_lock_data()
            .get_or_init(get_ecs_poly_data)
            .allow_tag
    }

    fn __get_once_lock_data() -> &'static OnceLock<ComponentData> {
        static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
        &ONCE_LOCK
    }

    fn get_symbol_name_c() -> &'static str {
        static SYMBOL_NAME_C: OnceLock<String> = OnceLock::new();
        SYMBOL_NAME_C.get_or_init(|| String::from("Poly\0"))
    }

    fn get_symbol_name() -> &'static str {
        let name = Self::get_symbol_name_c();
        &name[..name.len() - 1]
    }
}

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

/// Short for up(ChildOf)
pub const ECS_PARENT: u32 = 1 << 6;

/// Term id is a variable
pub const ECS_IS_VARIABLE: u32 = 1 << 7;

/// Term id is an entity
pub const ECS_IS_ENTITY: u32 = 1 << 8;

/// Term id is a name (don't attempt to lookup as entity)
pub const ECS_IS_NAME: u32 = 1 << 9;

/// Prevent observer from triggering on term
pub const ECS_FILTER: u32 = 1 << 10;

/// Union of flags used for traversing (EcsUp|EcsDown|EcsTraverseAll|EcsSelf|EcsCascade|EcsParent)
pub const ECS_TRAVERSE_FLAGS: u32 =
    ECS_UP | ECS_DOWN | ECS_TRAVERSE_ALL | ECS_SELF | ECS_CASCADE | ECS_PARENT;

impl Default for ecs_type_t {
    fn default() -> Self {
        Self {
            array: std::ptr::null_mut(),
            count: Default::default(),
        }
    }
}

impl Default for ecs_term_id_t {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: std::ptr::null_mut(),
            trav: Default::default(),
            flags: Default::default(),
        }
    }
}

impl Default for ecs_term_t {
    fn default() -> Self {
        Self {
            id: Default::default(),
            src: Default::default(),
            first: Default::default(),
            second: Default::default(),
            inout: Default::default(),
            oper: Default::default(),
            id_flags: Default::default(),
            name: std::ptr::null_mut(),
            field_index: Default::default(),
            idr: std::ptr::null_mut(),
            flags: Default::default(),
            move_: Default::default(),
        }
    }
}

impl Default for ecs_filter_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            terms: Default::default(),
            terms_buffer: std::ptr::null_mut(),
            terms_buffer_count: Default::default(),
            storage: std::ptr::null_mut(),
            instanced: Default::default(),
            flags: Default::default(),
            expr: std::ptr::null(),
            entity: Default::default(),
        }
    }
}

impl Default for ecs_query_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            filter: Default::default(),
            order_by_component: Default::default(),
            order_by: Default::default(),
            sort_table: Default::default(),
            group_by_id: Default::default(),
            group_by: Default::default(),
            on_group_create: Default::default(),
            on_group_delete: Default::default(),
            group_by_ctx: std::ptr::null_mut(),
            group_by_ctx_free: Default::default(),
            parent: std::ptr::null_mut(),
        }
    }
}

impl Default for ecs_observer_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            entity: Default::default(),
            filter: Default::default(),
            events: Default::default(),
            yield_existing: Default::default(),
            callback: Default::default(),
            run: Default::default(),
            ctx: std::ptr::null_mut(),
            binding_ctx: std::ptr::null_mut(),
            ctx_free: Default::default(),
            binding_ctx_free: Default::default(),
            observable: std::ptr::null_mut(),
            last_event_id: std::ptr::null_mut(),
            term_index: Default::default(),
        }
    }
}

impl Default for ecs_header_t {
    fn default() -> Self {
        Self {
            magic: ecs_filter_t_magic as ::std::os::raw::c_int,
            type_: Default::default(),
            mixins: std::ptr::null_mut(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ecs_iterable_t {
    fn default() -> Self {
        Self {
            init: Default::default(),
        }
    }
}

impl Default for ecs_filter_t {
    fn default() -> Self {
        unsafe { ECS_FILTER_INIT }
    }
}

impl Default for ecs_entity_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            id: Default::default(),
            name: std::ptr::null(),
            sep: std::ptr::null(),
            root_sep: std::ptr::null(),
            symbol: std::ptr::null(),
            use_low_id: Default::default(),
            add: Default::default(),
            add_expr: std::ptr::null(),
        }
    }
}

impl Default for ecs_event_desc_t {
    fn default() -> Self {
        Self {
            event: Default::default(),
            ids: std::ptr::null(),
            table: std::ptr::null_mut(),
            other_table: std::ptr::null_mut(),
            offset: Default::default(),
            count: Default::default(),
            entity: Default::default(),
            param: std::ptr::null(),
            observable: std::ptr::null_mut(),
            flags: Default::default(),
        }
    }
}

impl Default for ecs_system_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            entity: Default::default(),
            query: Default::default(),
            run: Default::default(),
            callback: Default::default(),
            ctx: std::ptr::null_mut(),
            binding_ctx: std::ptr::null_mut(),
            ctx_free: Default::default(),
            binding_ctx_free: Default::default(),
            interval: Default::default(),
            rate: Default::default(),
            tick_source: Default::default(),
            multi_threaded: Default::default(),
            no_readonly: Default::default(),
        }
    }
}

#[allow(clippy::derivable_impls)] // this is generated by bindgen
impl Default for ecs_pipeline_desc_t {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            query: Default::default(),
        }
    }
}

impl Default for ecs_app_desc_t {
    fn default() -> Self {
        Self {
            target_fps: Default::default(),
            delta_time: Default::default(),
            threads: Default::default(),
            frames: Default::default(),
            enable_rest: Default::default(),
            enable_monitor: Default::default(),
            port: Default::default(),
            init: Default::default(),
            ctx: std::ptr::null_mut(),
        }
    }
}

#[allow(clippy::derivable_impls)] // this is generated by bindgen
impl Default for EcsOpaque {
    fn default() -> Self {
        Self {
            as_type: Default::default(),
            serialize: Default::default(),
            assign_bool: Default::default(),
            assign_char: Default::default(),
            assign_int: Default::default(),
            assign_uint: Default::default(),
            assign_float: Default::default(),
            assign_string: Default::default(),
            assign_entity: Default::default(),
            assign_null: Default::default(),
            clear: Default::default(),
            ensure_element: Default::default(),
            ensure_member: Default::default(),
            count: Default::default(),
            resize: Default::default(),
        }
    }
}

impl Default for TickSource {
    fn default() -> Self {
        Self {
            tick: false,
            time_elapsed: 0.0,
        }
    }
}

impl CachedComponentData for TickSource {
    fn register_explicit(world: *mut WorldT) {
        try_register_struct_component::<Self>(world);
    }

    fn register_explicit_named(world: *mut WorldT, name: &CStr) {
        try_register_struct_component_named::<Self>(world, name);
    }

    fn is_registered() -> bool {
        Self::__get_once_lock_data().get().is_some()
    }

    fn get_data(world: *mut WorldT) -> &'static ComponentData {
        try_register_struct_component::<Self>(world);
        //this is safe because we checked if the component is registered / registered it
        unsafe { Self::get_data_unchecked() }
    }

    fn get_id(world: *mut WorldT) -> IdT {
        try_register_struct_component::<Self>(world);
        //this is safe because we checked if the component is registered / registered it
        unsafe { Self::get_id_unchecked() }
    }

    fn get_size(world: *mut WorldT) -> usize {
        try_register_struct_component::<Self>(world);
        //this is safe because we checked if the component is registered / registered it
        unsafe { Self::get_size_unchecked() }
    }

    fn get_alignment(world: *mut WorldT) -> usize {
        try_register_struct_component::<Self>(world);
        //this is safe because we checked if the component is registered / registered it
        unsafe { Self::get_alignment_unchecked() }
    }

    fn get_allow_tag(world: *mut WorldT) -> bool {
        try_register_struct_component::<Self>(world);
        //this is safe because we checked if the component is registered / registered it
        unsafe { Self::get_allow_tag_unchecked() }
    }

    fn __get_once_lock_data() -> &'static OnceLock<ComponentData> {
        static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
        &ONCE_LOCK
    }

    // Function for C compatibility, returns null-terminated string.
    fn get_symbol_name_c() -> &'static str {
        static SYMBOL_NAME_C: OnceLock<String> = OnceLock::new();
        SYMBOL_NAME_C.get_or_init(|| {
            let name = "EcsTickSource\0".to_string();
            name
        })
    }

    // Function to return a &str slice without the null termination for Rust.
    fn get_symbol_name() -> &'static str {
        let name = Self::get_symbol_name_c();
        &name[..name.len() - 1]
    }
}
