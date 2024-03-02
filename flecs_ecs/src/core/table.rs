//! Table is a wrapper class that gives direct access to the component arrays of a table, the table data

use std::{ffi::CStr, os::raw::c_void};

use super::{
    archetype::Archetype,
    c_binding::bindings::{
        ecs_search, ecs_table_count, ecs_table_get_column, ecs_table_get_column_size,
        ecs_table_get_depth, ecs_table_get_type, ecs_table_str,
    },
    c_types::{EntityT, IdT, TableT, WorldT},
    component_registration::CachedComponentData,
    ecs_pair, World,
};

/// A wrapper class that gives direct access to the component arrays of a table, the table data
#[derive(Debug)]
pub struct Table {
    world: *mut WorldT,
    table: *mut TableT,
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
    pub fn new(world: &World, table: *mut TableT) -> Self {
        Self {
            world: world.raw_world,
            table,
        }
    }

    /// Converts table type to string
    ///
    /// # See also
    ///
    /// * C++ API: `table::str`
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
    pub fn get_type(&self) -> Archetype {
        Archetype::new(self.world, unsafe { ecs_table_get_type(self.table) })
    }

    /// Returns the table count
    ///
    /// # See also
    ///
    /// * C++ API: `table::count`
    pub fn get_count(&self) -> i32 {
        unsafe { ecs_table_count(self.table) }
    }

    /// Find index for component type in table
    ///
    /// This operation returns the index of first occurrance of the type in the table type.
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
    /// * C++ API: `table::search`
    pub fn find_component_index<T: CachedComponentData>(&self) -> Option<i32> {
        self.find_component_id_index(T::get_id(self.world))
    }

    /// Find index for (component) id in table type
    ///
    /// This operation returns the index of first occurrance of the id in the table type. The id may be a wildcard.
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
    /// * C++ API: `table::search`
    pub fn find_component_id_index(&self, id: IdT) -> Option<i32> {
        let mut out_id: u64 = 0;
        let id_out_ptr: *mut u64 = &mut out_id;
        let found_index = unsafe { ecs_search(self.world, self.table, id, id_out_ptr) };
        if found_index == -1 {
            None
        } else {
            Some(found_index)
        }
    }

    /// Find index for pair of component types in table
    ///
    /// This operation returns the index of first occurrance of the pair in the table type.
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
    /// * C++ API: `table::search`
    pub fn find_pair_index<First: CachedComponentData, Second: CachedComponentData>(
        &self,
    ) -> Option<i32> {
        self.find_pair_index_by_ids(First::get_id(self.world), Second::get_id(self.world))
    }

    /// Find index for pair of component ids in table type
    ///
    /// This operation returns the index of first occurrance of the pair in the table type.
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
    /// * C++ API: `table::search`
    pub fn find_pair_index_by_ids(&self, first: EntityT, second: EntityT) -> Option<i32> {
        self.find_component_id_index(ecs_pair(first, second))
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
    pub fn has_type<T: CachedComponentData>(&self) -> bool {
        self.find_component_index::<T>().is_some()
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
    pub fn has_type_id(&self, id: IdT) -> bool {
        self.find_component_id_index(id).is_some()
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
    pub fn has_pair<First: CachedComponentData, Second: CachedComponentData>(&self) -> bool {
        self.find_pair_index::<First, Second>().is_some()
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
    pub fn has_pair_by_ids(&self, first: EntityT, second: EntityT) -> bool {
        self.find_pair_index_by_ids(first, second).is_some()
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
    /// * C++ API: `table::get_by_index`
    pub fn get_component_array_ptr_by_column_index(&self, index: i32) -> Option<*mut c_void> {
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
    pub fn get_component_array_ptr<T: CachedComponentData>(&self) -> Option<*mut T> {
        if let Some(ptr) = self.get_component_array_ptr_by_id(T::get_id(self.world)) {
            Some(ptr as *mut T)
        } else {
            None
        }
    }

    pub fn get_component_array_ptr_by_id(&self, id: IdT) -> Option<*mut c_void> {
        if let Some(index) = self.find_component_id_index(id) {
            if let Some(ptr) = self.get_component_array_ptr_by_column_index(index) {
                Some(ptr)
            } else {
                None
            }
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
    pub fn get_component_array_ptr_by_pair_ids(
        &self,
        first: EntityT,
        second: EntityT,
    ) -> Option<*mut c_void> {
        self.get_component_array_ptr_by_id(ecs_pair(first, second))
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
    pub fn get_component_array_ptr_by_pair<
        First: CachedComponentData,
        Second: CachedComponentData,
    >(
        &self,
    ) -> Option<*mut c_void> {
        self.get_component_array_ptr_by_pair_ids(
            First::get_id(self.world),
            Second::get_id(self.world),
        )
    }

    //TODO pair generic

    /// Get column size from table at the provided column index.
    ///
    /// # Arguments
    ///
    /// * `column_index` - The index of the column
    ///
    /// # Returns
    ///
    /// The size of the column
    ///
    /// # See also
    ///
    /// * C++ API: `table::column_size`
    pub fn get_column_size(&self, column_index: i32) -> usize {
        unsafe { ecs_table_get_column_size(self.table, column_index) }
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
    pub fn get_depth_for_relationship<Rel: CachedComponentData>(&self) -> i32 {
        self.get_depth_for_relationship_id(Rel::get_id(self.world))
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
    pub fn get_depth_for_relationship_id(&self, rel: EntityT) -> i32 {
        unsafe { ecs_table_get_depth(self.world, self.table, rel) }
    }
}

#[derive(Debug, Default)]
pub struct TableRange {
    pub table: Table,
    offset: i32,
    count: i32,
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
    pub fn new(table: Table, offset: i32, count: i32) -> Self {
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
    pub fn get_offset(&self) -> i32 {
        self.offset
    }

    /// returns the count of the range
    ///
    /// # See also
    ///
    /// * C++ API: `table_range::count`
    pub fn get_count(&self) -> i32 {
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
    /// * C++ API: `table::get_by_index`
    pub fn get_component_array_ptr_by_column_index(&self, index: i32) -> Option<*mut c_void> {
        let ptr = unsafe { ecs_table_get_column(self.table.table, index, self.offset) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
}
