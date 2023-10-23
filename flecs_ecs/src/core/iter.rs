use std::{ffi::CString, os::raw::c_void};

use crate::{
    core::{
        c_binding::bindings::{
            ecs_field_is_set, ecs_field_w_size, ecs_iter_str, ecs_table_has_module,
        },
        utility::errors::FlecsErrorCode,
    },
    ecs_assert,
};

use super::{
    c_binding::bindings::{
        ecs_field_column_index, ecs_field_id, ecs_field_is_readonly, ecs_field_is_self,
        ecs_field_size, ecs_field_src, ecs_id_is_pair, ecs_query_changed, ecs_query_skip,
        ecs_table_get_type,
    },
    c_types::{IdT, IterT},
    column::{Column, UntypedColumn},
    component_registration::{CachedComponentData, ComponentType},
    entity::Entity,
    id::Id,
    table::{Table, TableRange},
    utility::{functions::ecs_has_pair, types::FTime},
    world::World,
    Type::Type,
};

pub struct Iter<'a> {
    iter: &'a mut IterT,
    begin: usize,
    end: usize,
    current: usize,
}

impl<'a> Iter<'a> {
    /// Constructs iterator from C iterator object
    /// this operation is typically not invoked directly by the user
    ///
    /// # Arguments
    ///
    /// * `iter` - C iterator
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointer
    pub unsafe fn new(iter: &'a mut IterT) -> Self {
        let end = iter.count as usize;
        Self {
            iter,
            begin: 0,
            end,
            current: 0,
        }
    }

    /// # C++ API equivalent
    ///
    /// `iter::system`
    pub fn system(&self) -> Entity {
        Entity::new_from_existing(self.iter.world, self.iter.system)
    }

    /// # C++ API equivalent
    ///
    /// `iter::event`
    pub fn event(&self) -> Entity {
        Entity::new_from_existing(self.iter.world, self.iter.event)
    }

    /// # C++ API equivalent
    ///
    /// `iter::event_id`
    pub fn event_id(&self) -> Id {
        Id::new_from_existing(self.iter.world, self.iter.event_id)
    }

    /// # C++ API equivalent
    ///
    /// `iter::world`
    pub fn get_world(&self) -> World {
        World::new_from_world(self.iter.world)
    }

    /// Obtain mutable handle to entity being iterated over.
    ///
    /// # Arguments
    ///
    /// * `row` - Row being iterated over
    ///
    /// # C++ API equivalent
    ///
    /// `iter::entity`
    pub fn entity(&self, row: usize) -> Entity {
        unsafe { Entity::new_from_existing(self.iter.world, *self.iter.entities.add(row)) }
    }

    /// # C++ API equivalent
    ///
    /// `iter::c_ptr`
    pub fn get_raw_ref(&mut self) -> &mut IterT {
        self.iter
    }

    /// # C++ API equivalent
    ///
    /// `iter::count`
    pub fn count(&self) -> usize {
        self.iter.count as usize
    }

    /// # C++ API equivalent
    ///
    /// `iter::delta_time`
    pub fn get_delta_time(&self) -> FTime {
        self.iter.delta_time
    }

    /// # C++ API equivalent
    ///
    /// `iter::delta_system_time`
    pub fn get_delta_system_time(&self) -> FTime {
        self.iter.delta_system_time
    }

    /// # C++ API equivalent
    ///
    /// `iter::type`
    pub fn get_type(&self) -> Type {
        unsafe { Type::new(self.iter.world, ecs_table_get_type(self.iter.table)) }
    }

    /// # C++ API equivalent
    ///
    /// `iter::table`
    pub fn get_table(&self) -> Table {
        Table::new(self.iter.world, self.iter.table)
    }

    /// # C++ API equivalent
    ///
    /// `iter::range`
    pub fn get_table_range(&mut self) -> TableRange {
        let iter: &mut IterT = self.iter;
        TableRange::new_raw(iter.world, iter.table, iter.offset, iter.count)
    }

