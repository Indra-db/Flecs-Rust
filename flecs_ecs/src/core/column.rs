use crate::core::*;
use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    os::raw::c_void,
};

/// Wrapper class around a column.
///
/// # Type parameters
///
/// * `T`: The type of the column.

pub struct Column<'a, T> {
    slice_components: &'a mut [T],
    is_shared: bool,
}

impl<'a, T> Column<'a, T> {
    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    ///
    /// # See also
    ///
    /// * C++ API: `column::column`
    #[doc(alias = "column::column")]
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

impl<'a, T: ComponentId> Deref for Column<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice_components
    }
}

impl<'a, T: ComponentId> DerefMut for Column<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice_components
    }
}

pub struct UntypedColumn {
    array: *mut c_void,
    size: usize,
    count: usize,
    is_shared: bool,
}

/// Unsafe wrapper class around a column.
/// This class can be used when a system does not know the type of a column at
/// compile time.
///
/// # Arguments
///
/// * `array`: pointer to the component array.
/// * `size`: size of the component type.
/// * `count`: number of elements in the array.
/// * `is_shared`: whether the component is shared.
///
/// # See also
///
/// * C++ API: `untyped_column::untyped_column`
impl UntypedColumn {
    pub(crate) fn new(array: *mut c_void, size: usize, count: usize, is_shared: bool) -> Self {
        Self {
            array,
            size,
            count,
            is_shared,
        }
    }
}

impl Index<usize> for UntypedColumn {
    type Output = c_void;

    /// # Returns
    ///
    /// Returns element in component array
    ///
    /// # Safety
    ///
    /// This operator may only be used if the column is not shared.
    fn index(&self, index: usize) -> &Self::Output {
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

        unsafe { &*(self.array.add(index * self.size)) }
    }
}

impl IndexMut<usize> for UntypedColumn {
    /// # Returns
    ///
    /// Returns element in component array
    ///
    /// # Safety
    ///
    /// This operator may only be used if the column is not shared.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
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

        unsafe { &mut *(self.array.add(index * self.size)) }
    }
}
