//! Table is a wrapper class that gives direct access to the component arrays of a table, the table data

mod field;
mod iter;

pub use field::{Field, FieldUntyped};
pub use iter::{TableIter, TableRowIter};

use core::{ffi::CStr, ffi::c_void, ptr::NonNull};

use crate::core::*;
use crate::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::{string::String, vec::Vec};

/// A wrapper class that gives direct access to the component arrays of a table, the table data
#[derive(Debug, Clone, Copy, Eq)]
pub struct Table<'a> {
    world: WorldRef<'a>,
    pub(crate) table: NonNull<sys::ecs_table_t>,
}

impl PartialEq for Table<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
    }
}

impl<'a> Table<'a> {
    /// Creates a wrapper around a table
    ///
    /// # Arguments
    ///
    /// * `world` - The world the table is in
    /// * `table` - The table to wrap
    pub fn new(world: impl WorldProvider<'a>, table: NonNull<sys::ecs_table_t>) -> Self {
        Self {
            world: world.world(),
            table,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TableRange<'a> {
    pub table: Table<'a>,
    offset: i32,
    count: i32,
}

impl<'a> TableRange<'a> {
    /// Creates a new table range
    ///
    /// # Arguments
    ///
    /// * `table` - The table to wrap
    /// * `offset` - The offset to start from
    /// * `count` - The count of the range
    ///
    /// # Returns
    ///
    /// A new table range
    pub fn new(table: Table<'a>, offset: i32, count: i32) -> Self {
        Self {
            table,
            offset,
            count,
        }
    }

    /// Creates a new table range from raw table ptr.
    ///
    /// # Arguments
    ///
    /// * `world` - The world the table is in
    /// * `table` - The table to wrap
    /// * `offset` - The offset to start from
    /// * `count` - The count of the range
    ///
    /// # Returns
    ///
    /// A new table range
    ///
    /// # Safety
    ///
    /// The world and table pointers must be valid
    pub(crate) fn new_raw(
        world: impl WorldProvider<'a>,
        table: NonNull<sys::ecs_table_t>,
        offset: i32,
        count: i32,
    ) -> Self {
        Self {
            table: Table::new(world, table),
            offset,
            count,
        }
    }
}

pub trait TableOperations<'a>: IntoTable {
    fn table(&self) -> Table<'a>;
    fn offset(&self) -> i32;
    fn world(&self) -> WorldRef<'a>;

    /// Returns the table count
fn count(&self) -> i32;

    /// Converts table type to string
fn to_string(&self) -> Option<String> {
        unsafe {
            let raw_ptr = sys::ecs_table_str(self.world().world_ptr(), self.table_ptr_mut());

            if raw_ptr.is_null() {
                return None;
            }

            let len = CStr::from_ptr(raw_ptr).to_bytes().len();

            Some(String::from_utf8_unchecked(Vec::from_raw_parts(
                raw_ptr as *mut u8,
                len,
                len,
            )))
        }
    }

    /// Returns the type of the table
fn archetype(&self) -> Archetype<'a> {
        let type_vec = unsafe { sys::ecs_table_get_type(self.table_ptr_mut()) };
        let slice = unsafe {
            core::slice::from_raw_parts((*type_vec).array as _, (*type_vec).count as usize)
        };
        let world = self.world();
        // Safety: we already know table_ptr is NonNull
        unsafe {
            Archetype::new_locked(
                world,
                slice,
                TableLock::new(world, NonNull::new_unchecked(self.table_ptr_mut())),
            )
        }
    }

    /// Find type index for (component) id
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component
    ///
    /// # Returns
    ///
    /// The index of the id in the table type, or `None` if the id is not found
fn find_type_index_id(&self, id: sys::ecs_id_t) -> Option<i32> {
        let index = unsafe {
            sys::ecs_table_get_type_index(self.world().world_ptr(), self.table_ptr_mut(), id)
        };
        if index == -1 { None } else { Some(index) }
    }

    /// Find type index for component type
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of the component
    ///
    /// # Returns
    ///
    /// The index of the component in the table type, or `None` if the component is not in the table
fn find_type_index<T: ComponentId>(&self) -> Option<i32> {
        self.find_type_index_id(T::id(self.world()))
    }

