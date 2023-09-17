// Assuming the definitions of WorldT, TypeT, flecs::string, flecs::id_t, and flecs::id
// are known and have been translated to Rust already.

use libc::strlen;

use crate::{core::utility::errors::FlecsErrorCode, ecs_assert};

use super::{
    c_binding::bindings::ecs_type_str,
    c_types::{IdT, TypeT, WorldT},
    id::Id,
};

/// A type is a vector of component ids which can be requested from entities or tables.
pub struct Archetype {
    world: *mut WorldT,
    type_vec: *const TypeT,
}

impl Default for Archetype {
    fn default() -> Self {
        Self {
            world: std::ptr::null_mut(),
            type_vec: std::ptr::null(),
        }
    }
}

impl Archetype {
    pub fn new(world: *mut WorldT, type_vec: *const TypeT) -> Self {
        Archetype { world, type_vec }
    }

    /// Convert type to comma-separated string
    pub fn to_string(&self) -> Option<String> {
        unsafe {
            let raw_ptr = ecs_type_str(self.world, self.type_vec);

            if raw_ptr.is_null() {
                return None;
            }

            let len = strlen(raw_ptr) as usize;

            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            Some(String::from_utf8_unchecked(Vec::from_raw_parts(
                raw_ptr as *mut u8,
                len,
                len,
            )))
        }
    }

    /// Return the number of elements in the type.
    pub fn get_count(&self) -> i32 {
        if self.type_vec.is_null() {
            0
        } else {
            // this is safe because we know type_vec is not null
            unsafe { (*self.type_vec).count }
        }
    }

    /// Return pointer to array.
    pub fn get_array_ptr(&self) -> Option<*mut IdT> {
        if self.type_vec.is_null() {
            None
        } else {
            Some(unsafe { (*self.type_vec).array })
        }
    }

    /// Get id at specified index in type
    pub fn get_id_at_index(&self, index: i32) -> Option<Id> {
        ecs_assert!(!self.type_vec.is_null(), FlecsErrorCode::InvalidParameter);
        ecs_assert!(
            // this is safe because we know type_vec is not null since we would have asserted already if it was
            unsafe { (*self.type_vec).count } > index,
            FlecsErrorCode::OutOfRange
        );

        if self.type_vec.is_null() || index >= self.get_count() {
            None
        } else {
            // this is safe because we did checks above
            Some(Id::new(self.world, unsafe {
                *(*self.type_vec).array.add(index as usize)
            }))
        }
    }

    /// Return pointer to start of array.
    pub fn get_begin_ptr_of_array(&self) -> Option<*mut IdT> {
        self.get_array_ptr()
    }

    /// Return pointer to end of array.
    pub fn get_end_ptr_of_array(&self) -> Option<*mut IdT> {
        if self.type_vec.is_null() {
            None
        } else {
            Some(unsafe { (*self.type_vec).array.add(self.get_count() as usize) })
        }
    }

    /// Return pointer to type.
    pub fn get_raw_type_ptr(&self) -> *const TypeT {
        self.type_vec
    }
}
