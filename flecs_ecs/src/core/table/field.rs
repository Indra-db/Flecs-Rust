//! Table column API.

#[cfg(feature = "flecs_safety_locks")]
use super::iter::FieldError;
use crate::core::*;
use crate::sys;
use core::ffi::c_void;
use core::ops::Index;
use core::ops::IndexMut;
use core::ops::{Deref, DerefMut};
#[cfg(feature = "flecs_safety_locks")]
use core::ptr::NonNull;

// TODO I can probably return two different field types, one for shared and one for non-shared
// then I can customize the index behavior

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldIndex(pub(crate) usize);

impl FieldIndex {
    /// Constructs a new `FieldIndex` from a `usize` value.
    ///
    /// This is useful when you need a more efficient indexing mechanism for fields than usize as it avoids unnecessary bounds checks.
    ///
    /// # Safety
    /// The caller must ensure that `value` is a valid field index.
    /// This function does *not* perform any bounds or validity checks.
    #[inline(always)]
    pub unsafe fn new(value: usize) -> Self {
        Self(value)
    }
}

impl From<usize> for FieldIndex {
    #[inline(always)]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<FieldIndex> for usize {
    #[inline(always)]
    fn from(value: FieldIndex) -> Self {
        value.0
    }
}

/// Wrapper class around a table column with immutable access.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct Field<'a, T, const LOCK: bool> {
    pub(crate) slice_components: &'a [T],
    pub(crate) is_shared: bool,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) table: NonNull<sys::ecs_table_t>,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) stage_id: Option<i32>,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) column_index: i16,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) field_index: i8,
}

impl<T, const LOCK: bool> core::fmt::Debug for Field<'_, T, LOCK>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.slice_components)
    }
}

#[cfg(feature = "flecs_safety_locks")]
impl<T, const LOCK: bool> Drop for Field<'_, T, LOCK> {
    fn drop(&mut self) {
        if LOCK {
            unsafe {
                if let Some(stage_id) = self.stage_id {
                    table_column_lock_read_end::<true>(
                        self.table.as_mut(),
                        self.column_index,
                        stage_id,
                    );
                } else {
                    table_column_lock_read_end::<false>(self.table.as_mut(), self.column_index, 0);
                }
            }
        }
    }
}

impl<'a, T> Field<'a, T, false> {
    #[cfg(feature = "flecs_safety_locks")]
    #[inline(always)]
    pub(crate) fn new_lockless(
        slice_components: &'a [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<sys::ecs_table_t>,
        world: &WorldRef,
    ) -> Self {
        let stage_id = if world.is_currently_multithreaded() {
            Some(stage_id)
        } else {
            None
        };
        Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        }
    }

    #[inline(always)]
    #[cfg(not(feature = "flecs_safety_locks"))]
    pub(crate) fn new_lockless(slice_components: &'a [T], is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
        }
    }
}

impl<'a, T, const LOCK: bool> Field<'a, T, LOCK> {
    #[cfg(not(feature = "flecs_safety_locks"))]
    #[inline(always)]
    pub(crate) fn new(slice_components: &'a [T], is_shared: bool) -> Self {
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
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) fn new<const MULTITHREADED: bool>(
        slice_components: &'a [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<sys::ecs_table_t>,
        world: &WorldRef,
    ) -> Self {
        if LOCK {
            get_table_column_lock_read_begin::<MULTITHREADED>(
                world,
                table.as_ptr(),
                column_index,
                stage_id,
            );
        }
        let stage_id = if MULTITHREADED { Some(stage_id) } else { None };
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
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) fn new_result<const MULTITHREADED: bool>(
        slice_components: &'a [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<sys::ecs_table_t>,
        world: &WorldRef,
    ) -> Result<Self, FieldError> {
        if LOCK
            && table_column_lock_read_begin::<MULTITHREADED>(
                world,
                table.as_ptr(),
                column_index,
                stage_id,
            )
        {
            return Err(FieldError::Locked);
        }
        let stage_id = if MULTITHREADED { Some(stage_id) } else { None };
        Ok(Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        })
    }

    // #[cfg(feature = "flecs_safety_locks")]
    // pub(crate) fn lock_table(&self, world: &WorldRef) {
    //     get_table_column_lock_read_begin(
    //         world,
    //         self.table.as_ptr(),
    //         self.column_index,
    //         self.stage_id,
    //     );
    // }

    // #[cfg(feature = "flecs_safety_locks")]
    // pub(crate) fn unlock_table(&self) {
    //     table_column_lock_read_end(self.table.as_ptr(), self.column_index, self.stage_id);
    // }

    //// Get the table id of the column.
    #[cfg(feature = "flecs_safety_locks")]
    pub fn table_id(&self) -> u64 {
        unsafe { sys::flecs_table_id(self.table.as_ptr()) }
    }

    pub fn drop(self) {}

    /// whether the column / component is shared.
    ///
    /// # Returns
    /// `true` if the column is shared, `false` otherwise.
    #[inline(always)]
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }

    /// Get the length of the column.
    ///
    /// # Returns
    /// The number of elements in the column.
    #[inline(always)]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.slice_components.len()
    }

    /// Get element at the specified index.
    ///
    /// # Arguments
    /// * `index`: The index of the element to retrieve.
    ///
    /// # Returns
    /// Option of reference to the element at the specified index.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.slice_components.get(index)
    }

    /// Get table field as a slice
    pub fn as_slice(&self) -> &[T] {
        self.slice_components
    }
}