    /// # Returns
    ///
    /// returns true if current type is a module or it contains module contents
    ///
    /// # C++ API equivalent
    ///
    /// `iter::has_module`
    pub fn has_module(&self) -> bool {
        unsafe { ecs_table_has_module(self.iter.table) }
    }

    /// Access ctx.
    /// ctx contains the context pointer assigned to a system
    ///
    /// # C++ API equivalent
    ///
    /// `iter::ctx`
    pub fn get_context_ptr<T: CachedComponentData>(&mut self) -> &mut T {
        unsafe { &mut *(self.iter.ctx as *mut T) }
    }

    /// Access ctx.
    /// ctx contains the context pointer assigned to a system
    ///
    /// # C++ API equivalent
    ///
    /// `iter::ctx`
    pub fn get_context_ptr_untyped(&self) -> *mut c_void {
        self.iter.ctx
    }

    /// Access param.
    /// param contains the pointer passed to the param argument of system::run
    ///
    /// # C++ API equivalent
    ///
    /// `iter::param`
    pub fn param_untyped(&self) -> *mut c_void {
        self.iter.param
    }

    /// Access param.
    /// param contains the pointer passed to the param argument of system::run
    ///
    /// # C++ API equivalent
    ///
    /// `iter::param`
    pub fn param<T: CachedComponentData>(&mut self) -> &mut T {
        unsafe { &mut *(self.iter.param as *mut T) }
    }

    /// # Arguments
    ///
    /// * `index` - Index of the field to check
    ///
    /// # Returns
    ///
    /// Returns whether field is matched on self
    ///
    /// # C++ API equivalent
    ///
    /// `iter::is_self`
    pub fn is_self(&self, index: i32) -> bool {
        unsafe { ecs_field_is_self(self.iter, index) }
    }

    /// # Arguments
    ///
    /// * `index` - Index of the field to check
    ///
    /// # Returns
    ///
    /// Returns whether field is set
    ///
    /// # C++ API equivalent
    ///
    /// `iter::is_set`
    pub fn is_set(&self, index: i32) -> bool {
        unsafe { ecs_field_is_set(self.iter, index) }
    }

    /// # Arguments
    ///
    /// * `index` - Index of the field to check
    ///
    /// # Returns
    ///
    /// Returns whether field is readonly
    ///
    /// # C++ API equivalent
    ///
    /// `iter::is_readonly`
    pub fn is_readonly(&self, index: i32) -> bool {
        unsafe { ecs_field_is_readonly(self.iter, index) }
    }

    /// Number of fields in iterator.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::field_count`
    pub fn get_field_count(&self) -> i32 {
        self.iter.field_count
    }

    /// Size of field data type.
    ///
    /// # Arguments
    ///
    /// * `index` - The field id.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::size`
    pub fn get_field_size(&self, index: i32) -> usize {
        unsafe { ecs_field_size(self.iter, index) }
    }

    /// Obtain field source (0 if This).
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::src`
    pub fn get_field_src(&self, index: i32) -> Entity {
        unsafe { Entity::new_from_existing(self.iter.world, ecs_field_src(self.iter, index)) }
    }

    /// Obtain id matched for field.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::id`
    pub fn get_field_id(&self, index: i32) -> Id {
        unsafe { Id::new_from_existing(self.iter.world, ecs_field_id(self.iter, index)) }
    }

