use flecs_ecs_sys::ecs_table_range_t;

use crate::core::{Table, TableRange, TableT};

pub trait IntoTable {
    fn table_ptr_mut(&self) -> *mut TableT;
}

impl IntoTable for *mut TableT {
    #[inline]
    fn table_ptr_mut(&self) -> *mut TableT {
        *self
    }
}

impl IntoTable for *const TableT {
    #[inline]
    fn table_ptr_mut(&self) -> *mut TableT {
        *self as *mut TableT
    }
}

impl<'a> IntoTable for Table<'a> {
    #[inline]
    fn table_ptr_mut(&self) -> *mut TableT {
        self.table_ptr_mut()
    }
}

impl<'a> IntoTable for TableRange<'a> {
    #[inline]
    fn table_ptr_mut(&self) -> *mut TableT {
        self.table.table_ptr_mut()
    }
}

pub trait IntoTableRange {
    fn table_range(&self) -> TableRange;
    fn table_range_raw(&self) -> ecs_table_range_t;
}

impl<'a> IntoTableRange for TableRange<'a> {
    #[inline]
    fn table_range(&self) -> TableRange {
        self.clone()
    }

    #[inline]
    fn table_range_raw(&self) -> ecs_table_range_t {
        ecs_table_range_t {
            table: self.table.table_ptr_mut(),
            offset: self.offset(),
            count: self.count(),
        }
    }
}

impl<'a> IntoTableRange for Table<'a> {
    #[inline]
    fn table_range(&self) -> TableRange {
        TableRange::new(self, 0, self.count())
    }

    #[inline]
    fn table_range_raw(&self) -> ecs_table_range_t {
        ecs_table_range_t {
            table: self.table_ptr_mut(),
            offset: 0,
            count: self.count(),
        }
    }
}