    /// Find type index for pair of component types
    ///
    /// # Arguments
    ///
    /// * `first` - First element of the pair
    /// * `second` - Second element of the pair
    ///
    /// # Returns
    ///
    /// The index of the pair in the table type, or `None` if the pair is not in the table
fn find_type_index_pair_ids(
        &self,
        first: impl Into<Entity>,
        second: impl Into<Entity>,
    ) -> Option<i32> {
        self.find_type_index_id(ecs_pair(*first.into(), *second.into()))
    }

    /// Find type index for pair of component types
    ///
    /// # Type parameters
    ///
    /// * `First` - The type of the first component
    /// * `Second` - The type of the second component
    ///
    /// # Returns
    ///
    /// The index of the pair in the table type, or `None` if the pair is not in the table
fn find_type_index_pair<First: ComponentId, Second: ComponentId>(&self) -> Option<i32> {
        let world = self.world();
        self.find_type_index_pair_ids(First::id(world), Second::id(world))
    }

    /// Find type index for pair of component types
    ///
    /// # Type parameters
    ///
    /// * `First` - The type of the first component
    ///
    /// # Arguments
    ///
    /// * `second` - The id of the second component
    ///
    /// # Returns
    ///
    /// The index of  the pair in the table type, or `None` if the pair is not in the table
fn find_type_index_first<First: ComponentId>(&self, second: impl Into<Entity>) -> Option<i32> {
        self.find_type_index_pair_ids(First::id(self.world()), second)
    }

    /// Find type index for pair of component types
    ///
    /// # Type parameters
    ///
    /// * `Second` - The type of the second component
    ///
    /// # Arguments
    ///
    /// * `first` - The id of the first component
    ///
    /// # Returns
    ///
    /// The index of  the pair in the table type, or `None` if the pair is not in the table
fn find_type_index_second<Second: ComponentId>(&self, first: impl Into<Entity>) -> Option<i32> {
        self.find_type_index_pair_ids(first, Second::id(self.world()))
    }

    /// Find index for (component) id in table type
    ///
    /// This operation returns the index of first occurrence of the id in the table type. The id may be a wildcard.
    /// The found id may be different from the provided id if it is a wildcard.
    ///
    /// This is a constant time operation.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component
    ///
    /// # Returns
    ///
    /// The index of the id in the table, or `None` if the id is not in the table
fn find_column_index_id(&self, id: sys::ecs_id_t) -> Option<i32> {
        let index = unsafe {
            sys::ecs_table_get_column_index(self.world().world_ptr(), self.table_ptr_mut(), id)
        };
        if index == -1 { None } else { Some(index) }
    }

    /// Find column index for component type in table
    ///
    /// This operation returns the index of first occurrence of the type in the table type.
    ///
    /// This is a constant time operation.
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of the component
    ///
    /// # Returns
    ///
    /// The index of the component in the table, or `None` if the component is not in the table
fn find_column_index<T: ComponentId>(&self) -> Option<i32> {
        self.find_column_index_id(T::id(self.world()))
    }

    /// Find index for pair of component types in table
    ///
    /// This operation returns the index of first occurrence of the pair in the table type.
    ///
    /// This is a constant time operation.
    ///
    /// # Type parameters
    ///
    /// * `First` - The type of the first component
    /// * `Second` - The type of the second component
    ///
    /// # Returns
    ///
    /// The index of the pair in the table, or `None` if the pair is not in the table
fn find_column_index_pair<First: ComponentId, Second: ComponentId>(&self) -> Option<i32> {
        let world = self.world();
        self.find_column_index_id(ecs_pair(First::id(world), Second::id(world)))
    }

    /// Find index for pair of component ids in table type
    ///
    /// This operation returns the index of first occurrence of the pair in the table type.
    ///
    /// This is a constant time operation.
    ///
    /// # Arguments
    ///
    /// * `first` - The id of the first component
    /// * `second` - The id of the second component
    ///
    /// # Returns
    ///
    /// The index of the pair in the table, or `None` if the pair is not in the table
fn find_column_index_pair_ids(
        &self,
        first: impl Into<Entity>,
        second: impl Into<Entity>,
    ) -> Option<i32> {
        self.find_column_index_id(ecs_pair(*first.into(), *second.into()))
    }

