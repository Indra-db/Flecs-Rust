use std::{ffi::CStr, os::raw::c_void};

use crate::{
    core::FlecsErrorCode,
    ecs_assert,
    sys::{
        ecs_field_column_index, ecs_field_id, ecs_field_is_readonly, ecs_field_is_self,
        ecs_field_is_set, ecs_field_size, ecs_field_src, ecs_field_w_size, ecs_id_is_pair,
        ecs_iter_str, ecs_query_changed, ecs_query_skip, ecs_table_get_type,
    },
};

use super::{
    c_types::{IdT, IterT},
    column::{Column, UntypedColumn},
    component_registration::CachedComponentData,
    entity::Entity,
    id::Id,
    table::{Table, TableRange},
    world::World,
    Archetype, FTime, IsEnum,
};

pub struct Iter<'a> {
    iter: &'a mut IterT,
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
    ///
    /// # See also
    ///
    /// * C++ API: `iter::iter`
    #[doc(alias = "iter::iter")]
    pub unsafe fn new(iter: &'a mut IterT) -> Self {
        Self { iter }
    }

    pub fn iter(&self) -> IterIterator {
        IterIterator {
            iter: self,
            index: 0,
        }
    }

    /// Wrap the system id in the iterator in an `Entity` object
    ///
    /// # See also
    ///
    /// * C++ API: `iter::system`
    #[doc(alias = "iter::system")]
    pub fn system(&self) -> Entity {
        Entity::new_from_existing_raw(self.iter.world, self.iter.system)
    }

    /// Wrap the event id in the iterator in an `Entity` object
    ///
    /// # See also
    ///
    /// * C++ API: `iter::event`
    #[doc(alias = "iter::event")]
    pub fn get_event(&self) -> Entity {
        Entity::new_from_existing_raw(self.iter.world, self.iter.event)
    }

    /// Wrap the event id in the iterator in an `Id` object
    ///
    /// # See also
    ///
    /// * C++ API: `iter::event_id`
    #[doc(alias = "iter::event_id")]
    pub fn get_event_id(&self) -> Id {
        Id::new_from_existing(self.iter.world, self.iter.event_id)
    }

    /// wrap the world in the iterator in a `World` object
    ///
    /// # See also
    ///
    /// * C++ API: `iter::world`
    #[doc(alias = "iter::world")]
    pub fn get_world(&self) -> World {
        World::new_wrap_raw_world(self.iter.world)
    }

    /// Obtain mutable handle to entity being iterated over.
    ///
    /// # Arguments
    ///
    /// * `row` - Row being iterated over
    ///
    /// # See also
    ///
    /// * C++ API: `iter::entity`
    #[doc(alias = "iter::entity")]
    pub fn get_entity(&self, row: usize) -> Entity {
        unsafe { Entity::new_from_existing_raw(self.iter.world, *self.iter.entities.add(row)) }
    }

    /// Return a mut reference to the raw iterator object
    ///
    /// # See also
    ///
    /// * C++ API: `iter::c_ptr`
    #[doc(alias = "iter::c_ptr")]
    pub fn get_raw_ref(&mut self) -> &mut IterT {
        self.iter
    }

    /// Return the count of entities in the iterator
    ///
    /// # See also
    ///
    /// * C++ API: `iter::count`
    #[doc(alias = "iter::count")]
    pub fn count(&self) -> usize {
        self.iter.count as usize
    }

    /// Return the delta time stored in the iterator
    ///
    /// # See also
    ///
    /// * C++ API: `iter::delta_time`
    #[doc(alias = "iter::delta_time")]
    pub fn get_delta_time(&self) -> FTime {
        self.iter.delta_time
    }

    /// Return the delta system time stored in the iterator
    ///
    /// # See also
    ///
    /// * C++ API: `iter::delta_system_time`
    #[doc(alias = "iter::delta_system_time")]
    pub fn get_delta_system_time(&self) -> FTime {
        self.iter.delta_system_time
    }

    /// Return the table stored in the iterator as an `Archetype` object
    ///
    /// # See also
    ///
    /// * C++ API: `iter::type`
    #[doc(alias = "iter::type")]
    pub fn get_archetype(&self) -> Archetype {
        unsafe { Archetype::new(self.iter.world, ecs_table_get_type(self.iter.table)) }
    }

    /// # See also
    ///
    /// * C++ API: `iter::table`
    #[doc(alias = "iter::table")]
    pub fn get_table(&self) -> Table {
        Table::new(
            &World::new_wrap_raw_world(self.iter.real_world),
            self.iter.table,
        )
    }

    /// # See also
    ///
    /// * C++ API: `iter::range`
    #[doc(alias = "iter::range")]
    pub fn get_table_range(&mut self) -> TableRange {
        let iter: &mut IterT = self.iter;
        TableRange::new_raw(iter.real_world, iter.table, iter.offset, iter.count)
    }

    /// Access ctx.
    /// ctx contains the context pointer assigned to a system
    ///
    /// # See also
    ///
    /// * C++ API: `iter::ctx`
    #[doc(alias = "iter::ctx")]
    pub fn get_context_ptr<T: CachedComponentData>(&mut self) -> &mut T {
        unsafe { &mut *(self.iter.ctx as *mut T) }
    }

    /// Access ctx.
    /// ctx contains the context pointer assigned to a system
    ///
    /// # See also
    ///
    /// * C++ API: `iter::ctx`
    #[doc(alias = "iter::ctx")]
    pub fn get_context_ptr_untyped(&self) -> *mut c_void {
        self.iter.ctx
    }

    /// Access param.
    /// param contains the pointer passed to the param argument of `system::run`
    ///
    /// # See also
    ///
    /// * C++ API: `iter::param`
    #[doc(alias = "iter::param")]
    pub fn param_untyped(&self) -> *mut c_void {
        self.iter.param
    }

    /// Access param.
    /// param contains the pointer passed to the param argument of `system::run`
    ///
    /// # See also
    ///
    /// * C++ API: `iter::param`
    #[doc(alias = "iter::param")]
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
    /// # See also
    ///
    /// * C++ API: `iter::is_self`
    #[doc(alias = "iter::is_self")]
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
    /// # See also
    ///
    /// * C++ API: `iter::is_set`
    #[doc(alias = "iter::is_set")]
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
    /// # See also
    ///
    /// * C++ API: `iter::is_readonly`
    #[doc(alias = "iter::is_readonly")]
    pub fn is_readonly(&self, index: i32) -> bool {
        unsafe { ecs_field_is_readonly(self.iter, index) }
    }

    /// Number of fields in iterator.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::field_count`
    #[doc(alias = "iter::field_count")]
    pub fn get_field_count(&self) -> i32 {
        self.iter.field_count
    }

    /// Size of field data type.
    ///
    /// # Arguments
    ///
    /// * `index` - The field id.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::size`
    #[doc(alias = "iter::size")]
    pub fn get_field_size(&self, index: i32) -> usize {
        unsafe { ecs_field_size(self.iter, index) }
    }

    /// Obtain field source (0 if This).
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::src`
    #[doc(alias = "iter::src")]
    pub fn get_field_src(&self, index: i32) -> Entity {
        unsafe { Entity::new_from_existing_raw(self.iter.world, ecs_field_src(self.iter, index)) }
    }

    /// Obtain id matched for field.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::id`
    #[doc(alias = "iter::id")]
    pub fn get_field_id(&self, index: i32) -> Id {
        unsafe { Id::new_from_existing(self.iter.world, ecs_field_id(self.iter, index)) }
    }

    /// Obtain pair id matched for field.
    /// This operation will return None if the field is not a pair.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::pair`
    #[doc(alias = "iter::pair")]
    pub fn get_pair_id(&self, index: i32) -> Option<Id> {
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
    /// # See also
    ///
    /// * C++ API: `iter::column_index`
    #[doc(alias = "iter::column_index")]
    pub fn get_field_column_index(&self, index: i32) -> i32 {
        unsafe { ecs_field_column_index(self.iter, index) }
    }

    /// Convert current iterator result to string
    ///
    /// # See also
    ///
    /// * C++ API: `iter::str`
    #[doc(alias = "iter::str")]
    pub fn to_str(&self) -> &CStr {
        let c_str = unsafe { ecs_iter_str(self.iter) };
        ecs_assert!(!c_str.is_null(), FlecsErrorCode::InvalidParameter);
        unsafe { CStr::from_ptr(c_str) }
    }

    /// Get read/write access to field data.
    /// If the matched id for the specified field does not match with the provided
    /// type or if the field is readonly, the function will assert.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it casts `c_void` data to a type T expecting
    /// The user to pass the correct index + the index being within bounds
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
    /// # See also
    ///
    /// * C++ API: `iter::field`
    #[doc(alias = "iter::field")]
    // TODO? in C++ API there is a mutable and immutable version of this function
    // Maybe we should create a ColumnView struct that is immutable and use the Column struct for mutable access?
    pub unsafe fn get_field_data<T: CachedComponentData + IsEnum>(&self, index: i32) -> Column<T> {
        self.get_field_internal::<T>(index)
    }

    // pub unsafe fn get_field_data_as_(&self, index: i32) -> Column<IdT> {
    //     self.get_field_internal::<T>(index)
    // }

    /// Get unchecked access to field data.
    /// Unchecked access is required when a system does not know the type of a field at compile time.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # Returns
    ///
    /// Returns an `UntypedColumn` object that can be used to access the field data.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::field`
    #[doc(alias = "iter::field")]
    pub fn get_untyped_field_data(&self, index: i32) -> UntypedColumn {
        self.get_field_untyped_internal(index)
    }

    /// Get readonly access to entity ids.
    ///
    /// # Returns
    ///
    /// The entity ids.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::entities`
    #[doc(alias = "iter::entities")]
    pub fn get_entities(&self) -> &[Entity] {
        unsafe {
            std::slice::from_raw_parts(
                self.iter.entities as *const Entity,
                self.iter.count as usize,
            )
        }
        //TODO this should return our Column struct. check cpp.
    }

    /// Obtain the total number of tables the iterator will iterate over.
    ///
    /// # Returns
    ///
    /// The total number of tables that will be iterated over.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::table_count`
    #[doc(alias = "iter::table_count")]
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
    /// # See also
    ///
    /// * C++ API: `iter::changed`
    #[doc(alias = "iter::changed")]
    pub fn is_changed(&self) -> bool {
        unsafe { ecs_query_changed(std::ptr::null_mut(), self.iter) }
    }

    /// Skip current table.
    /// This indicates to the query that the data in the current table is not
    /// modified. By default, iterating a table with a query will mark the
    /// iterated components as dirty if they are annotated with `InOut` or Out.
    ///
    /// When this operation is invoked, the components of the current table will
    /// not be marked dirty.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::skip`
    #[doc(alias = "iter::skip")]
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
    /// # See also
    ///
    /// * C++ API: `iter::group_id`
    #[doc(alias = "iter::group_id")]
    pub fn get_group_id(&self) -> IdT {
        self.iter.group_id
    }

    fn get_field_internal<T: CachedComponentData + IsEnum>(&self, index: i32) -> Column<T> {
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

        if !T::IS_ENUM {
            Column::<T>::new_from_array(
                unsafe {
                    ecs_field_w_size(self.iter, T::get_size(self.iter.world), index) as *mut T
                },
                count,
                is_shared,
            )
        } else {
            Column::<T>::new_from_array(
                unsafe {
                    World::new_wrap_raw_world(self.iter.world)
                        .get_alive(
                            *(ecs_field_w_size(self.iter, T::get_size(self.iter.world), index)
                                as *mut IdT),
                        )
                        .get_enum_mut_internal::<T>()
                },
                count,
                is_shared,
            )
        }
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

pub struct IterIterator<'a> {
    iter: &'a Iter<'a>,
    index: usize,
}

impl<'a> Iterator for IterIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.iter.count() {
            let result = self.index;
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}