    /// Obtain pair id matched for field.
    /// This operation will return None if the field is not a pair.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    pub fn get_field_pair_id(&self, index: i32) -> Option<Id> {
        unsafe {
            let id = ecs_field_id(self.iter, index);
            if ecs_id_is_pair(id) {
                Some(Id::new_from_existing(self.iter.world, id))
            } else {
                None
            }
        }
    }

    /// Obtain column index for field.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::column_index`
    pub fn get_field_column_index(&self, index: i32) -> i32 {
        unsafe { ecs_field_column_index(self.iter, index) }
    }

    /// Convert current iterator result to string
    ///
    /// # C++ API equivalent
    ///
    /// `iter::str`
    pub fn to_str(&self) -> CString {
        let c_str = unsafe { ecs_iter_str(self.iter) };
        ecs_assert!(!c_str.is_null(), FlecsErrorCode::InvalidParameter);
        unsafe { CString::from_raw(c_str) }
    }

    // TODO? in C++ API there is a mutable and immutable version of this function
    // Maybe we should create a ColumnView struct that is immutable and use the Column struct for mutable access?
    /// Get read/write acccess to field data.
    /// If the matched id for the specified field does not match with the provided
    /// type or if the field is readonly, the function will assert.
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of component to get the field data for
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::field`
    pub fn get_field_data<T: CachedComponentData>(&self, index: i32) -> Column<T> {
        self.get_field_internal::<T>(index)
    }

    /// Get unchecked access to field data.
    /// Unchecked access is required when a system does not know the type of a field at compile time.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # Returns
    ///
    /// Returns an UntypedColumn object that can be used to access the field data.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::field`
    pub fn get_untyped_field_data(&self, index: i32) -> UntypedColumn {
        self.get_field_untyped_internal(index)
    }

    /// Obtain the total number of tables the iterator will iterate over.
    ///
    /// # Returns
    ///
    /// The total number of tables that will be iterated over.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::table_count`
    pub fn get_table_count(&self) -> i32 {
        self.iter.table_count
    }

    /// Check if the current table has changed since the last iteration.
    ///
    /// # Returns
    ///
    /// Returns true if the current table has changed.
    ///
    /// # Safety
    ///
    /// Can only be used when iterating queries and/or systems.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::changed`
    pub fn is_changed(&self) -> bool {
        unsafe { ecs_query_changed(std::ptr::null_mut(), self.iter) }
    }

    /// Skip current table.
    /// This indicates to the query that the data in the current table is not
    /// modified. By default, iterating a table with a query will mark the
    /// iterated components as dirty if they are annotated with InOut or Out.
    ///
    /// When this operation is invoked, the components of the current table will
    /// not be marked dirty.
    ///
    /// # C++ API equivalent
    ///
    /// `iter::skip`
    pub fn skip(&mut self) {
        unsafe { ecs_query_skip(self.iter) };
    }

    /// # Returns
    ///
    /// Return group id for current table
    ///
    /// # Safety
    ///
    /// grouped queries only
    ///
    /// # C++ API equivalent
    ///
    /// `iter::group_id`
    pub fn get_group_id(&self) -> IdT {
        self.iter.group_id
    }

    fn get_field_internal<T: CachedComponentData>(&self, index: i32) -> Column<T> {
        ecs_assert!(
            {
                unsafe {
                    let term_id = ecs_field_id(self.iter, index);
                    let is_pair = ecs_id_is_pair(term_id);
                    let is_id_correct = T::get_id(self.iter.world) == term_id;
                    is_pair || is_id_correct
                }
            },
            FlecsErrorCode::ColumnTypeMismatch
        );

        let is_shared = !self.is_self(index);

        // If a shared column is retrieved with 'column', there will only be a
        // single value. Ensure that the application does not accidentally read
        // out of bounds.
        let count = if is_shared { 1 } else { self.count() };

        Column::<T>::new_from_array(
            unsafe { ecs_field_w_size(self.iter, T::get_size(self.iter.world), index) as *mut T },
            count,
            is_shared,
        )
    }

    fn get_field_untyped_internal(&self, index: i32) -> UntypedColumn {
        let size = unsafe { ecs_field_size(self.iter, index) };
        let is_shared = !self.is_self(index);

        // If a shared column is retrieved with 'column', there will only be a
        // single value. Ensure that the application does not accidentally read
        // out of bounds.
        let count = if is_shared {
            1
        } else {
            // If column is owned, there will be as many values as there are entities
            self.count()
        };

        UntypedColumn::new(
            unsafe { ecs_field_w_size(self.iter, 0, index) as *mut c_void },
            size,
            count,
            is_shared,
        )
    }
}
#[cfg(feature = "flecs_rules")]
impl<'a> Iter<'a> {}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let result = self.current;
            self.current += 1;
            Some(result)
        } else {
            // Reset current to begin for reiteration
            self.current = self.begin;
            None
        }
    }
}
