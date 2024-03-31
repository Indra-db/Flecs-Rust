use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    ops::Index,
};

use super::{
    c_types::{IdT, TypeT, WorldT},
    id::Id,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;

use crate::{ecs_assert, sys::ecs_type_str};

/// Archetype type.
/// A type is a vector of component ids which can be requested from entities or tables.
///
/// # See also
///
/// * C++ API: `type`
#[doc(alias = "type")]
pub struct Archetype {
    world: *mut WorldT,
    type_vec: *const TypeT,
}

impl Display for Archetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.to_string() {
            write!(f, "{}", s)
        } else {
            write!(f, "empty archetype")
        }
    }
}

impl Debug for Archetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.to_string() {
            write!(f, "{}", s)
        } else {
            write!(f, "empty archetype")
        }
    }
}

impl Archetype {
    pub(crate) fn new(world: *mut WorldT, type_vec: *const TypeT) -> Self {
        Archetype { world, type_vec }
    }

    /// Convert type to comma-separated string
    ///
    /// # Returns
    ///
    /// Some(String) - if the type is not null. None - if the type is null.
    ///
    /// # See also
    ///
    /// * C++ API: `Type::str()`
    #[doc(alias = "Type::str()")]
    pub fn to_string(&self) -> Option<String> {
        unsafe {
            let raw_ptr = ecs_type_str(self.world, self.type_vec);

            if raw_ptr.is_null() {
                return None;
            }

            let len = CStr::from_ptr(raw_ptr).to_bytes().len();
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
    ///
    /// # See also
    ///
    /// * C++ API: `Type::count()`
    #[doc(alias = "Type::count()")]
    pub fn count(&self) -> i32 {
        if self.type_vec.is_null() {
            0
        } else {
            // this is safe because we know type_vec is not null
            unsafe { (*self.type_vec).count }
        }
    }

    /// Return a slice to the array of types.
    ///
    /// # Returns
    ///
    /// `Some(&[IdT])` - A slice to the array if the type is not null and has elements.
    /// `None` - If the type is null or has no elements.
    ///
    /// # Safety
    ///
    /// This method is safe as long as the underlying array pointed to by `type_vec` does not change
    /// while the slice is in use and the elements are valid `IdT` instances. The caller must ensure
    /// that the `EcsType` instance (or the underlying `type_vec` data it points to) lives at least as
    /// long as the returned slice to avoid dangling references.
    ///
    /// # See also
    ///
    /// * C++ API: `type::array()`
    #[doc(alias = "type::array()")]
    pub fn as_slice(&self) -> Option<&[IdT]> {
        if self.type_vec.is_null() {
            None
        } else {
            // SAFETY: This is safe because we know `type_vec` is not null and we assume
            // the caller ensures that `self` (and thus `type_vec`) lives at least as long as
            // the returned slice. We use `count` to determine the number of elements.
            // The caller must ensure no mutations occur to the underlying data to avoid undefined behavior.
            Some(unsafe {
                std::slice::from_raw_parts((*self.type_vec).array, self.count() as usize)
            })
        }
    }

    /// Get id (struct) at specified index in type
    ///
    /// # Returns
    ///
    /// Result returned as Id Type.
    /// Some(Id) - if the type is not null and the index is within bounds.
    /// None - if the type is null or the index is out of bounds.
    ///
    /// # See also
    ///
    /// * C++ API: `type::get`
    #[doc(alias = "type::get")]
    pub fn id_at_index(&self, index: i32) -> Option<Id> {
        ecs_assert!(!self.type_vec.is_null(), FlecsErrorCode::InvalidParameter);
        ecs_assert!(
            // this is safe because we know type_vec is not null since we would have asserted already if it was
            unsafe { (*self.type_vec).count } > index,
            FlecsErrorCode::OutOfRange
        );

        if self.type_vec.is_null() || index >= self.count() {
            None
        } else {
            // this is safe because we did checks above
            Some(Id::new_from_existing(self.world, unsafe {
                *(*self.type_vec).array.add(index as usize)
            }))
        }
    }

    /// Return pointer to start of array.
    ///
    /// # Returns
    ///
    /// Some(*mut `IdT`) - if the type is not null.
    /// None - if the type is null.
    ///
    /// # See also
    ///
    /// * C++ API: `type::begin`
    #[doc(alias = "type::begin")]
    pub fn begin_ptr_array(&self) -> Option<*mut IdT> {
        if self.type_vec.is_null() {
            None
        } else {
            Some(unsafe { (*self.type_vec).array })
        }
    }

    /// Return pointer to end of array.
    ///
    /// # Returns
    ///
    /// Some(*mut `IdT`) - if the type is not null.
    /// None - if the type is null.
    ///
    /// # See also
    ///
    /// * C++ API: `type::end`
    #[doc(alias = "type::end")]
    pub fn end_ptr_array(&self) -> Option<*mut IdT> {
        if self.type_vec.is_null() {
            None
        } else {
            Some(unsafe { (*self.type_vec).array.add(self.count() as usize) })
        }
    }

    /// Return pointer to type.
    /// Implicit conversion to `type_t`*.
    ///
    /// # Safety
    ///
    /// This method is considered unsafe because it returns a raw pointer to the type data.
    ///
    /// # See also
    ///
    /// * C++ API: `type::operator`
    #[doc(alias = "type::operator")]
    pub unsafe fn get_raw_type_ptr(&self) -> *const TypeT {
        self.type_vec
    }
}

pub struct ArchetypeIter<'a> {
    current: *const IdT,
    end: *const IdT,
    _marker: std::marker::PhantomData<&'a IdT>,
}

impl<'a> Iterator for ArchetypeIter<'a> {
    type Item = &'a IdT;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let item = unsafe { &*self.current };
            self.current = unsafe { self.current.offset(1) };
            Some(item)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a Archetype {
    type Item = &'a IdT;
    type IntoIter = ArchetypeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let array = self.as_slice();
        ArchetypeIter {
            current: array.map(|a| a.as_ptr()).unwrap_or(std::ptr::null()),
            end: unsafe {
                array
                    .map(|a| a.as_ptr().add(a.len()))
                    .unwrap_or(std::ptr::null())
            },
            _marker: std::marker::PhantomData,
        }
    }
}

impl Index<i32> for Archetype {
    type Output = IdT;

    fn index(&self, index: i32) -> &Self::Output {
        let slice = self.as_slice().expect("Archetype type is null");
        let uindex = index as usize;
        &slice[uindex]
    }
}
