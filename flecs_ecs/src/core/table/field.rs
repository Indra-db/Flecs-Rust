//! Table column API.

use crate::core::*;
use core::{
    ffi::c_void,
    ops::{Deref, DerefMut},
};

/// Wrapper class around a table column with immutable access.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct Field<'a, T> {
    pub(crate) slice_components: &'a [T],
    pub(crate) is_shared: bool,
}

impl<'a, T> Field<'a, T> {
    pub(crate) fn new(slice_components: &'a [T], is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
        }
    }

    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<T: ComponentId> Deref for Field<'_, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.slice_components
    }
}

/// Wrapper class around a table column with mutable access.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct FieldMut<'a, T> {
    pub(crate) slice_components: &'a mut [T],
    pub(crate) is_shared: bool,
}

impl<'a, T> FieldMut<'a, T> {
    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    pub fn new(slice_components: &'a mut [T], is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
        }
    }

    /// whether the column / component is shared.
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<T: ComponentId> Deref for FieldMut<'_, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.slice_components
    }
}

impl<T: ComponentId> DerefMut for FieldMut<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice_components
    }
}

/// Wrapper class around an untyped table column with immutable access.
/// This class is used primarily for dynamic component types.
pub struct FieldUntyped {
    pub(crate) array: *const c_void,
    pub(crate) size: usize,
    pub(crate) count: usize,
    pub(crate) is_shared: bool,
}

/// Unsafe wrapper class around a column with immutable access.
/// This class can be used when a system does not know the type of a column at
/// compile time.
///
/// # Arguments
///
/// * `array`: pointer to the component array.
/// * `size`: size of the component type.
/// * `count`: number of elements in the array.
/// * `is_shared`: whether the component is shared.
impl FieldUntyped {
    pub(crate) fn new(array: *const c_void, size: usize, count: usize, is_shared: bool) -> Self {
        Self {
            array,
            size,
            count,
            is_shared,
        }
    }

    pub fn at(&self, index: usize) -> *const c_void {
        ecs_assert!(
            index < self.count,
            FlecsErrorCode::OutOfRange,
            "Index {} is out of range {}",
            index,
            self.count
        );

        ecs_assert!(
            !self.is_shared || index == 0,
            FlecsErrorCode::InvalidParameter,
            "Column is shared, cannot index"
        );

        unsafe { self.array.add(index * self.size) }
    }
}

/// Wrapper class around an untyped table column with mutable access.
/// This class is used primarily for dynamic component types.
pub struct FieldUntypedMut {
    pub(crate) array: *mut c_void,
    pub(crate) size: usize,
    pub(crate) count: usize,
    pub(crate) is_shared: bool,
}

/// Unsafe wrapper class around a column with mutable access.
/// This class can be used when a system does not know the type of a column at
/// compile time.
///
/// # Arguments
///
/// * `array`: pointer to the component array.
/// * `size`: size of the component type.
/// * `count`: number of elements in the array.
/// * `is_shared`: whether the component is shared.
impl FieldUntypedMut {
    pub(crate) fn new(array: *mut c_void, size: usize, count: usize, is_shared: bool) -> Self {
        Self {
            array,
            size,
            count,
            is_shared,
        }
    }

    pub fn at(&self, index: usize) -> *const c_void {
        ecs_assert!(
            index < self.count,
            FlecsErrorCode::OutOfRange,
            "Index {} is out of range {}",
            index,
            self.count
        );

        ecs_assert!(
            !self.is_shared || index == 0,
            FlecsErrorCode::InvalidParameter,
            "Column is shared, cannot index"
        );

        unsafe { self.array.add(index * self.size) }
    }

    pub fn at_mut(&self, index: usize) -> *mut c_void {
        ecs_assert!(
            index < self.count,
            FlecsErrorCode::OutOfRange,
            "Index {} is out of range {}",
            index,
            self.count
        );

        ecs_assert!(
            !self.is_shared,
            FlecsErrorCode::InvalidParameter,
            "Column is shared, cannot index"
        );

        unsafe { self.array.add(index * self.size) }
    }
}

// no impl Index/IndexMut for FieldUntyped because it's untyped and it does not support returning ptrs well
