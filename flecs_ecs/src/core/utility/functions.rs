use std::os::raw::c_char;

use crate::{
    core::{
        c_binding::bindings::{
            ecs_field_w_size, ecs_get_mut_id, ecs_has_id, ecs_modified_id, ecs_os_api,
            ecs_strip_generation, ECS_GENERATION_MASK, ECS_ROW_MASK,
        },
        c_types::{EntityT, IdT, InOutKind, IterT, OperKind, WorldT, ECS_DEPENDS_ON, ECS_PAIR},
        component_registration::CachedComponentData,
        utility::errors::FlecsErrorCode,
    },
    ecs_assert,
};

use super::{
    super::c_types::RUST_ECS_COMPONENT_MASK,
    traits::{InOutType, OperType},
};

#[inline(always)]
pub fn ecs_entity_t_comb(lo: u64, hi: u64) -> u64 {
    //((hi as u64) << 32) + lo as u64
    (hi << 32) + lo
}

#[inline(always)]
pub fn ecs_pair(pred: u64, obj: u64) -> u64 {
    ECS_PAIR | ecs_entity_t_comb(obj, pred)
}

#[inline(always)]
pub fn ecs_dependson(entity: EntityT) -> EntityT {
    ecs_pair(ECS_DEPENDS_ON, entity)
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

/// Internal helper function to set a component for an entity.
///
/// This function sets the given value for an entity in the ECS world, ensuring
/// that the type of the component is valid.
///
/// # Type Parameters
///
/// * `T`: The type of the component data. Must implement `CachedComponentData`.
///
/// # Parameters
///
/// * `entity`: The ID of the entity.
/// * `value`: The value to set for the component.
/// * `id`: The ID of the component type.
pub(crate) fn set_helper<T: CachedComponentData>(
    world: *mut WorldT,
    entity: EntityT,
    value: T,
    id: IdT,
) {
    ecs_assert!(
        T::get_size(world) != 0,
        FlecsErrorCode::InvalidParameter,
        "invalid type: {}",
        T::get_symbol_name()
    );

    let comp = unsafe { ecs_get_mut_id(world, entity, id) as *mut T };
    unsafe {
        std::ptr::write(comp, value);
        ecs_modified_id(world, entity, id)
    };
}

#[inline(always)]
pub fn strip_generation(entity: EntityT) -> IdT {
    unsafe { ecs_strip_generation(entity) }
}

#[inline(always)]
pub fn get_generation(entity: EntityT) -> u32 {
    ((entity & ECS_GENERATION_MASK) >> 32) as u32
}

/// gets the component data from the iterator
/// # Safety
/// This function is unsafe because it dereferences the iterator and uses the index to get the component data.
/// ensure that the iterator is valid and the index is valid.
#[inline(always)]
pub unsafe fn ecs_field<T: CachedComponentData>(it: *const IterT, index: i32) -> *mut T {
    let size = std::mem::size_of::<T>();
    ecs_field_w_size(it, size, index) as *mut T
}

pub(crate) fn type_to_inout<T: InOutType>() -> InOutKind {
    T::IN_OUT
}

pub(crate) fn type_to_oper<T: OperType>() -> OperKind {
    T::OPER
}

/// Copies the given Rust &str to a C string and returns a pointer to the C string.
/// this is intended to be used when the C code needs to take ownership of the string.
/// for example when naming a component where the rust function takes &str and the C function takes *mut c_char
pub fn copy_and_allocate_c_char_from_rust_str(data: &str) -> *mut c_char {
    ecs_assert!(
        data.is_ascii(),
        FlecsErrorCode::InvalidParameter,
        "string must be ascii"
    );
    let bytes = data.as_bytes();
    let len = bytes.len() + 1; // +1 for the null terminator
    let memory_c_str = unsafe { ecs_os_api.malloc_.unwrap()(len as i32) } as *mut u8;

    for (i, &byte) in bytes.iter().enumerate() {
        unsafe {
            memory_c_str.add(i).write(byte);
        }
    }

    // Write the null terminator to the end of the memory
    unsafe { memory_c_str.add(bytes.len()).write(0) };

    memory_c_str as *mut c_char
}

pub unsafe fn print_c_string(c_string: *const c_char) {
    // Ensure the pointer is not null
    assert!(!c_string.is_null(), "Null pointer passed to print_c_string");

    // Create a CStr from the raw pointer
    let c_str = std::ffi::CStr::from_ptr(c_string);

    // Convert CStr to a Rust string slice (&str)
    // This can fail if the C string is not valid UTF-8, so handle errors appropriately
    match c_str.to_str() {
        Ok(s) => println!("{}", s),
        Err(_) => println!("Failed to convert C string to Rust string"),
    }
}
