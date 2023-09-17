use std::os::raw::c_void;

use libc::strlen;

use super::{
    archetype::Archetype,
    c_binding::bindings::{
        ecs_search, ecs_table_count, ecs_table_get_column, ecs_table_get_column_size,
        ecs_table_get_depth, ecs_table_get_type, ecs_table_str,
    },
    c_types::{EntityT, IdT, TableT, WorldT},
    component::CachedComponentData,
    utility::functions::ecs_pair,
};

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
    pub fn new(world: *mut WorldT, table: *mut TableT) -> Self {
        Self { world, table }
    }

    pub fn to_string(&self) -> Option<String> {
        unsafe {
            let raw_ptr = ecs_table_str(self.world, self.table);

            if raw_ptr.is_null() {
                return None;
            }

            let len = strlen(raw_ptr) as usize;
            Some(String::from_utf8_unchecked(Vec::from_raw_parts(
                raw_ptr as *mut u8,
                len,
                len,
            )))
        }
    }

    pub fn get_type(&self) -> Archetype {
        Archetype::new(self.world, unsafe { ecs_table_get_type(self.table) })
    }

    pub fn get_count(&self) -> i32 {
        unsafe { ecs_table_count(self.table) }
    }

    pub fn find_component_id_index<T: CachedComponentData>(&self) -> i32 {
        self.find_component_id_index_by_id(T::get_id(self.world))
    }

    pub fn find_component_id_index_by_id(&self, id: IdT) -> i32 {
        let mut out_id: u64 = 0;
        let id_out_ptr: *mut u64 = &mut out_id;
        unsafe { ecs_search(self.world, self.table, id, id_out_ptr) }
    }

    pub fn find_pair_index<First: CachedComponentData, Second: CachedComponentData>(&self) -> i32 {
        self.find_pair_index_by_ids(First::get_id(self.world), Second::get_id(self.world))
    }

    pub fn find_pair_index_by_ids(&self, first: EntityT, second: EntityT) -> i32 {
        self.find_component_id_index_by_id(ecs_pair(first, second))
    }

    pub fn contains_type<T: CachedComponentData>(&self) -> bool {
        self.find_component_id_index::<T>() != -1
    }

    pub fn contains_type_id(&self, id: IdT) -> bool {
        self.find_component_id_index_by_id(id) != -1
    }

    pub fn contains_pair<First: CachedComponentData, Second: CachedComponentData>(&self) -> bool {
        self.find_pair_index::<First, Second>() != -1
    }

    pub fn contains_pair_by_ids(&self, first: EntityT, second: EntityT) -> bool {
        self.find_pair_index_by_ids(first, second) != -1
    }

    pub fn get_component_array_ptr_by_column_index(&self, index: i32) -> *mut c_void {
        unsafe { ecs_table_get_column(self.table, index, 0) }
    }

    pub fn get_component_array_ptr<T: CachedComponentData>(&self) -> *mut T {
        self.get_component_array_ptr_by_id(T::get_id(self.world)) as *mut T
    }

    pub fn get_component_array_ptr_by_id(&self, id: IdT) -> *mut c_void {
        let index = self.find_component_id_index_by_id(id);
        if index == -1 {
            std::ptr::null_mut()
        } else {
            self.get_component_array_ptr_by_column_index(index)
        }
    }

    pub fn get_component_array_ptr_by_pair(&self, first: EntityT, second: EntityT) -> *mut c_void {
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

struct TableRange {
    pub table: Table,
    offset: i32,
    count: i32,
}

impl Default for TableRange {
    fn default() -> Self {
        Self {
            table: Table::default(),
            offset: 0,
            count: 0,
        }
    }
}

impl TableRange {
    pub fn new(table: Table, offset: i32, count: i32) -> Self {
        Self {
            table,
            offset,
            count,
        }
    }

    pub fn new_raw(world: *mut WorldT, table: *mut TableT, offset: i32, count: i32) -> Self {
        Self {
            table: Table::new(world, table),
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
