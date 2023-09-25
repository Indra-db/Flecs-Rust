use std::sync::OnceLock;

use crate::core::{
    c_binding::bindings::{ecs_has_id, ECS_ROW_MASK},
    c_types::WorldT,
};

use super::super::c_types::{PAIR, RUST_ECS_COMPONENT_MASK};

#[inline(always)]
pub fn ecs_entity_t_comb(lo: u64, hi: u64) -> u64 {
    //((hi as u64) << 32) + lo as u64
    (hi << 32) + lo
}

#[inline(always)]
pub fn ecs_pair(pred: u64, obj: u64) -> u64 {
    PAIR.0 | ecs_entity_t_comb(obj, pred)
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[inline(always)]
pub fn ecs_has_pair(world: *const WorldT, entity: u64, first: u64, second: u64) -> bool {
    unsafe { ecs_has_id(world, entity, ecs_pair(first, second)) }
}

#[inline(always)]
pub fn ecs_pair_first(e: u64) -> u64 {
    ecs_entity_t_hi(e & RUST_ECS_COMPONENT_MASK)
}

#[inline(always)]
pub fn ecs_pair_second(e: u64) -> u64 {
    ecs_entity_t_lo(e)
}

#[inline(always)]
pub fn ecs_entity_t_lo(value: u64) -> u64 {
    value as u32 as u64
}

#[inline(always)]
pub fn ecs_entity_t_hi(value: u64) -> u64 {
    value >> 32
}

/// returns [type]
#[inline(always)]
pub fn get_only_type_name<T>() -> &'static str {
    use std::any::type_name;
    let name = type_name::<T>();
    name.split("::").last().unwrap_or(name)
}

/// returns [module]::[type]
#[inline(always)]
pub fn get_full_type_name<T>() -> &'static str {
    use std::any::type_name;
    type_name::<T>()
}

#[inline(always)]
pub fn is_empty_type<T>() -> bool {
    std::mem::size_of::<T>() == 0
}

#[inline(always)]
pub fn ecs_record_to_row(row: u32) -> i32 {
    (row & ECS_ROW_MASK) as i32
}
