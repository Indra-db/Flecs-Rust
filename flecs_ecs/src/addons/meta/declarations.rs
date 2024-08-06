use crate::core::*;
use crate::sys;

// Primitive type aliases
pub type BoolT = sys::ecs_bool_t;
pub type CharT = sys::ecs_char_t;
pub type U8T = sys::ecs_u8_t;
pub type U16T = sys::ecs_u16_t;
pub type U32T = sys::ecs_u32_t;
pub type U64T = sys::ecs_u64_t;
pub type UptrT = sys::ecs_uptr_t;
pub type I8T = sys::ecs_i8_t;
pub type I16T = sys::ecs_i16_t;
pub type I32T = sys::ecs_i32_t;
pub type I64T = sys::ecs_i64_t;
pub type IptrT = sys::ecs_iptr_t;
pub type F32T = sys::ecs_f32_t;
pub type F64T = sys::ecs_f64_t;

// Embedded type aliases
pub type MemberT = sys::ecs_member_t;
pub type EnumConstantT = sys::ecs_enum_constant_t;
pub type BitmaskConstantT = sys::ecs_bitmask_constant_t;

// Components
//todo this needs a pass, changed with v4, not doing it now as meta is still early stages
// pub type MetaType = EcsMetaType;
// pub type MetaTypeSerialized = sys::EcsMetaTypeSerialized;
pub type Primitive = sys::EcsPrimitive;
//pub type Enum = EcsEnum;
pub type Bitmask = EcsBitmask;
pub type Member = sys::EcsMember;
//pub type Struct = EcsStruct;
pub type Array = sys::EcsArray;
pub type Vector = sys::EcsVector;
pub type Unit = sys::EcsUnit;

// Base type for bitmasks
pub struct EcsBitmask {
    value: u32,
}

pub const BOOL: sys::ecs_entity_t = ECS_BOOL_T;
pub const CHAR: sys::ecs_entity_t = ECS_CHAR_T;
pub const BYTE: sys::ecs_entity_t = ECS_BYTE_T;
pub const U32: sys::ecs_entity_t = ECS_U32_T;
pub const U64: sys::ecs_entity_t = ECS_U64_T;
pub const U_PTR: sys::ecs_entity_t = ECS_UPTR_T;
pub const I8: sys::ecs_entity_t = ECS_I8_T;
pub const I16: sys::ecs_entity_t = ECS_I16_T;
pub const I32: sys::ecs_entity_t = ECS_I32_T;
pub const I64: sys::ecs_entity_t = ECS_I64_T;
pub const I_PTR: sys::ecs_entity_t = ECS_IPTR_T;
pub const F32: sys::ecs_entity_t = ECS_F32_T;
pub const F64: sys::ecs_entity_t = ECS_F64_T;
pub const STRING: sys::ecs_entity_t = ECS_STRING_T;
pub const ENTITY: sys::ecs_entity_t = ECS_ENTITY_T;
pub const CONSTANT: sys::ecs_entity_t = ECS_CONSTANT;
pub const QUANTITY: sys::ecs_entity_t = ECS_QUANTITY;

#[derive(Debug, PartialEq, Eq)]
pub enum EcsTypeKind {
    PrimitiveType,
    BitmaskType,
    EnumType,
    StructType,
    ArrayType,
    VectorType,
    OpaqueType,
}

impl EcsTypeKind {
    pub fn last_type_kind() -> EcsTypeKind {
        EcsTypeKind::OpaqueType
    }
}

#[repr(C)]
pub struct EcsMetaType {
    kind: EcsTypeKind,
    existing: bool,   // Indicates if the type exists or is populated from reflection
    partial: bool,    // Indicates if the reflection data is a partial type description
    size: usize,      // Computed size
    alignment: usize, // Computed alignment
}

#[derive(Debug, PartialEq, Eq)]
pub enum EcsPrimitiveKind {
    Bool = 1,
    Char,
    Byte,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    UPtr,
    IPtr,
    String,
    Entity,
}

impl EcsPrimitiveKind {
    pub fn last_primitive_kind() -> EcsPrimitiveKind {
        EcsPrimitiveKind::Entity
    }
}
