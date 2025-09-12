use crate::core::*;
use crate::sys;

pub trait IntoTable {
    fn table_ptr_mut(&self) -> *mut sys::ecs_table_t;
}

impl IntoTable for *mut sys::ecs_table_t {
    #[inline]
    fn table_ptr_mut(&self) -> *mut sys::ecs_table_t {
        *self
    }
}

impl IntoTable for *const sys::ecs_table_t {
    #[inline]
    fn table_ptr_mut(&self) -> *mut sys::ecs_table_t {
        *self as *mut sys::ecs_table_t
    }
}

impl IntoTable for Table<'_> {
    #[inline]
    fn table_ptr_mut(&self) -> *mut sys::ecs_table_t {
        self.table.as_ptr()
    }
}

impl IntoTable for TableRange<'_> {
    #[inline]
    fn table_ptr_mut(&self) -> *mut sys::ecs_table_t {
        self.table.table.as_ptr()
    }
}

pub trait IntoTableRange {
    fn range(&self) -> TableRange<'_>;
    fn range_raw(&self) -> sys::ecs_table_range_t;
}

impl IntoTableRange for TableRange<'_> {
    #[inline]
    fn range(&self) -> TableRange<'_> {
        *self
    }

    #[inline]
    fn range_raw(&self) -> sys::ecs_table_range_t {
        sys::ecs_table_range_t {
            table: self.table.table.as_ptr(),
            offset: self.offset(),
            count: self.count(),
        }
    }
}

impl IntoTableRange for Table<'_> {
    #[inline]
    fn range(&self) -> TableRange<'_> {
        TableRange::new(*self, 0, self.count())
    }

    #[inline]
    fn range_raw(&self) -> sys::ecs_table_range_t {
        sys::ecs_table_range_t {
            table: self.table.as_ptr(),
            offset: 0,
            count: self.count(),
        }
    }
}
