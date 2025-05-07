//! Table column API.

#[cfg(feature = "flecs_safety_readwrite_locks")]
use super::iter::FieldError;
use crate::core::*;
#[cfg(feature = "flecs_safety_readwrite_locks")]
use core::ptr::NonNull;
use core::{
    ffi::c_void,
    ops::{Deref, DerefMut},
};
#[cfg(feature = "flecs_safety_readwrite_locks")]
use flecs_ecs_sys::{ecs_id_record_t, ecs_table_t};

/// Wrapper class around an immutable table column.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct Field<'a, T, const LOCK: bool> {
    pub(crate) slice_components: &'a [T],
    pub(crate) is_shared: bool,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) table: NonNull<ecs_table_t>,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) field_index: i8,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) stage_id: i32,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) column_index: i16,
}

impl<T, const LOCK: bool> core::fmt::Debug for Field<'_, T, LOCK>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.slice_components)
    }
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<T, const LOCK: bool> Drop for Field<'_, T, LOCK> {
    fn drop(&mut self) {
        if LOCK {
            unsafe {
                table_column_lock_read_end(self.table.as_mut(), self.column_index, self.stage_id);
            }
        }
    }
}

impl<'a, T> Field<'a, T, false> {
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new_lockless(
        slice_components: &'a [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<ecs_table_t>,
    ) -> Self {
        Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        }
    }

    #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
    pub(crate) fn new_lockless(slice_components: &'a [T], is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
        }
    }
}

impl<'a, T, const LOCK: bool> Field<'a, T, LOCK> {
    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
    pub fn new(slice_components: &'a mut [T], is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
        }
    }

    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    ///
    /// # See also
    ///
    /// * C++ API: `field::field`
    #[doc(alias = "field::field")]
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new(
        slice_components: &'a [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<ecs_table_t>,
        world: &WorldRef,
    ) -> Self {
        if LOCK {
            get_table_column_lock_read_begin(world, table.as_ptr(), column_index, stage_id);
        }
        Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        }
    }

    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    ///
    /// # See also
    ///
    /// * C++ API: `field::field`
    #[doc(alias = "field::field")]
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new_result(
        slice_components: &'a [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<ecs_table_t>,
        world: &WorldRef,
    ) -> Result<Self, FieldError> {
        if LOCK && table_column_lock_read_begin(world, table.as_ptr(), column_index, stage_id) {
            return Err(FieldError::Locked);
        }
        Ok(Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        })
    }

    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn lock_table(&self, world: &WorldRef) {
        get_table_column_lock_read_begin(
            world,
            self.table.as_ptr(),
            self.column_index,
            self.stage_id,
        );
    }

    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn unlock_table(&self) {
        table_column_lock_read_end(self.table.as_ptr(), self.column_index, self.stage_id);
    }

    //// Get the table id of the column.
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub fn table_id(&self) -> u64 {
        unsafe { flecs_ecs_sys::ecs_rust_table_id(self.table.as_ptr()) }
    }

    pub fn drop(self) {}

    /// whether the column / component is shared.
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<T: ComponentId> Deref for Field<'_, T, true> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice_components
    }
}

/// Wrapper class around a mutable table column.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct FieldMut<'a, T, const LOCK: bool> {
    pub slice_components: &'a mut [T],
    pub(crate) is_shared: bool,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) table: NonNull<ecs_table_t>,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) field_index: i8,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) stage_id: i32,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) column_index: i16,
}

impl<T, const LOCK: bool> core::fmt::Debug for FieldMut<'_, T, LOCK>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.slice_components)
    }
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<T, const LOCK: bool> Drop for FieldMut<'_, T, LOCK> {
    fn drop(&mut self) {
        if LOCK {
            unsafe {
                table_column_lock_write_end(self.table.as_mut(), self.column_index, self.stage_id);
            }
        }
    }
}

impl<'a, T> FieldMut<'a, T, false> {
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new_lockless(
        slice_components: &'a mut [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<ecs_table_t>,
    ) -> Self {
        Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        }
    }

    #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
    pub(crate) fn new_lockless(slice_components: &'a mut [T], is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
        }
    }
}

