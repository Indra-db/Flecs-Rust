use crate::{core::FlecsErrorCode, ecs_assert};

use super::{component_registration::CachedComponentData, iter::Iter};
use std::{
    ops::{Deref, Index, IndexMut},
    os::raw::c_void,
};

/// Wrapper class around a column.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct Column<'a, T>
where
    T: CachedComponentData,
{
    slice_components: &'a mut [T],
    is_shared: bool,
}

impl<'a, T> Column<'a, T>
where
    T: CachedComponentData,
{
    /// Create a new column from component array.
    ///
    /// # Parameters
    ///
    /// * `array`: pointer to the component array.
    /// * `count`: number of elements in the array.
    /// * `is_shared`: whether the component is shared.
    ///
    /// # See also
    ///
    /// * C++ API: `column::column`
    pub fn new_from_array(array: *const T, count: usize, is_shared: bool) -> Self {
        Self {
            slice_components: unsafe { std::slice::from_raw_parts_mut(array as *mut T, count) },
            is_shared,
        }
    }

    /// Create a new column from an iterator.
    ///
    /// # Parameters
    ///
    /// * `iter`: the iterator to create the column from.
    /// * `index_column`: the index of the signature of the query being iterated over.
    pub fn new_from_iter(iter: &'a mut Iter, index_column: i32) -> Self {
        iter.get_field_data::<T>(index_column)
    }

    /// wether the column / component is shared.
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<'a, T> Index<usize> for Column<'a, T>
where
    T: CachedComponentData,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.slice_components[index]
    }
}

impl<'a, T> IndexMut<usize> for Column<'a, T>
where
    T: CachedComponentData,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.slice_components[index]
    }
}

impl<'a, T> Deref for Column<'a, T>
where
    T: CachedComponentData,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        ecs_assert!(
            self.slice_components.is_empty(),
            FlecsErrorCode::OutOfRange,
            "Column is empty"
        );
        &self.slice_components[0]
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
    pub fn new(array: *mut c_void, size: usize, count: usize, is_shared: bool) -> Self {
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
