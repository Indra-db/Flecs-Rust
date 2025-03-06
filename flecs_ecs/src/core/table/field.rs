//! Table column API.

use core::{
    ffi::c_void,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::core::*;

/// Wrapper class around an immutable table column.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct Field<'a, T> {
    pub(crate) slice_components: &'a [T],
    pub(crate) is_shared: bool,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) component_access: NonNull<ReadWriteComponentsMap>,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) id: Entity,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) table_id: u64,
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<'a, T> Drop for Field<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let component_access = self.component_access.as_mut();
            component_access.decrement_read(*self.id, self.table_id);
        }
    }
}

impl<'a, T> Field<'a, T> {
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
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) fn new(
        slice_components: &'a [T],
        is_shared: bool,
        id: Entity,
        table_id: u64,
        mut component_access: NonNull<ReadWriteComponentsMap>,
    ) -> Self {
        unsafe {
            let component_access = component_access.as_mut();
            component_access.increment_read(*id, table_id);
        }
        Self {
            slice_components,
            is_shared,
            component_access,
            id,
            table_id,
        }
    }

    pub fn drop(self) {}

    /// whether the column / component is shared.
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<T: ComponentId> Deref for Field<'_, T> {
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
pub struct FieldMut<'a, T> {
    pub(crate) slice_components: &'a mut [T],
    pub(crate) is_shared: bool,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) component_access: NonNull<ReadWriteComponentsMap>,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) id: Entity,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) table_id: u64,
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<'a, T> Drop for FieldMut<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let component_access = self.component_access.as_mut();
            component_access.clear_write(*self.id, self.table_id);
        }
    }
}

impl<'a, T> FieldMut<'a, T> {
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
        id: Entity,
        table_id: u64,
        mut component_access: NonNull<ReadWriteComponentsMap>,
    ) -> Self {
        unsafe {
            let component_access = component_access.as_mut();
            component_access.set_write(*id, table_id);
        }
        Self {
            slice_components,
            is_shared,
            component_access,
            id,
            table_id,
        }
    }

    pub fn drop(self) {}

    /// whether the column / component is shared.
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<T: ComponentId> Deref for FieldMut<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice_components
    }
}

impl<T: ComponentId> DerefMut for FieldMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice_components
    }
}

pub struct FieldAt<'a, T> {
    pub(crate) component: &'a T,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) component_access: NonNull<ReadWriteComponentsMap>,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) id: Entity,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) table_id: u64,
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<'a, T> Drop for FieldAt<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let component_access = self.component_access.as_mut();
            component_access.decrement_read(*self.id, self.table_id);
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
        id: Entity,
        table_id: u64,
        mut component_access: NonNull<ReadWriteComponentsMap>,
    ) -> Self {
        unsafe {
            let component_access = component_access.as_mut();
            component_access.increment_read(*id, table_id);
        }
        Self {
            component,
            component_access,
            id,
            table_id,
        }
    }
}

pub struct FieldAtMut<'a, T> {
    pub(crate) component: &'a mut T,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) component_access: NonNull<ReadWriteComponentsMap>,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) id: Entity,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) table_id: u64,
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<'a, T> Drop for FieldAtMut<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let component_access = self.component_access.as_mut();
            component_access.clear_write(*self.id, self.table_id);
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
        id: Entity,
        table_id: u64,
        mut component_access: NonNull<ReadWriteComponentsMap>,
    ) -> Self {
        unsafe {
            let component_access = component_access.as_mut();
            component_access.set_write(*id, table_id);
        }
        Self {
            component,
            component_access,
            id,
            table_id,
        }
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
/// * C++ API: `untyped_field::untyped_column`
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