impl<'a, T, const LOCK: bool> FieldMut<'a, T, LOCK> {
    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    ///
    /// # See also
    ///
    /// * C++ API: `field::field`
    #[doc(alias = "field::field")]
    #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
    pub(crate) fn new(slice_components: &'a mut [T], is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
        }
    }

    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    ///
    /// # See also
    ///
    /// * C++ API: `field::field`
    #[doc(alias = "field::field")]
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new(
        slice_components: &'a mut [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<ecs_table_t>,
        world: &WorldRef,
    ) -> Self {
        if LOCK {
            get_table_column_lock_write_begin(world, table.as_ptr(), column_index, stage_id);
        }

        Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        }
    }

    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    ///
    /// # See also
    ///
    /// * C++ API: `field::field`
    #[doc(alias = "field::field")]
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new_result(
        slice_components: &'a mut [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<ecs_table_t>,
        world: &WorldRef,
    ) -> Result<Self, FieldError> {
        if LOCK && table_column_lock_write_begin(world, table.as_ptr(), column_index, stage_id) {
            return Err(FieldError::Locked);
        }

        Ok(Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        })
    }

    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn lock_table(&self, world: &WorldRef) {
        get_table_column_lock_write_begin(
            world,
            self.table.as_ptr(),
            self.column_index,
            self.stage_id,
        );
    }

    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn unlock_table(&self) {
        table_column_lock_write_end(self.table.as_ptr(), self.column_index, self.stage_id);
    }

    //// Get the table id of the column.
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub fn table_id(&self) -> u64 {
        unsafe { flecs_ecs_sys::ecs_rust_table_id(self.table.as_ptr()) }
    }

    pub fn drop(self) {}

    /// whether the column / component is shared.
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<T: ComponentId, const LOCK: bool> Deref for FieldMut<'_, T, LOCK> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice_components
    }
}

impl<T: ComponentId, const LOCK: bool> DerefMut for FieldMut<'_, T, LOCK> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice_components
    }
}

pub struct FieldAt<'a, T> {
    pub(crate) component: &'a T,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) idr: NonNull<ecs_id_record_t>,
}

impl<T> core::fmt::Debug for FieldAt<'_, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.component)
    }
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<T> Drop for FieldAt<'_, T> {
    fn drop(&mut self) {
        unsafe {
            sparse_id_record_lock_read_end(self.idr.as_mut());
        }
    }
}

impl<T: ComponentId> Deref for FieldAt<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<'a, T> FieldAt<'a, T> {
    #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
    pub(crate) fn new(component: &'a T) -> Self {
        Self { component }
    }

    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new(
        component: &'a T,
        world: &WorldRef,
        mut idr: NonNull<ecs_id_record_t>,
    ) -> Self {
        sparse_id_record_lock_read_begin(world, unsafe { idr.as_mut() });
        Self { component, idr }
    }
}

pub struct FieldAtMut<'a, T> {
    pub(crate) component: &'a mut T,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) idr: NonNull<ecs_id_record_t>,
}

impl<T> core::fmt::Debug for FieldAtMut<'_, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.component)
    }
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<T> Drop for FieldAtMut<'_, T> {
    fn drop(&mut self) {
        unsafe {
            sparse_id_record_lock_write_end(self.idr.as_mut());
        }
    }
}

impl<T: ComponentId> Deref for FieldAtMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<T: ComponentId> DerefMut for FieldAtMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component
    }
}

impl<'a, T> FieldAtMut<'a, T> {
    #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
    pub(crate) fn new(component: &'a mut T) -> Self {
        Self { component }
    }

    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new(
        component: &'a mut T,
        world: &WorldRef,
        mut idr: NonNull<ecs_id_record_t>,
    ) -> Self {
        sparse_id_record_lock_write_begin(world, unsafe { idr.as_mut() });
        Self { component, idr }
    }
}

/// Wrapper class around an untyped table column.
/// This class is used primarily for dynamic component types.
pub struct FieldUntyped {
    pub(crate) array: *mut c_void,
    pub(crate) size: usize,
    pub(crate) count: usize,
    pub(crate) is_shared: bool,
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
impl FieldUntyped {
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

    pub fn at_mut(&mut self, index: usize) -> *mut c_void {
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
