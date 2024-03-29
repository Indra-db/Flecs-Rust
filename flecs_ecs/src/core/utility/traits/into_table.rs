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
