use flecs_ecs_sys::ecs_table_range_t;

use crate::core::{Table, TableRange, TableT};

pub trait IntoTable {
    fn get_table(&self) -> *mut TableT;
}

impl IntoTable for *mut TableT {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        *self
    }
}

impl IntoTable for *const TableT {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        *self as *mut TableT
    }
}

impl IntoTable for Table {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        self.get_raw_table()
    }
}

impl IntoTable for TableRange {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        self.table.get_raw_table()
    }
}

pub trait IntoTableRange {
    fn get_table_range(&self) -> TableRange;
    fn get_table_range_raw(&self) -> ecs_table_range_t;
}

impl IntoTableRange for TableRange {
    #[inline]
    fn get_table_range(&self) -> TableRange {
        self.clone()
    }

    #[inline]
    fn get_table_range_raw(&self) -> ecs_table_range_t {
        ecs_table_range_t {
            table: self.table.get_raw_table(),
            offset: self.get_offset(),
            count: self.get_count(),
        }
    }
}

impl IntoTableRange for Table {
    #[inline]
    fn get_table_range(&self) -> TableRange {
        TableRange::new(self, 0, self.get_count())
    }

    #[inline]
    fn get_table_range_raw(&self) -> ecs_table_range_t {
        ecs_table_range_t {
            table: self.get_raw_table(),
            offset: 0,
            count: self.get_count(),
        }
    }
}
