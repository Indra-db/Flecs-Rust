//! Table is a wrapper class that gives direct access to the component arrays of a table, the table data

use std::{ffi::CStr, os::raw::c_void};

use crate::sys::{
    ecs_table_count, ecs_table_get_column, ecs_table_get_column_index, ecs_table_get_column_size,
    ecs_table_get_depth, ecs_table_get_type, ecs_table_get_type_index, ecs_table_str,
};

use super::{
    archetype::Archetype,
    c_types::{EntityT, IdT, TableT, WorldT},
    component_registration::ComponentId,
    ecs_pair, IntoWorld, World,
};

/// A wrapper class that gives direct access to the component arrays of a table, the table data
#[derive(Debug)]
pub struct Table {
    world: *mut WorldT,
    table: *mut TableT,
}

impl Clone for Table {
    fn clone(&self) -> Self {
        Self {
            world: self.world,
            table: self.table,
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            world: std::ptr::null_mut(),
            table: std::ptr::null_mut(),
        }
    }
}

impl Table {
    /// Creates a wrapper around a table
    ///
    /// # Arguments
    ///
    /// * `world` - The world the table is in
    /// * `table` - The table to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `table::table`
    #[doc(alias = "table::table")]
    pub fn new(world: impl IntoWorld, table: *mut TableT) -> Self {
        Self {
            world: world.world_ptr_mut(),
            table,
        }
    }