    /// Find index for pair of component types in table type
    ///
    /// # Type parameters
    ///
    /// * `First` - The type of the first component
    ///
    /// # Arguments
    ///
    /// * `second` - The id of the second component
    ///
    /// # Returns
    ///
    /// The index of the pair in the table, or `None` if the pair is not in the table
fn find_column_index_first<First: ComponentId>(
        &self,
        second: impl Into<Entity>,
    ) -> Option<i32> {
        self.find_column_index_pair_ids(First::id(self.world()), second)
    }

    /// Find index for pair of component types in table type
    ///
    /// # Type parameters
    ///
    /// * `Second` - The type of the second component
    ///
    /// # Arguments
    ///
    /// * `first` - The id of the first component
    ///
    /// # Returns
    ///
    /// The index of the pair in the table, or `None` if the pair is not in the table
fn find_column_index_second<Second: ComponentId>(
        &self,
        first: impl Into<Entity>,
    ) -> Option<i32> {
        self.find_column_index_pair_ids(first, Second::id(self.world()))
    }

    /// Test if table has component type
    ///
    /// This is a constant time operation.
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of the component
    ///
    /// # Returns
    ///
    /// True if the table has the component type, false otherwise
fn has_type<T: ComponentId>(&self) -> bool {
        self.find_type_index::<T>().is_some()
    }

    /// Test if table has (component) id
    ///
    /// This is a constant time operation.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component
    ///
    /// # Returns
    ///
    /// True if the table has the component id, false otherwise
fn has_type_id(&self, id: sys::ecs_id_t) -> bool {
        self.find_type_index_id(id).is_some()
    }

    /// Test if table has pair of component types
    ///
    /// This is a constant time operation.
    ///
    /// # Type parameters
    ///
    /// * `First` - The type of the first component
    /// * `Second` - The type of the second component
    ///
    /// # Returns
    ///
    /// True if the table has the pair of component types, false otherwise
fn has_pair<First: ComponentId, Second: ComponentId>(&self) -> bool {
        self.find_type_index_pair::<First, Second>().is_some()
    }

    /// Test if table has pair of component ids
    ///
    /// This is a constant time operation.
    ///
    /// # Arguments
    ///
    /// * `first` - The id of the first component
    /// * `second` - The id of the second component
    ///
    /// # Returns
    ///
    /// True if the table has the pair of component ids, false otherwise
fn has_pair_ids(&self, first: impl Into<Entity>, second: impl Into<Entity>) -> bool {
        self.find_type_index_pair_ids(first, second).is_some()
    }

    /// Get column, components array ptr from table by column index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the component
    ///
    /// # Returns
    ///
    /// Some(Pointer) to the column, or `None` if not a component
fn column_untyped(&self, index: i32) -> Option<*mut c_void> {
        let ptr = unsafe { sys::ecs_table_get_column(self.table_ptr_mut(), index, self.offset()) };
        if ptr.is_null() { None } else { Some(ptr) }
    }

    /// Get column, components array ptr from table by component type.
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of the component
    ///
    /// # Returns
    ///
    /// Some(Pointer) to the column, or `None` if not found
fn get_mut<T: ComponentId>(&self) -> Option<&mut [T]> {
        self.get_mut_untyped(T::id(self.world())).map(|ptr| unsafe {
            core::slice::from_raw_parts_mut(ptr as *mut T, (self.count()) as usize)
        })
    }

    /// Get column, components array ptr from table by component type.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component
    ///
    /// # Returns
    ///
    /// Some(Pointer) to the column, or `None` if not found
fn get_mut_untyped(&self, id: sys::ecs_id_t) -> Option<*mut c_void> {
        if let Some(index) = self.find_column_index_id(id) {
            self.column_untyped(index)
        } else {
            None
        }
    }