impl<'a, T, const LOCK: bool> Index<FieldIndex> for Field<'a, T, LOCK> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: FieldIndex) -> &'a Self::Output {
        // Safety: This index can only be obtained from `it.iter` or unsafe FieldIndex::new
        ecs_assert!(
            !(self.is_shared && idx.0 > 0),
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { self.slice_components.get_unchecked(idx.0) }
    }
}

impl<'a, T, const LOCK: bool> Index<usize> for Field<'a, T, LOCK> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: usize) -> &Self::Output {
        ecs_assert!(
            !(self.is_shared && idx > 0),
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        &self.slice_components[idx]
    }
}

/// Wrapper class around a table column with mutable access.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct FieldMut<'a, T, const LOCK: bool> {
    pub(crate) slice_components: &'a mut [T],
    pub(crate) is_shared: bool,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) table: NonNull<sys::ecs_table_t>,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) stage_id: Option<i32>,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) column_index: i16,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) field_index: i8,
}

impl<T, const LOCK: bool> core::fmt::Debug for FieldMut<'_, T, LOCK>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.slice_components)
    }
}

#[cfg(feature = "flecs_safety_locks")]
impl<T, const LOCK: bool> Drop for FieldMut<'_, T, LOCK> {
    fn drop(&mut self) {
        if LOCK {
            if let Some(stage_id) = self.stage_id {
                unsafe {
                    table_column_lock_write_end::<true>(
                        self.table.as_mut(),
                        self.column_index,
                        stage_id,
                    );
                }
            } else {
                unsafe {
                    table_column_lock_write_end::<false>(self.table.as_mut(), self.column_index, 0);
                }
            }
        }
    }
}

