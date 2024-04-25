use crate::core::*;
use crate::sys;

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

impl IntoTable for Table<'_> {
    #[inline]
    fn table_ptr_mut(&self) -> *mut TableT {
        self.table_ptr_mut()
    }
}

impl IntoTable for TableRange<'_> {
    #[inline]
    fn table_ptr_mut(&self) -> *mut TableT {
        self.table.table_ptr_mut()
    }
}

pub trait IntoTableRange {
    fn table_range(&self) -> TableRange;
    fn table_range_raw(&self) -> sys::ecs_table_range_t;
}

impl IntoTableRange for TableRange<'_> {
    #[inline]
    fn table_range(&self) -> TableRange {
        *self
    }

    #[inline]
    fn table_range_raw(&self) -> sys::ecs_table_range_t {
        sys::ecs_table_range_t {
            table: self.table.table_ptr_mut(),
            offset: self.offset(),
            count: self.count(),
        }
    }
}

impl IntoTableRange for Table<'_> {
    #[inline]
    fn table_range(&self) -> TableRange {
        TableRange::new(*self, 0, self.count())
    }

    #[inline]
    fn table_range_raw(&self) -> sys::ecs_table_range_t {
        sys::ecs_table_range_t {
            table: self.table_ptr_mut(),
            offset: 0,
            count: self.count(),
        }
    }
}
