use super::*;
// Meta primitive components (don't use low ids to save id space)
create_pre_registered_component!(Bool, ECS_BOOL_T);
create_pre_registered_component!(Char, ECS_CHAR_T);
create_pre_registered_component!(Byte, ECS_BYTE_T);
create_pre_registered_component!(UPtr, ECS_UPTR_T);
create_pre_registered_component!(IPtr, ECS_IPTR_T);
create_pre_registered_component!(I8, ECS_I8_T);
create_pre_registered_component!(I16, ECS_I16_T);
create_pre_registered_component!(I32, ECS_I32_T);
create_pre_registered_component!(I64, ECS_I64_T);
create_pre_registered_component!(U8, ECS_U8_T);
create_pre_registered_component!(U16, ECS_U16_T);
create_pre_registered_component!(U32, ECS_U32_T);
create_pre_registered_component!(U64, ECS_U64_T);
create_pre_registered_component!(F32, ECS_F32_T);
create_pre_registered_component!(F64, ECS_F64_T);
create_pre_registered_component!(String, ECS_STRING_T);
create_pre_registered_component!(Entity, ECS_ENTITY_T);
create_pre_registered_component!(Quantity, ECS_QUANTITY);
create_pre_registered_component!(EcsOpaque, ECS_OPAQUE);

// Meta type components
pub type Type = sys::EcsType;
pub type TypeSerializer = sys::EcsTypeSerializer;
pub type Primitive = sys::EcsPrimitive;
pub type EcsEnum = sys::EcsEnum;
pub type Bitmask = sys::EcsBitmask;
pub type Member = sys::EcsMember;
pub type MemberRanges = sys::EcsMemberRanges;
pub type EcsStruct = sys::EcsStruct;
pub type Array = sys::EcsArray;
pub type Vector = sys::EcsVector;
pub type Unit = sys::EcsUnit;
pub type UnitPrefix = sys::EcsUnitPrefix;

impl_component_traits_binding_type_w_id!(Type, ECS_META_TYPE);
impl_component_traits_binding_type_w_id!(TypeSerializer, ECS_META_TYPE_SERIALIZER);
impl_component_traits_binding_type_w_id!(Primitive, ECS_PRIMITIVE);
impl_component_traits_binding_type_w_id!(EcsEnum, ECS_ENUM);
impl_component_traits_binding_type_w_id!(Bitmask, ECS_BITMASK);
impl_component_traits_binding_type_w_id!(Member, ECS_MEMBER);
impl_component_traits_binding_type_w_id!(MemberRanges, ECS_MEMBER_RANGES);
impl_component_traits_binding_type_w_id!(EcsStruct, ECS_STRUCT);
impl_component_traits_binding_type_w_id!(Array, ECS_ARRAY);
impl_component_traits_binding_type_w_id!(Vector, ECS_VECTOR);
impl_component_traits_binding_type_w_id!(Unit, ECS_UNIT);
impl_component_traits_binding_type_w_id!(UnitPrefix, ECS_UNIT_PREFIX);