impl<'a, T> FieldMut<'a, T, false> {
    #[cfg(feature = "flecs_safety_locks")]
    #[inline(always)]
    pub(crate) fn new_lockless(
        slice_components: &'a mut [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<sys::ecs_table_t>,
        world: &WorldRef,
    ) -> Self {
        let stage_id = if world.is_currently_multithreaded() {
            Some(stage_id)
        } else {
            None
        };
        Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        }
    }

    #[cfg(not(feature = "flecs_safety_locks"))]
    #[inline(always)]
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
    #[inline(always)]
    #[cfg(not(feature = "flecs_safety_locks"))]
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
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) fn new<const MULTITHREADED: bool>(
        slice_components: &'a mut [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<sys::ecs_table_t>,
        world: &WorldRef,
    ) -> Self {
        if LOCK {
            get_table_column_lock_write_begin::<MULTITHREADED>(
                world,
                table.as_ptr(),
                column_index,
                stage_id,
            );
        }

        let stage_id = if MULTITHREADED { Some(stage_id) } else { None };

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
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) fn new_result<const MULTITHREADED: bool>(
        slice_components: &'a mut [T],
        is_shared: bool,
        stage_id: i32,
        column_index: i16,
        field_index: i8,
        table: NonNull<sys::ecs_table_t>,
        world: &WorldRef,
    ) -> Result<Self, FieldError> {
        if LOCK
            && table_column_lock_write_begin::<MULTITHREADED>(
                world,
                table.as_ptr(),
                column_index,
                stage_id,
            )
        {
            return Err(FieldError::Locked);
        }

        let stage_id = if MULTITHREADED { Some(stage_id) } else { None };

        Ok(Self {
            slice_components,
            is_shared,
            table,
            field_index,
            stage_id,
            column_index,
        })
    }

    // #[cfg(feature = "flecs_safety_locks")]
    // pub(crate) fn lock_table(&self, world: &WorldRef) {
    //     get_table_column_lock_write_begin(
    //         world,
    //         self.table.as_ptr(),
    //         self.column_index,
    //         self.stage_id,
    //     );
    // }

    // #[cfg(feature = "flecs_safety_locks")]
    // pub(crate) fn unlock_table(&self) {
    //     table_column_lock_write_end(self.table.as_ptr(), self.column_index, self.stage_id);
    // }

    //// Get the table id of the column.
    #[cfg(feature = "flecs_safety_locks")]
    pub fn table_id(&self) -> u64 {
        unsafe { flecs_ecs_sys::flecs_table_id(self.table.as_ptr()) }
    }

    pub fn drop(self) {}

    /// whether the column / component is shared.
    #[inline(always)]
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }

    /// Get the length of the column.
    ///
    /// # Returns
    /// The number of elements in the column.
    #[inline(always)]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.slice_components.len()
    }

    /// Get Reference to the element at the specified index.
    ///
    /// # Arguments
    /// * `index`: The index of the element to retrieve.
    ///
    /// # Returns
    /// Option of reference to the element at the specified index.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.slice_components.get(index)
    }

    /// Get mutable reference to the element at the specified index.
    ///
    /// # Arguments
    /// * `index`: The index of the element to retrieve.
    ///
    /// # Returns
    /// Option of mutable reference to the element at the specified index.
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.slice_components.get_mut(index)
    }

    /// Get table field as a slice
    pub fn as_slice(&self) -> &[T] {
        self.slice_components
    }

    /// Get mutable table field as a slice
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.slice_components
    }
}

impl<'a, T, const LOCK: bool> Index<FieldIndex> for FieldMut<'a, T, LOCK> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: FieldIndex) -> &T {
        // Safety: This index can only be obtained from `it.iter` or unsafe FieldIndex::new
        ecs_assert!(
            !(self.is_shared && idx.0 > 0),
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { self.slice_components.get_unchecked(idx.0) }
    }
}

impl<'a, T, const LOCK: bool> IndexMut<FieldIndex> for FieldMut<'a, T, LOCK> {
    #[inline(always)]
    fn index_mut(&mut self, idx: FieldIndex) -> &mut T {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            !(self.is_shared && idx.0 > 0),
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { &mut *self.slice_components.get_unchecked_mut(idx.0) }
    }
}

impl<'a, T, const LOCK: bool> Index<usize> for FieldMut<'a, T, LOCK> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: usize) -> &T {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            !(self.is_shared && idx > 0),
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        &self.slice_components[idx]
    }
}

impl<'a, T, const LOCK: bool> IndexMut<usize> for FieldMut<'a, T, LOCK> {
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut T {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            !(self.is_shared && idx > 0),
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        &mut self.slice_components[idx]
    }
}

pub struct FieldAt<'a, T> {
    pub(crate) component: &'a T,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) idr: NonNull<sys::ecs_component_record_t>,
    #[cfg(feature = "flecs_safety_locks")]
    is_multithreaded: bool,
}

impl<T> core::fmt::Debug for FieldAt<'_, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.component)
    }
}

#[cfg(feature = "flecs_safety_locks")]
impl<T> Drop for FieldAt<'_, T> {
    fn drop(&mut self) {
        if self.is_multithreaded {
            unsafe {
                sparse_id_record_lock_read_end::<true>(self.idr.as_mut());
            }
        } else {
            unsafe {
                sparse_id_record_lock_read_end::<false>(self.idr.as_mut());
            }
        }
    }
}

impl<T: ComponentId> Deref for FieldAt<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<'a, T> FieldAt<'a, T> {
    #[cfg(not(feature = "flecs_safety_locks"))]
    pub(crate) fn new(component: &'a T) -> Self {
        Self { component }
    }

    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) fn new(
        component: &'a T,
        world: &WorldRef,
        mut idr: NonNull<sys::ecs_component_record_t>,
    ) -> Self {
        let is_multithreaded = world.is_currently_multithreaded();
        if is_multithreaded {
            sparse_id_record_lock_read_begin::<true>(world, unsafe { idr.as_mut() });
        } else {
            sparse_id_record_lock_read_begin::<false>(world, unsafe { idr.as_mut() });
        }

        Self {
            component,
            idr,
            is_multithreaded,
        }
    }
}

