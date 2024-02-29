use std::ffi::{CStr, CString};

use crate::{core::utility::errors::FlecsErrorCode, ecs_assert};

use super::{
    c_binding::bindings::ecs_type_str,
    c_types::{IdT, TypeT, WorldT},
    id::Id,
};

/// A type is a vector of component ids which can be requested from entities or tables.
pub struct Type {
    world: *mut WorldT,
    types: *const TypeT,
    current: usize,
}

impl Type {
    pub fn new(world: *mut WorldT, types: *const TypeT) -> Self {
        Self {
            world,
            types,
            current: 0,
        }
    }

    /// Convert type to comma-separated string
    ///
    /// # See also
    ///
    /// * `type::str`
    pub fn to_str(&self) -> CString {
        let c_str = unsafe { ecs_type_str(self.world, self.types) };
        ecs_assert!(!c_str.is_null(), FlecsErrorCode::InvalidParameter);
        unsafe { CString::from_raw(c_str) }
    }

    /// Return number of ids in type
    ///
    /// # See also
    ///
    /// * `type::count`
    #[inline]
    pub fn get_count(&self) -> i32 {
        unsafe { (*self.types).count }
    }

    /// Return slice to array.
    ///
    /// # See also
    ///
    /// * `type::array`
    #[inline]
    pub fn get_array(&self) -> &[IdT] {
        unsafe { std::slice::from_raw_parts((*self.types).array, self.get_count() as usize) }
    }

    /// Get id at specified index in type
    ///
    /// # See also
    ///
    /// * `type::get`
    pub fn get_id(&self, index: usize) -> Id {
        ecs_assert!(
            index < self.get_count() as usize,
            FlecsErrorCode::OutOfRange
        );
        ecs_assert!(!self.types.is_null(), FlecsErrorCode::InvalidParameter);
        unsafe { Id::new_from_existing(self.world, *(*self.types).array.add(index)) }
    }

    pub fn get_raw_type(&self) -> *const TypeT {
        self.types
    }
}

impl Iterator for Type {
    type Item = IdT;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.get_count() as usize {
            let result = unsafe { *(*self.types).array.add(self.current) };
            self.current += 1;
            Some(result)
        } else {
            self.current = 0;
            None
        }
    }
}
