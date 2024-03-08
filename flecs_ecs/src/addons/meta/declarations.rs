use crate::{
    core::c_types::{
        EntityT, ECS_BOOL_T, ECS_BYTE_T, ECS_CHAR_T, ECS_CONSTANT, ECS_ENTITY_T, ECS_F32_T,
        ECS_F64_T, ECS_I16_T, ECS_I32_T, ECS_I64_T, ECS_I8_T, ECS_IPTR_T, ECS_QUANTITY,
        ECS_STRING_T, ECS_U32_T, ECS_U64_T, ECS_UPTR_T,
    },
    sys::{
        ecs_bitmask_constant_t, ecs_bool_t, ecs_char_t, ecs_enum_constant_t, ecs_f32_t, ecs_f64_t,
        ecs_i16_t, ecs_i32_t, ecs_i64_t, ecs_i8_t, ecs_iptr_t, ecs_member_t, ecs_u16_t, ecs_u32_t,
        ecs_u64_t, ecs_u8_t, ecs_uptr_t, EcsArray, EcsEnum, EcsMember, EcsMetaTypeSerialized,
        EcsPrimitive, EcsStruct, EcsUnit, EcsVector,
    },
};

// Primitive type aliases
pub type BoolT = ecs_bool_t;
pub type CharT = ecs_char_t;
pub type U8T = ecs_u8_t;
pub type U16T = ecs_u16_t;
pub type U32T = ecs_u32_t;
pub type U64T = ecs_u64_t;
pub type UptrT = ecs_uptr_t;
pub type I8T = ecs_i8_t;
pub type I16T = ecs_i16_t;
pub type I32T = ecs_i32_t;
pub type I64T = ecs_i64_t;
pub type IptrT = ecs_iptr_t;
pub type F32T = ecs_f32_t;
pub type F64T = ecs_f64_t;

// Embedded type aliases
pub type MemberT = ecs_member_t;
pub type EnumConstantT = ecs_enum_constant_t;
pub type BitmaskConstantT = ecs_bitmask_constant_t;

// Components
pub type MetaType = EcsMetaType;
pub type MetaTypeSerialized = EcsMetaTypeSerialized;
pub type Primitive = EcsPrimitive;
pub type Enum = EcsEnum;
pub type Bitmask = EcsBitmask;
pub type Member = EcsMember;
pub type Struct = EcsStruct;
pub type Array = EcsArray;
pub type Vector = EcsVector;
pub type Unit = EcsUnit;

// Base type for bitmasks
pub struct EcsBitmask {
    value: u32,
}

pub const BOOL: EntityT = ECS_BOOL_T;
pub const CHAR: EntityT = ECS_CHAR_T;
pub const BYTE: EntityT = ECS_BYTE_T;
pub const U32: EntityT = ECS_U32_T;
pub const U64: EntityT = ECS_U64_T;
pub const U_PTR: EntityT = ECS_UPTR_T;
pub const I8: EntityT = ECS_I8_T;
pub const I16: EntityT = ECS_I16_T;
pub const I32: EntityT = ECS_I32_T;
pub const I64: EntityT = ECS_I64_T;
pub const I_PTR: EntityT = ECS_IPTR_T;
pub const F32: EntityT = ECS_F32_T;
pub const F64: EntityT = ECS_F64_T;
pub const STRING: EntityT = ECS_STRING_T;
pub const ENTITY: EntityT = ECS_ENTITY_T;
pub const CONSTANT: EntityT = ECS_CONSTANT;
pub const QUANTITY: EntityT = ECS_QUANTITY;

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
