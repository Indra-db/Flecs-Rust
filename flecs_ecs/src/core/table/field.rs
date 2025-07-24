//! Table column API.

use crate::core::*;
use crate::sys;
use core::ffi::c_void;
use core::ops::Index;
use core::ops::IndexMut;

// TODO I can probably return two different field types, one for shared and one for non-shared
// then I can customize the index behavior

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldIndex(pub(crate) usize);

impl Into<usize> for FieldIndex {
    #[inline(always)]
    fn into(self) -> usize {
        self.0
    }
}

/// Wrapper class around a table column with immutable access.
///
/// # Type parameters
///
/// * `T`: The type of the column.
#[derive(Debug)]
pub struct Field<'a, T> {
    pub(crate) slice_components: *const T,
    pub(crate) is_shared: bool,
    pub(crate) _marker: core::marker::PhantomData<&'a T>,
}

impl<'a, T> Field<'a, T> {
    pub(crate) fn new(slice_components: *const T, is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<'a, T> Index<FieldIndex> for Field<'a, T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: FieldIndex) -> &'a Self::Output {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            self.is_shared && idx.0 == 0,
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { &*self.slice_components.add(idx.0) }
    }
}

impl<'a, T> Index<usize> for Field<'a, T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: usize) -> &Self::Output {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            self.is_shared && idx == 0,
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { self.slice_components.add(idx).as_ref().unwrap() }
    }
}

/// Wrapper class around a table column with mutable access.
///
/// # Type parameters
///
/// * `T`: The type of the column.
pub struct FieldMut<'a, T> {
    pub(crate) slice_components: *mut T,
    pub(crate) is_shared: bool,
    pub(crate) _marker: core::marker::PhantomData<&'a mut T>,
}

impl<'a, T> FieldMut<'a, T> {
    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    pub fn new(slice_components: *mut T, is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
            _marker: core::marker::PhantomData,
        }
    }

    /// whether the column / component is shared.
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<'a, T> Index<FieldIndex> for FieldMut<'a, T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: FieldIndex) -> &T {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            self.is_shared && idx.0 == 0,
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { &*self.slice_components.add(idx.0) }
    }
}

impl<'a, T> IndexMut<FieldIndex> for FieldMut<'a, T> {
    #[inline(always)]
    fn index_mut(&mut self, idx: FieldIndex) -> &mut T {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            self.is_shared && idx.0 == 0,
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { &mut *self.slice_components.add(idx.0) }
    }
}

impl<'a, T> Index<usize> for FieldMut<'a, T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: usize) -> &T {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            self.is_shared && idx == 0,
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { self.slice_components.add(idx).as_ref().unwrap() }
    }
}

impl<'a, T> IndexMut<usize> for FieldMut<'a, T> {
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut T {
        // Safety: This index can only be obtained from `it.iter`
        ecs_assert!(
            self.is_shared && idx == 0,
            FlecsErrorCode::InvalidParameter,
            "Field is shared, cannot index above index 0"
        );
        unsafe { self.slice_components.add(idx).as_mut().unwrap() }
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

/// copy of `ecs_field_w_size` from flecs_sys. Rewriting in rust for inlining.
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
pub fn ecs_field<T>(it: &sys::ecs_iter_t, index: i8) -> *mut T {
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
            || unsafe { sys::ecs_field_size(it, index) } != 0,
        FlecsErrorCode::InvalidParameter,
        "mismatching size for field {}",
        index
    );

    if !it.ptrs.is_null() && it.offset == 0 {
        let ptr = unsafe { *it.ptrs.add(index_usize) };
        if !ptr.is_null() {
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
            return ptr as *mut T;
        }
    }

    //return std::ptr::null_mut();

    ecs_field_fallback(it, index)
}

#[cold]
#[inline(never)]
fn ecs_field_fallback<T>(it: &sys::ecs_iter_t, index: i8) -> *mut T {
    let index_usize = index as usize;
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

    unsafe { sys::ecs_table_get_column(table, column_index as i32, row as i32) as *mut T }
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn ecs_field_w_size(it: &sys::ecs_iter_t, _size: usize, index: i8) -> *mut c_void {
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
            || unsafe { sys::ecs_field_size(it, index) } != 0,
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