    /// Converts table type to string
    ///
    /// # See also
    ///
    /// * C++ API: `table::str`
    #[doc(alias = "table::str")]
    pub fn to_string(&self) -> Option<String> {
        unsafe {
            let raw_ptr = ecs_table_str(self.world, self.table);

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
    ///
    /// # See also
    ///
    /// * C++ API: `table::type`
    #[doc(alias = "table::type")]
    pub fn archetype(&self) -> Archetype {
        Archetype::new(self.world, unsafe { ecs_table_get_type(self.table) })
    }

    /// Returns the table count
    ///
    /// # See also
    ///
    /// * C++ API: `table::count`
    #[doc(alias = "table::count")]
    pub fn count(&self) -> i32 {
        unsafe { ecs_table_count(self.table) }
    }

    /// Find type index for (component) id
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component
    ///
    /// # Returns
    ///
    /// The index of the id in the table type, or None if the id is not found
    ///
    /// # See also
    ///
    /// * C++ API: `table::type_index`
    #[doc(alias = "table::type_index")]
    pub fn find_type_index_id(&self, id: IdT) -> Option<i32> {
        let index = unsafe { ecs_table_get_type_index(self.world, self.table, id) };
        if index == -1 {
            None
        } else {
            Some(index)
        }
    }

    /// Find type index for component type
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of the component
    ///
    /// # Returns
    ///
    /// The index of the component in the table type, or None if the component is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::type_index`
    #[doc(alias = "table::type_index")]
    pub fn find_type_index<T: ComponentId>(&self) -> Option<i32> {
        self.find_type_index_id(T::get_id(self.world))
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
    /// The index of the pair in the table type, or None if the pair is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::type_index`
    #[doc(alias = "table::type_index")]
    pub fn find_type_index_pair_ids(&self, first: EntityT, second: EntityT) -> Option<i32> {
        self.find_type_index_id(ecs_pair(first, second))
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
    /// The index of the pair in the table type, or None if the pair is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::type_index`
    #[doc(alias = "table::type_index")]
    pub fn find_type_index_pair<First: ComponentId, Second: ComponentId>(&self) -> Option<i32> {
        self.find_type_index_pair_ids(First::get_id(self.world), Second::get_id(self.world))
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
    /// The index of the pair in the table type, or None if the pair is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::type_index`
    #[doc(alias = "table::type_index")]
    pub fn find_type_index_pair_second_id<First: ComponentId>(
        &self,
        second: EntityT,
    ) -> Option<i32> {
        self.find_type_index_pair_ids(First::get_id(self.world), second)
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
    /// The index of the id in the table, or None if the id is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::column_index`
    #[doc(alias = "table::column_index")]
    pub fn find_column_index_id(&self, id: IdT) -> Option<i32> {
        let index = unsafe { ecs_table_get_column_index(self.world, self.table, id) };
        if index == -1 {
            None
        } else {
            Some(index)
        }
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
    /// The index of the component in the table, or None if the component is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::column_index`
    #[doc(alias = "table::column_index")]
    pub fn find_column_index<T: ComponentId>(&self) -> Option<i32> {
        self.find_column_index_id(T::get_id(self.world))
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
    /// The index of the pair in the table, or None if the pair is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::column_index`
    #[doc(alias = "table::column_index")]
    pub fn find_column_index_pair<First: ComponentId, Second: ComponentId>(&self) -> Option<i32> {
        self.find_column_index_id(ecs_pair(
            First::get_id(self.world),
            Second::get_id(self.world),
        ))
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
    /// The index of the pair in the table, or None if the pair is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::column_index`
    #[doc(alias = "table::column_index")]
    pub fn find_column_index_pair_ids(&self, first: EntityT, second: EntityT) -> Option<i32> {
        self.find_column_index_id(ecs_pair(first, second))
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
    /// The index of the pair in the table, or None if the pair is not in the table
    ///
    /// # See also
    ///
    /// * C++ API: `table::column_index`
    #[doc(alias = "table::column_index")]
    pub fn find_column_index_pair_second_id<First: ComponentId>(
        &self,
        second: EntityT,
    ) -> Option<i32> {
        self.find_column_index_pair_ids(First::get_id(self.world), second)
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
    ///
    /// # See also
    ///
    /// * C++ API: `table::has`
    #[doc(alias = "table::has")]
    pub fn has_type<T: ComponentId>(&self) -> bool {
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
    ///
    /// # See also
    ///
    /// * C++ API: `table::has`
    #[doc(alias = "table::has")]
    pub fn has_type_id(&self, id: IdT) -> bool {
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
    ///
    /// # See also
    ///
    /// * C++ API: `table::has`
    #[doc(alias = "table::has")]
    pub fn has_pair<First: ComponentId, Second: ComponentId>(&self) -> bool {
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
    ///
    /// # See also
    ///
    /// * C++ API: `table::has`
    #[doc(alias = "table::has")]
    pub fn has_pair_ids(&self, first: EntityT, second: EntityT) -> bool {
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
    /// Some(Pointer) to the column, or None if not a component
    ///
    /// # See also
    ///
    /// * C++ API: `table::get_column`
    #[doc(alias = "table::get_column")]
    pub fn column_untyped(&self, index: i32) -> Option<*mut c_void> {
        let ptr = unsafe { ecs_table_get_column(self.table, index, 0) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Get column, components array ptr from table by component type.
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of the component
    ///
    /// # Returns
    ///
    /// Some(Pointer) to the column, or None if not found
    ///
    /// # See also
    ///
    /// * C++ API: `table::get`
    #[doc(alias = "table::get")]
    pub fn get_mut<T: ComponentId>(&self) -> Option<&mut [T]> {
        self.get_mut_untyped(T::get_id(self.world))
            .map(|ptr| unsafe {
                std::slice::from_raw_parts_mut(ptr as *mut T, self.count() as usize)
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
    /// Some(Pointer) to the column, or None if not found
    ///
    /// # See also
    ///
    /// * C++ API: `table::get`
    pub fn get_mut_untyped(&self, id: IdT) -> Option<*mut c_void> {
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
    /// Some(Pointer) to the column, or None if not found
    ///
    /// # See also
    ///
    /// * C++ API: `table::get`
    #[doc(alias = "table::get")]
    pub fn get_pair_ids_mut_untyped(&self, first: EntityT, second: EntityT) -> Option<*mut c_void> {
        self.get_mut_untyped(ecs_pair(first, second))
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
    /// Some(Pointer) to the column, or None if not found
    ///
    /// # See also
    ///
    /// * C++ API: `table::get`
    #[doc(alias = "table::get")]
    pub fn get_pair_mut_untyped<First: ComponentId, Second: ComponentId>(
        &self,
    ) -> Option<*mut c_void> {
        self.get_pair_ids_mut_untyped(First::get_id(self.world), Second::get_id(self.world))
    }

    //TODO pair generic

    /// Get column size from table at the provided column index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the column
    ///
    /// # Returns
    ///
    /// The size of the column
    ///
    /// # See also
    ///
    /// * C++ API: `table::column_size`
    #[doc(alias = "table::column_size")]
    pub fn column_size(&self, index: i32) -> usize {
        unsafe { ecs_table_get_column_size(self.table, index) }
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
    ///
    /// # See also
    ///
    /// * C++ API: `table::depth`
    #[doc(alias = "table::depth")]
    pub fn depth<Rel: ComponentId>(&self) -> i32 {
        self.depth_id(Rel::get_id(self.world))
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
    ///
    /// # See also
    ///
    /// * C++ API: `table::depth`
    #[doc(alias = "table::depth")]
    pub fn depth_id(&self, rel: EntityT) -> i32 {
        unsafe { ecs_table_get_depth(self.world, self.table, rel) }
    }

    /// Get raw table ptr
    pub fn table_ptr_mut(&self) -> *mut TableT {
        self.table
    }
}

#[derive(Debug, Default)]
pub struct TableRange {
    pub table: Table,
    offset: i32,
    count: i32,
}

impl Clone for TableRange {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            offset: self.offset,
            count: self.count,
        }
    }
}
impl TableRange {
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
    ///
    /// # See also
    ///
    /// * C++ API: `table_range::table_range`
    #[doc(alias = "table_range::table_range")]
    pub fn new(table: &Table, offset: i32, count: i32) -> Self {
        Self {
            table: table.clone(),
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
    pub(crate) fn new_raw(world: *mut WorldT, table: *mut TableT, offset: i32, count: i32) -> Self {
        Self {
            table: Table::new(&World::new_wrap_raw_world(world), table),
            offset,
            count,
        }
    }

    /// returns the offset which it starts from
    ///
    /// # See also
    ///
    /// * C++ API: `table_range::offset`
    #[doc(alias = "table_range::offset")]
    pub fn offset(&self) -> i32 {
        self.offset
    }

    /// returns the count of the range
    ///
    /// # See also
    ///
    /// * C++ API: `table_range::count`
    #[doc(alias = "table_range::count")]
    pub fn count(&self) -> i32 {
        self.count
    }

    /// Get column, components array ptr from table by column index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the component
    ///
    /// # Returns
    ///
    /// Some(Pointer) to the column, or None if not a component
    ///
    /// # See also
    ///
    /// * C++ API: `table::get_column`
    #[doc(alias = "table::get_column")]
    pub fn column_untyped(&self, index: i32) -> Option<*mut c_void> {
        let ptr = unsafe { ecs_table_get_column(self.table.table, index, self.offset) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
}
