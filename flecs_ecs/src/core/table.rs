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

    /// Find index of (component) id in table
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of the component
    ///
    /// # See also
    ///
    /// * C++ API: `table::search`
    pub fn find_component_id_index<T: CachedComponentData>(&self) -> Option<i32> {
        self.find_component_id_index_by_id(T::get_id(self.world))
    }

    pub fn find_component_id_index_by_id(&self, id: IdT) -> Option<i32> {
        let mut out_id: u64 = 0;
        let id_out_ptr: *mut u64 = &mut out_id;
        let found_index = unsafe { ecs_search(self.world, self.table, id, id_out_ptr) };
        if found_index == -1 {
            None
        } else {
            Some(found_index)
        }
    }

    pub fn find_pair_index<First: CachedComponentData, Second: CachedComponentData>(
        &self,
    ) -> Option<i32> {
        self.find_pair_index_by_ids(First::get_id(self.world), Second::get_id(self.world))
    }

    pub fn find_pair_index_by_ids(&self, first: EntityT, second: EntityT) -> Option<i32> {
        self.find_component_id_index_by_id(ecs_pair(first, second))
    }

    pub fn contains_type<T: CachedComponentData>(&self) -> bool {
        self.find_component_id_index::<T>().is_some()
    }

    pub fn contains_type_id(&self, id: IdT) -> bool {
        self.find_component_id_index_by_id(id).is_some()
    }

    pub fn contains_pair<First: CachedComponentData, Second: CachedComponentData>(&self) -> bool {
        self.find_pair_index::<First, Second>().is_some()
    }

    pub fn contains_pair_by_ids(&self, first: EntityT, second: EntityT) -> bool {
        self.find_pair_index_by_ids(first, second).is_some()
    }

    pub fn get_component_array_ptr_by_column_index(&self, index: i32) -> *mut c_void {
        unsafe { ecs_table_get_column(self.table, index, 0) }
    }

    pub fn get_component_array_ptr<T: CachedComponentData>(&self) -> Option<*mut T> {
        if let Some(ptr) = self.get_component_array_ptr_by_id(T::get_id(self.world)) {
            Some(ptr as *mut T)
        } else {
            None
        }
    }

    pub fn get_component_array_ptr_by_id(&self, id: IdT) -> Option<*mut c_void> {
        if let Some(index) = self.find_component_id_index_by_id(id) {
            Some(self.get_component_array_ptr_by_column_index(index))
        } else {
            None
        }
    }

    pub fn get_component_array_ptr_by_pair(
        &self,
        first: EntityT,
        second: EntityT,
    ) -> Option<*mut c_void> {
        self.get_component_array_ptr_by_id(ecs_pair(first, second))
    }

    //TODO pair generic

    pub fn get_column_size(&self, column_index: i32) -> usize {
        unsafe { ecs_table_get_column_size(self.table, column_index) }
    }

    pub fn get_depth_for_relationship<Rel: CachedComponentData>(&self) -> i32 {
        self.get_depth_for_relationship_id(Rel::get_id(self.world))
    }

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
    pub fn new(table: Table, offset: i32, count: i32) -> Self {
        Self {
            table,
            offset,
            count,
        }
    }

    pub(crate) fn new_raw(world: *mut WorldT, table: *mut TableT, offset: i32, count: i32) -> Self {
        Self {
            table: Table::new(&World::new_wrap_raw_world(world), table),
            offset,
            count,
        }
    }

    pub fn get_offset(&self) -> i32 {
        self.offset
    }

    pub fn get_count(&self) -> i32 {
        self.count
    }

    pub fn get_component_array_ptr_by_column_index(&self, index: i32) -> *mut c_void {
        unsafe { ecs_table_get_column(self.table.table, index, self.offset) }
    }
}
