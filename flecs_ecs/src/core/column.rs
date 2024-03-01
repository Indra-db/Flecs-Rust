use crate::{core::FlecsErrorCode, ecs_assert};

use super::{component_registration::CachedComponentData, iter::Iter};
use std::{
    ops::{Deref, Index, IndexMut},
    os::raw::c_void,
};

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
    pub fn new_from_array(array: *const T, count: usize, is_shared: bool) -> Self {
        Self {
            slice_components: unsafe { std::slice::from_raw_parts_mut(array as *mut T, count) },
            is_shared,
        }
    }

    pub fn new_from_iter(iter: &'a mut Iter, index_column: i32) -> Self {
        iter.get_field_data::<T>(index_column)
    }

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