pub struct FieldAtMut<'a, T> {
    pub(crate) component: &'a mut T,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) idr: NonNull<sys::ecs_component_record_t>,
    #[cfg(feature = "flecs_safety_locks")]
    is_multithreaded: bool,
}

impl<T> core::fmt::Debug for FieldAtMut<'_, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.component)
    }
}

#[cfg(feature = "flecs_safety_locks")]
impl<T> Drop for FieldAtMut<'_, T> {
    fn drop(&mut self) {
        if self.is_multithreaded {
            unsafe {
                sparse_id_record_lock_write_end::<true>(self.idr.as_mut());
            }
        } else {
            unsafe {
                sparse_id_record_lock_write_end::<false>(self.idr.as_mut());
            }
        }
    }
}

impl<T: ComponentId> Deref for FieldAtMut<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<T: ComponentId> DerefMut for FieldAtMut<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component
    }
}

impl<'a, T> FieldAtMut<'a, T> {
    #[cfg(not(feature = "flecs_safety_locks"))]
    #[inline(always)]
    pub(crate) fn new(component: &'a mut T) -> Self {
        Self { component }
    }

    #[cfg(feature = "flecs_safety_locks")]
    #[inline(always)]
    pub(crate) fn new(
        component: &'a mut T,
        world: &WorldRef,
        mut idr: NonNull<sys::ecs_component_record_t>,
    ) -> Self {
        let is_multithreaded = world.is_currently_multithreaded();
        if is_multithreaded {
            sparse_id_record_lock_write_begin::<true>(world, unsafe { idr.as_mut() });
        } else {
            sparse_id_record_lock_write_begin::<false>(world, unsafe { idr.as_mut() });
        }

        Self {
            component,
            idr,
            is_multithreaded,
        }
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
    #[inline(always)]
    pub(crate) fn new(array: *const c_void, size: usize, count: usize, is_shared: bool) -> Self {
        Self {
            array,
            size,
            count,
            is_shared,
        }
    }

    #[inline(always)]
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
    #[inline(always)]
    pub(crate) fn new(array: *mut c_void, size: usize, count: usize, is_shared: bool) -> Self {
        Self {
            array,
            size,
            count,
            is_shared,
        }
    }

    #[inline(always)]
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

    #[inline(always)]
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

/// copy of `ecs_field_w_size` from `flecs_sys`. Rewriting in rust for inlining.
/// Gets the component data from the iterator.
/// Retrieves a pointer to the data array for a specified query field.
///
/// This function obtains a pointer to an array of data corresponding to the term in the query,
/// based on the given index. The index starts from 0, representing the first term in the query.
///
/// For instance, given a query "Position, Velocity", invoking this function with index 0 would
/// return a pointer to the "Position" data array, and index 1 would return the "Velocity" data array.
///
/// If the specified field is not owned by the entity being iterated (e.g., a shared component from a prefab,
/// a component from a parent, or a component from another entity), this function returns a direct pointer
/// instead of an array pointer. Use `ecs_field_is_self` to dynamically check if a field is owned.
///
/// The `size` of the type `T` must match the size of the data type of the returned array. Mismatches between
/// the provided type size and the actual data type size may cause the operation to assert. The size of the
/// field can be obtained dynamically using `ecs_field_size`.
///
/// # Safety
///
/// This function is unsafe because it dereferences the iterator and uses the index to get the component data.
/// Ensure that the iterator is valid and the index is valid.
///
/// # Arguments
///
/// - `it`: A pointer to the iterator.
/// - `index`: The index of the field in the iterator, starting from 0.
///
/// # Returns
///
/// A pointer to the data of the specified field. The pointer type is determined by the generic type `T`.
///
/// # Example
///
/// ```ignore
/// // Assuming `it` is a valid iterator pointer obtained from a query.
/// let position_ptr: *mut Position = ecs_field(it, 0);
/// let velocity_ptr: *mut Velocity = ecs_field(it, 1);
/// ```
#[inline(always)]
pub(crate) fn flecs_field<T>(it: &sys::ecs_iter_t, index: i8) -> *mut T {
    let _size = const { core::mem::size_of::<T>() };

    const {
        assert!(core::mem::size_of::<T>() != 0, "Size of T must not be zero");
    }
    //flecs_ecs::core::table::field::ecs_field::panic_cold_explicit
    let index_usize = index as usize;
    //TODO should be soft asserts
    ecs_assert!(
        it.flags & sys::EcsIterIsValid != 0,
        FlecsErrorCode::InvalidParameter,
        "operation invalid before calling next()"
    );

    ecs_assert!(
        index >= 0,
        FlecsErrorCode::InvalidParameter,
        "invalid field index {}",
        index
    );
    ecs_assert!(
        index < it.field_count,
        FlecsErrorCode::InvalidParameter,
        "field index {} out of bounds",
        index
    );

    ecs_assert!(
        unsafe { sys::ecs_field_size(it, index) } == _size
            || unsafe { sys::ecs_field_size(it, index) } == 0,
        FlecsErrorCode::InvalidParameter,
        "mismatching size for field {}",
        index
    );

    let ptrs = it.ptrs;
    let offset = it.offset;

    if ptrs.is_null() || offset != 0 {
        return ecs_field_fallback(it, index);
    }

    // fast path: direct load
    let p = unsafe { *ptrs.add(index_usize) };

    if p.is_null() {
        return ecs_field_fallback(it, index);
    }

    // #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    // {
    //     // Make sure that address in ptrs array is the same as what this
    //     // function would have returned if no ptrs array was set.
    //     // not done due to const casting in rust
    //     // let temp_ptrs = it.ptrs;
    //     // it.ptrs = core::ptr::null_mut();
    //     // ecs_assert!(
    //     //     ptr == unsafe { sys::ecs_field_w_size(it, _size, index) },
    //     //     FlecsErrorCode::InternalError,
    //     //     "ptr address mismatch"
    //     // );
    //     // it.ptrs = temp_ptrs;
    // }

    // best case
    p as *mut T
}

#[inline(never)]
fn ecs_field_fallback<T>(it: &sys::ecs_iter_t, index: i8) -> *mut T {
    let index_usize = index as usize;
    let tr = unsafe { *it.trs.add(index_usize) };
    if tr.is_null() {
        /* We're just passing in a pointer to a value that may not be
         * a component on the entity (such as a pointer to a new value
         * in an on_replace hook). */
        return core::ptr::null_mut();
    }

    let tr = unsafe { &*tr };
    #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    {
        ecs_assert!(
            (unsafe { sys::ecs_id_get_flags(it.real_world, sys::ecs_field_id(it, index)) }
                & sys::EcsIdIsSparse)
                == 0,
            FlecsErrorCode::InvalidOperation,
            "use ecs_field_at to access fields for sparse components"
        );
    }

    let src = unsafe { *it.sources.add(index_usize) };
    let (table, row) = if src == 0 {
        (it.table, it.offset as usize)
    } else {
        let r = unsafe { &*sys::ecs_record_find(it.real_world, src) };
        let row = ecs_record_to_row(r.row) as usize;
        (r.table, row)
    };

    ecs_assert!(
        !table.is_null(),
        FlecsErrorCode::InternalError,
        "table is null"
    );
    ecs_assert!(
        tr.hdr.table == table,
        FlecsErrorCode::InternalError,
        "table mismatch in table record"
    );

    let column_index = tr.column;
    ecs_assert!(
        column_index != -1,
        FlecsErrorCode::NotAComponent,
        "only components can be fetched with fields"
    );
    ecs_assert!(
        column_index >= 0,
        FlecsErrorCode::InternalError,
        "column index {} is negative",
        column_index
    );
    ecs_assert!(
        (column_index as i32) < unsafe { sys::ecs_table_column_count(table) },
        FlecsErrorCode::InternalError,
        "column index {} out of bounds for table with {} columns",
        column_index,
        unsafe { sys::ecs_table_column_count(table) }
    );
    ecs_assert!(
        (row as i32) < unsafe { sys::ecs_table_count(table) }
            || (!it.query.is_null()
                && (unsafe { (*it.query).flags } & sys::EcsQueryMatchEmptyTables) != 0),
        FlecsErrorCode::InternalError,
        "row {} out of bounds for table with {} rows",
        row,
        unsafe { sys::ecs_table_count(table) }
    );

    unsafe { sys::ecs_table_get_column(table, column_index as i32, row as i32) as *mut T }
}

pub(crate) fn flecs_field_w_size(it: &sys::ecs_iter_t, _size: usize, index: i8) -> *mut c_void {
    let index_usize = index as usize;
    //TODO should be soft asserts
    ecs_assert!(
        it.flags & sys::EcsIterIsValid != 0,
        FlecsErrorCode::InvalidParameter,
        "operation invalid before calling next()"
    );
    ecs_assert!(
        index >= 0,
        FlecsErrorCode::InvalidParameter,
        "invalid field index {}",
        index
    );
    ecs_assert!(
        index < it.field_count,
        FlecsErrorCode::InvalidParameter,
        "field index {} out of bounds",
        index
    );
    ecs_assert!(
        _size != 0,
        FlecsErrorCode::InvalidParameter,
        "size must not be zero for field {}",
        index
    );

    ecs_assert!(
        unsafe { sys::ecs_field_size(it, index) } == _size
            || unsafe { sys::ecs_field_size(it, index) } == 0,
        FlecsErrorCode::InvalidParameter,
        "mismatching size for field {}",
        index
    );

    if !it.ptrs.is_null() && it.offset == 0 {
        let ptr = unsafe { *it.ptrs.add(index_usize) };
        if !ptr.is_null() {
            #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
            {
                // Make sure that address in ptrs array is the same as what this
                // function would have returned if no ptrs array was set.
                // not done due to const casting in rust
                // let temp_ptrs = it.ptrs;
                // it.ptrs = core::ptr::null_mut();
                // ecs_assert!(
                //     ptr == unsafe { sys::ecs_field_w_size(it, _size, index) },
                //     FlecsErrorCode::InternalError,
                //     "ptr address mismatch"
                // );
                // it.ptrs = temp_ptrs;
            }
            return ptr;
        }
    }

    let tr = unsafe { *it.trs.add(index_usize) };
    if tr.is_null() {
        ecs_assert!(
            !unsafe { sys::ecs_field_is_set(it, index) },
            FlecsErrorCode::InternalError,
            "field is set but no table record found"
        );
        return core::ptr::null_mut();
    }

    let tr = unsafe { &*tr };
    #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    {
        ecs_assert!(
            (unsafe { sys::ecs_id_get_flags(it.world, sys::ecs_field_id(it, index)) }
                & sys::EcsIdIsSparse)
                == 0,
            FlecsErrorCode::InvalidOperation,
            "use ecs_field_at to access fields for sparse components"
        );
    }