    /// Get column, components array ptr from table by pair ids.
    ///
    /// # Arguments
    ///
    /// * `first` - The id of the first component
    /// * `second` - The id of the second component
    ///
    /// # Returns
    ///
    /// Some(Pointer) to the column, or `None` if not found
fn get_pair_ids_mut_untyped(
        &self,
        first: impl Into<Entity>,
        second: impl Into<Entity>,
    ) -> Option<*mut c_void> {
        self.get_mut_untyped(ecs_pair(*first.into(), *second.into()))
    }

    /// Get column, components array ptr from table by pair of component types.
    ///
    /// # Type parameters
    ///
    /// * `First` - The type of the first component
    /// * `Second` - The type of the second component
    ///
    /// # Returns
    ///
    /// Some(Pointer) to the column, or `None` if not found
fn get_pair_mut_untyped<First: ComponentId, Second: ComponentId>(&self) -> Option<*mut c_void> {
        let world = self.world();
        self.get_pair_ids_mut_untyped(First::id(world), Second::id(world))
    }

    /// Get column size from table at the provided column index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the column
    ///
    /// # Returns
    ///
    /// The size of the column
fn column_size(&self, index: i32) -> usize {
        unsafe { sys::ecs_table_get_column_size(self.table_ptr_mut(), index) }
    }

    /// Return depth for table in tree for relationship type.
    /// Depth is determined by counting the number of targets encountered while traversing up the
    /// relationship tree for rel. Only acyclic relationships are supported.
    ///
    /// # Type parameters
    ///
    /// * `Rel` - The type of the relationship
    ///
    /// # Returns
    ///
    /// The depth of the relationship
fn depth<Rel: ComponentId>(&self) -> i32 {
        self.depth_id(Rel::id(self.world()))
    }

    /// Return depth for table in tree for relationship type.
    /// Depth is determined by counting the number of targets encountered while traversing up the
    /// relationship tree for rel. Only acyclic relationships are supported.
    ///
    /// # Arguments
    ///
    /// * `rel` - The id of the relationship
    ///
    /// # Returns
    ///
    /// The depth of the relationship
fn depth_id(&self, rel: impl Into<Entity>) -> i32 {
        unsafe {
            sys::ecs_table_get_depth(
                self.world().world_ptr_mut(),
                self.table_ptr_mut(),
                *rel.into(),
            )
        }
    }
}

impl<'a> TableOperations<'a> for Table<'a> {
    fn table(&self) -> Table<'a> {
        *self
    }

    fn offset(&self) -> i32 {
        0
    }

    fn world(&self) -> WorldRef<'a> {
        self.world
    }

    /// Returns the table count
fn count(&self) -> i32 {
        unsafe { sys::ecs_table_count(self.table_ptr_mut()) }
    }
}

impl<'a> TableOperations<'a> for TableRange<'a> {
    fn table(&self) -> Table<'a> {
        self.table
    }

    fn offset(&self) -> i32 {
        self.offset
    }

    fn world(&self) -> WorldRef<'a> {
        self.table.world
    }

    /// Returns the table range count
    ///
    /// # See also
    fn count(&self) -> i32 {
        self.count
    }
}

/// A lock on a [`Table`].
///
/// When a table is locked, modifications to it will throw an assert. When the
/// table is locked recursively, it will take an equal amount of unlock
/// operations to actually unlock the table.
///
/// Table locks can be used to build safe iterators where it is guaranteed that
/// the contents of a table are not modified while it is being iterated.
///
/// The operation only works when called on the world, and has no side effects
/// when called on a stage. The assumption is that when called on a stage,
/// operations are deferred already.
pub(crate) struct TableLock<'a> {
    world: WorldRef<'a>,
    table: NonNull<sys::ecs_table_t>,
}

impl<'a> TableLock<'a> {
    pub fn new(world: impl WorldProvider<'a>, table: NonNull<sys::ecs_table_t>) -> Self {
        unsafe { sys::ecs_table_lock(world.world_ptr_mut(), table.as_ptr()) };
        Self {
            world: world.world(),
            table,
        }
    }
}

impl Drop for TableLock<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }

        unsafe {
            sys::ecs_table_unlock(self.world.world_ptr_mut(), self.table.as_ptr());
        }
    }
}