    let src = unsafe { *it.sources.add(index_usize) };
    let (table, row) = if src == 0 {
        (it.table, it.offset as usize)
    } else {
        let r = unsafe { &*sys::ecs_record_find(it.real_world, src) };
        let row = ecs_record_to_row(r.row) as usize;
        (r.table, row)
    };

    ecs_assert!(
        !table.is_null(),
        FlecsErrorCode::InternalError,
        "table is null"
    );
    ecs_assert!(
        tr.hdr.table == table,
        FlecsErrorCode::InternalError,
        "table mismatch in table record"
    );

    let column_index = tr.column;
    ecs_assert!(
        column_index != -1,
        FlecsErrorCode::NotAComponent,
        "only components can be fetched with fields"
    );
    ecs_assert!(
        column_index >= 0,
        FlecsErrorCode::InternalError,
        "column index {} is negative",
        column_index
    );
    ecs_assert!(
        (column_index as i32) < unsafe { sys::ecs_table_column_count(table) },
        FlecsErrorCode::InternalError,
        "column index {} out of bounds for table with {} columns",
        column_index,
        unsafe { sys::ecs_table_column_count(table) }
    );
    ecs_assert!(
        (row as i32) < unsafe { sys::ecs_table_count(table) }
            || (!it.query.is_null()
                && (unsafe { (*it.query).flags } & sys::EcsQueryMatchEmptyTables) != 0),
        FlecsErrorCode::InternalError,
        "row {} out of bounds for table with {} rows",
        row,
        unsafe { sys::ecs_table_count(table) }
    );

    unsafe { sys::ecs_table_get_column(table, column_index as i32, row as i32) }
}
