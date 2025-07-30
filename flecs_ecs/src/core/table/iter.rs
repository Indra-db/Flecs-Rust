//! Iterators used to iterate over tables and table rows in [`Query`], [`System`][crate::addons::system::System] and [`Observer`].
use core::marker::PhantomData;
use core::{ffi::CStr, ffi::c_void, ptr::NonNull};

use crate::core::table::field::{FieldIndex, FieldMut, FieldUntypedMut};
use crate::core::*;
use crate::sys;

pub struct TableIter<'a, const IS_RUN: bool = true, P = ()> {
    pub iter: &'a mut sys::ecs_iter_t,
    pub(crate) count: usize,
    marker: PhantomData<P>,
}

impl<'a, const IS_RUN: bool, P> TableIter<'a, IS_RUN, P>
where
    P: ComponentId,
{
    /// The world. Can point to stage when in deferred/readonly mode.
    pub fn world(&self) -> WorldRef<'a> {
        unsafe { WorldRef::from_ptr(self.iter.world) }
    }

    /// Actual world. Never points to a stage.
    pub fn real_world(&self) -> WorldRef<'a> {
        unsafe { WorldRef::from_ptr(self.iter.real_world) }
    }

    /// Constructs iterator from C iterator object.
    ///
    /// This operation is typically not invoked directly by the user.
    ///
    /// # Arguments
    ///
    /// * `iter` - C iterator
    ///
    /// # See also
    ///
    /// # Safety
    /// - caller must ensure that iter.param points to type T
    pub unsafe fn new(iter: &'a mut sys::ecs_iter_t) -> Self {
        let count = iter.count as usize;
        Self {
            iter,
            count,
            marker: PhantomData,
        }
    }

    /// Return the iterator type
    ///
    /// # Example
    ///
    /// ```
    /// #[derive(Component)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// world.entity().set(Position { x: 1.0, y: 2.0 });
    ///
    /// let query = world.new_query::<&Position>();
    ///
    /// query.run(|mut it| {
    ///     while it.next() {
    ///         //for each different table
    ///         for i in it.iter() {
    ///             //for each entity in the table
    ///             let pos = it.field::<Position>(0);
    ///             assert_eq!(pos[0].x, 1.0);
    ///             assert_eq!(pos[0].y, 2.0);
    ///         }
    ///     }
    /// });
    /// ```
    #[inline(always)]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = FieldIndex> {
        // Range has no bounds checks, .map(FieldIndex) is inlined out to a single `add+load`
        (0..self.count).map(FieldIndex)
    }

    /// Wrap the system id in the iterator in an [`EntityView`] object.
    pub fn system(&self) -> EntityView<'a> {
        EntityView::new_from(self.world(), self.iter.system)
    }

    /// Wrap the event id in the iterator in an [`EntityView`] object.
    pub fn event(&self) -> EntityView<'a> {
        EntityView::new_from(self.world(), self.iter.event)
    }

    /// Wrap the event id in the iterator in an [`IdView`] object.
    pub fn event_id(&self) -> IdView<'a> {
        IdView::new_from_id(self.world(), self.iter.event_id)
    }

    /// Obtain mutable handle to entity being iterated over.
    ///
    /// # Arguments
    ///
    /// * `row` - Row being iterated over
    pub fn entity(&self, row: impl Into<usize>) -> Option<EntityView<'a>> {
        let ptr = unsafe { self.iter.entities.add(row.into()) };
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { EntityView::new_from(self.real_world(), *ptr) })
    }

    /// Obtain entity id being iterated over. Can return Entity with id 0 if the entity at the specified row is not found.
    ///
    /// # Arguments
    ///
    /// * `row` - Row being iterated over
    #[inline(always)]
    pub fn entity_id(&self, row: impl Into<usize>) -> Entity {
        let ptr = unsafe { self.iter.entities.add(row.into()) };

        if ptr.is_null() {
            return Entity::null();
        }
        unsafe { Entity(*ptr) }
    }

    /// Return a mut reference to the raw iterator object.
    pub fn iter_mut(&mut self) -> &mut sys::ecs_iter_t {
        self.iter
    }

    /// Return the count of entities in the iterator.
    #[inline(always)]
    pub fn count(&self) -> usize {
        //TODO soft assert
        // ecs_check(iter_->flags & EcsIterIsValid, ECS_INVALID_PARAMETER,
        //     "operation invalid before calling next()");
        self.count
    }

    /// Return the delta time stored in the iterator.
    ///
    /// This is the time since the last frame.
    ///
    /// # See also
    ///
    /// * [`TableIter::delta_system_time()`]
    pub fn delta_time(&self) -> FTime {
        self.iter.delta_time
    }

    /// Return the delta system time stored in the iterator.
    ///
    /// This is the time since the last system invocation.
    ///
    /// # See also
    ///
    /// * [`TableIter::delta_time()`]
    pub fn delta_system_time(&self) -> FTime {
        self.iter.delta_system_time
    }

    /// Return the table stored in the iterator as an `Archetype` object
    pub fn archetype(&self) -> Option<Archetype<'a>> {
        self.table().map(|t| t.archetype())
    }

    pub fn table(&self) -> Option<Table<'a>> {
        NonNull::new(self.iter.table).map(|ptr| Table::new(self.real_world(), ptr))
    }

    pub fn other_table(&self) -> Option<Table<'a>> {
        NonNull::new(self.iter.other_table).map(|ptr| Table::new(self.real_world(), ptr))
    }

    pub fn range(&self) -> Option<TableRange<'a>> {
        self.table()
            .map(|t| TableRange::new(t, self.iter.offset, self.count as i32))
    }

    /// Get the variable of the iterator
    ///
    /// # Arguments
    ///
    /// * `var_id` - The variable id
    pub fn get_var(&self, var_id: i32) -> EntityView<'a> {
        ecs_assert!(var_id != -1, FlecsErrorCode::InvalidParameter, 0);
        let var =
            unsafe { sys::ecs_iter_get_var(self.iter as *const _ as *mut sys::ecs_iter_t, var_id) };
        let world = self.world();
        EntityView::new_from(world, var)
    }

    /// Get the variable of the iterator by name
    ///
    /// # Arguments
    ///
    /// * `var_id` - The variable id
    pub fn get_var_by_name(&self, name: &str) -> EntityView<'a> {
        let name = compact_str::format_compact!("{}\0", name);

        let world = self.world();
        let rule_query = self.iter.query;
        let var_id = unsafe { sys::ecs_query_find_var(rule_query, name.as_ptr() as *const _) };
        ecs_assert!(
            var_id != -1,
            FlecsErrorCode::InvalidParameter,
            name.as_str()
        );
        EntityView::new_from(world, unsafe {
            sys::ecs_iter_get_var(self.iter as *const _ as *mut _, var_id)
        })
    }

    /// Access ctx.
    /// ctx contains the context pointer assigned to a system
    ///
    /// # See also
    ///
    ///
    /// # Safety
    /// - caller must ensure the ctx variable was set to a type accessible as C and is not aliased
    pub unsafe fn context<T>(&mut self) -> &'a mut T {
        unsafe { &mut *(self.iter.ctx as *mut T) }
    }

    /// Access ctx.
    /// ctx contains the context pointer assigned to a system
    pub fn context_ptr(&self) -> *mut c_void {
        self.iter.ctx
    }

    /// Access param.
    /// param contains the pointer passed to the param argument of `system::run`
    ///
    /// # Safety
    ///
    /// - Caller must ensure the type is correct when accessing the pointer.
    pub unsafe fn param_untyped(&self) -> *mut c_void {
        self.iter.param
    }

    /// Access param.
    /// param contains the pointer passed to the param argument of `system::run` or the event payload
    ///
    /// There is no `param_mut` function to discourage the user from making changes to events sent to observers as
    /// order of execution is not defined.
    pub fn param(&self) -> &P::UnderlyingType {
        const {
            assert!(
                !P::IS_TAG,
                "called `.param()` on a ZST / tag data, which cannot be used when no payload is provided"
            );
        }

        let ptr = self.iter.param as *const P::UnderlyingType;

        assert!(
            !ptr.is_null(),
            "Tried to get param on an iterator where it was null."
        );

        unsafe { &*ptr }
    }

    /// # Arguments
    ///
    /// * `index` - Index of the field to check
    ///
    /// # Returns
    ///
    /// Returns whether field is matched on self
    #[inline(always)]
    pub fn is_self(&self, index: i8) -> bool {
        self.iter.sources.is_null() || unsafe { (*self.iter.sources.add(index as usize)) == 0 }
    }

    /// # Arguments
    ///
    /// * `index` - Index of the field to check
    ///
    /// # Returns
    ///
    /// Returns whether field is set
    pub fn is_set(&self, index: i8) -> bool {
        self.iter.set_fields & (1u32 << (index as usize)) != 0
    }

    /// # Arguments
    ///
    /// * `index` - Index of the field to check
    ///
    /// # Returns
    ///
    /// Returns whether field is readonly
    pub fn is_readonly(&self, index: i8) -> bool {
        unsafe { sys::ecs_field_is_readonly(self.iter, index) }
    }

    /// Number of fields in iterator.
    pub fn field_count(&self) -> i8 {
        self.iter.field_count
    }

    /// Size of field data type.
    ///
    /// # Arguments
    ///
    /// * `index` - The field id.
    pub fn size(&self, index: i8) -> usize {
        unsafe { sys::ecs_field_size(self.iter, index) }
    }

    /// Obtain field source (0 if This).
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    pub fn src(&self, index: usize) -> EntityView<'a> {
        unsafe { EntityView::new_from(self.world(), sys::ecs_field_src(self.iter, index as i8)) }
    }

    /// Obtain id matched for field.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    pub fn id(&self, index: i8) -> IdView<'a> {
        unsafe { IdView::new_from_id(self.world(), sys::ecs_field_id(self.iter, index)) }
    }

    /// Obtain pair id matched for field.
    /// This operation will return `None` if the field is not a pair.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    pub fn pair(&self, index: i8) -> Option<IdView<'a>> {
        unsafe {
            let id = sys::ecs_field_id(self.iter, index);
            if sys::ecs_id_is_pair(id) {
                Some(IdView::new_from_id(self.world(), id))
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
    pub fn column_index(&self, index: i8) -> i32 {
        unsafe { sys::ecs_field_column(self.iter, index) }
    }

    /// Obtain term that triggered an observer
    pub fn term_index(&self) -> i8 {
        self.iter.term_index
    }

    /// Convert current iterator result to string
    pub fn to_str(&self) -> &'a CStr {
        let c_str = unsafe { sys::ecs_iter_str(self.iter) };
        ecs_assert!(!c_str.is_null(), FlecsErrorCode::InvalidParameter);
        unsafe { CStr::from_ptr(c_str) }
    }

    #[inline(always)]
    #[allow(clippy::extra_unused_type_parameters)] //release build clippy warning
    fn field_safety_checks<T: ComponentId, const READONLY: bool, const TYPED: bool>(
        &self,
        _index: i8,
    ) {
        ecs_assert!(
            _index < self.iter.field_count,
            FlecsErrorCode::InvalidParameter,
            _index
        );
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, _index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field_handle from .each, use .field_at instead",
        );

        if !READONLY {
            ecs_assert!(
                !unsafe { sys::ecs_field_is_readonly(self.iter, _index) },
                FlecsErrorCode::AccessViolation,
                "field is readonly, check if your specified query terms are set &mut"
            );
        }
        if TYPED {
            ecs_assert!(
                {
                    let term_id = unsafe { sys::ecs_field_id(self.iter, _index) };
                    let id = <T::UnderlyingType as ComponentId>::entity_id(self.world());
                    id == term_id || unsafe { sys::ecs_id_is_pair(term_id) }
                },
                FlecsErrorCode::InvalidParameter,
                "id mismatch: expected {id}, got {term_id} or term_id is not a pair",
            );
        }
    }

    /// Get immutable access to field data.
    ///
    /// # Arguments
    ///
    /// * `field_handle` - The field handle.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    #[inline(always)]
    pub fn field<T: ComponentId>(&self, index: i8) -> Field<'a, T::UnderlyingType> {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        self.field_safety_checks::<T, true, true>(index);
        self.field_internal::<T::UnderlyingType>(index)
    }

    /// Get immutable access to field data.
    ///
    /// # Arguments
    ///
    /// * `field_handle` - The field handle.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    #[inline(always)]
    pub fn get_field<T: ComponentId>(&self, index: i8) -> Option<Field<'a, T::UnderlyingType>> {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        self.field_safety_checks::<T, true, true>(index);
        self.get_field_internal::<T::UnderlyingType>(index)
    }

    /// Get mutable access to field data.
    ///
    /// # Arguments
    ///
    /// * `field_handle` - The field handle.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    #[inline(always)]
    pub fn field_mut<T: ComponentId>(&'a self, index: i8) -> FieldMut<'a, T::UnderlyingType> {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        self.field_safety_checks::<T, false, true>(index);
        self.field_internal_mut::<T::UnderlyingType>(index)
    }

    /// Get mutable access to field data.
    ///
    /// # Arguments
    ///
    /// * `field_handle` - The field handle.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    #[inline(always)]
    pub fn get_field_mut<T: ComponentId>(
        &'a self,
        index: i8,
    ) -> Option<FieldMut<'a, T::UnderlyingType>> {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        self.field_safety_checks::<T, false, true>(index);
        self.get_field_internal_mut::<T::UnderlyingType>(index)
    }

    /// Get immutable access to untyped field data.
    ///
    /// # Arguments
    ///
    /// * `field_handle` - The field handle.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    #[inline(always)]
    pub fn field_untyped(&self, index: i8) -> FieldUntyped {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        self.field_safety_checks::<(), true, false>(index);
        self.field_untyped_internal(index)
    }

    /// Get immutable access to untyped field data.
    ///
    /// # Arguments
    ///
    /// * `field_handle` - The field handle.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    #[inline(always)]
    pub fn get_field_untyped(&self, index: i8) -> Option<FieldUntyped> {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        self.field_safety_checks::<(), true, false>(index);
        self.get_field_untyped_internal(index)
    }

    /// Get mutable access to untyped field data.
    ///
    /// # Arguments
    ///
    /// * `field_handle` - The field handle.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    #[inline(always)]
    pub fn field_untyped_mut(&self, index: i8) -> FieldUntypedMut {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        self.field_safety_checks::<(), false, false>(index);
        self.field_untyped_internal_mut(index)
    }

    /// Get mutable access to untyped field data.
    ///
    /// # Arguments
    ///
    /// * `field_handle` - The field handle.
    ///
    /// # Returns
    ///
    /// Returns a column object that can be used to access the field data.
    #[inline(always)]
    pub fn get_field_untyped_mut(&self, index: i8) -> Option<FieldUntypedMut> {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        self.field_safety_checks::<(), false, false>(index);
        self.get_field_untyped_internal_mut(index)
    }

    /// Get the typed field data for the specified row
    /// This function may be used to access shared fields when row is set to 0.
    ///
    /// # Arguments
    ///
    /// * `index` - The index.
    /// * `row` - The row index.
    ///
    /// # Returns
    ///
    /// An option containing a reference to the field data
    pub fn field_at<T: ComponentId>(
        &'a self,
        index: i8,
        row: impl Into<usize>,
    ) -> &'a T::UnderlyingType {
        self.field_safety_checks::<T, true, true>(index);
        let row = row.into();
        if self.iter.row_fields & (1u32 << index) != 0 {
            let field = self.field_at_internal::<T>(index, row);
            if field.is_null() {
                panic!("Tried to access field at index {index} for row {row}, but field is null.");
            }
            unsafe { &*field }
        } else {
            let field = self.field_internal::<T::UnderlyingType>(index);
            unsafe { &*field.slice_components.add(row) }
        }
    }

    /// Get the typed field data for the specified row
    /// This function may be used to access shared fields when row is set to 0.
    ///
    /// # Arguments
    ///
    /// * `index` - The index.
    /// * `row` - The row index.
    ///
    /// # Returns
    ///
    /// An option containing a reference to the field data
    pub fn get_field_at<T: ComponentId>(
        &'a self,
        index: i8,
        row: impl Into<usize>,
    ) -> Option<&'a T::UnderlyingType> {
        self.field_safety_checks::<T, true, true>(index);
        let row = row.into();
        if self.iter.row_fields & (1u32 << index) != 0 {
            let field = self.field_at_internal::<T>(index, row);
            if field.is_null() {
                None
            } else {
                Some(unsafe { &*field })
            }
        } else {
            let field = self.get_field_internal::<T::UnderlyingType>(index)?;
            Some(unsafe { &*field.slice_components.add(row) })
        }
    }

    /// Get the mutable typed field data for the specified row
    /// This function may be used to access shared fields when row is set to 0.
    #[allow(clippy::mut_from_ref)]
    pub fn field_at_mut<T: ComponentId>(
        &'a self,
        index: i8,
        row: impl Into<usize>,
    ) -> &'a mut T::UnderlyingType {
        self.field_safety_checks::<T, false, true>(index);
        let row = row.into();
        if self.iter.row_fields & (1u32 << index) != 0 {
            let field = self.field_at_internal_mut::<T>(index, row);
            if field.is_null() {
                panic!("Tried to access field at index {index} for row {row}, but field is null.");
            }
            unsafe { &mut *field }
        } else {
            let field = self.field_internal_mut::<T::UnderlyingType>(index);
            unsafe { &mut *field.slice_components.add(row) }
        }
    }

    /// Get the mutable typed field data for the specified row
    /// This function may be used to access shared fields when row is set to 0.
    #[allow(clippy::mut_from_ref)]
    pub fn get_field_at_mut<T: ComponentId>(
        &'a self,
        index: i8,
        row: impl Into<usize>,
    ) -> Option<&'a mut T::UnderlyingType> {
        self.field_safety_checks::<T, false, true>(index);
        let row = row.into();
        if self.iter.row_fields & (1u32 << index) != 0 {
            let field = self.field_at_internal_mut::<T>(index, row);
            if field.is_null() {
                None
            } else {
                Some(unsafe { &mut *field })
            }
        } else {
            let field = self.get_field_internal_mut::<T::UnderlyingType>(index)?;
            Some(unsafe { &mut *field.slice_components.add(row) })
        }
    }

    /// Get the untyped field data for the specified row
    /// This function may be used to access shared fields when row is set to 0.
    pub fn field_at_untyped(&self, index: i8, row: usize) -> *const c_void {
        self.field_safety_checks::<(), true, false>(index);
        if self.iter.row_fields & (1u32 << index) != 0 {
            self.field_at_untyped_internal(index, row)
        } else {
            let field = self.get_field_untyped_internal(index);
            if field.is_none() {
                return core::ptr::null();
            }
            let field = field.unwrap();
            unsafe { field.array.add(row * field.size) }
        }
    }

    /// Get the mutable untyped field data for the specified row
    /// This function may be used to access shared fields when row is set to 0.
    pub fn field_at_untyped_mut(&self, index: i8, row: usize) -> *mut c_void {
        self.field_safety_checks::<(), false, false>(index);
        if self.iter.row_fields & (1u32 << index) != 0 {
            self.field_at_untyped_internal_mut(index, row)
        } else {
            let field = self.get_field_untyped_internal_mut(index);
            if field.is_none() {
                return core::ptr::null_mut();
            }
            let field = field.unwrap();
            unsafe { field.array.add(row * field.size) }
        }
    }

    /// Get the component id of the field matched with the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Action;
    ///
    /// #[derive(Component)]
    /// struct DerivedAction;
    ///
    /// #[derive(Component)]
    /// struct DerivedAction2;
    ///
    /// let mut world = World::new();
    ///
    /// let comp = world.component::<Action>();
    ///
    /// world
    ///     .component::<DerivedAction>()
    ///     .add_trait::<(flecs::IsA, Action)>();
    ///
    /// world
    ///     .component::<DerivedAction>()
    ///     .add_trait::<(flecs::IsA, Action)>();
    ///
    /// world
    ///     .component::<DerivedAction2>()
    ///     .add_trait::<(flecs::IsA, Action)>();
    ///
    /// let entity = world
    ///     .entity()
    ///     .add(DerivedAction)
    ///     .add(DerivedAction2);
    ///
    /// world.new_query::<&Action>().run(|mut it| {
    ///     let mut vec = vec![];
    ///     while it.next() {
    ///         for i in it.iter() {
    ///             vec.push(it.component_id_at(0));
    ///         }
    ///     }
    ///     let id = world.component_id::<DerivedAction>();
    ///     let id2 = world.component_id::<DerivedAction2>();
    ///     assert_eq!(vec, vec![id, id2]);
    /// });
    /// ```
    pub fn component_id_at(&self, index: i8) -> Id {
        ecs_assert!(
            index < self.iter.field_count,
            FlecsErrorCode::InvalidParameter,
            index
        );

        let id = unsafe { self.iter.ids.add(index as usize).read() };
        crate::core::Id::new(id)
    }

    /// Get readonly access to entity ids.
    ///
    /// # Returns
    ///
    /// The entity ids.
    pub fn entities(&self) -> Field<Entity> {
        // let slice = unsafe {
        //     core::slice::from_raw_parts_mut(, self.count)
        // };
        Field::<Entity>::new(self.iter.entities as *mut Entity, false)
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
    /// * [`Query::is_changed()`]
    pub fn is_changed(&mut self) -> bool {
        unsafe { sys::ecs_iter_changed(self.iter) }
    }

    /// Returns whether the query has any data changed since the last iteration.
    ///
    /// This operation must be invoked before obtaining the iterator, as this will
    /// reset the changed state.
    ///
    /// # Returns
    ///
    /// The operation will return `true` after:
    /// - new entities have been matched with
    /// - matched entities were deleted
    /// - matched components were changed
    ///
    /// Otherwise, it will return `false`.
    ///
    /// # See also
    ///
    /// * [`TableIter::is_changed()`]
    /// * [`Query::is_changed()`]
    pub fn is_any_changed(&self) -> bool {
        unsafe { sys::ecs_query_changed(self.iter.query as *mut sys::ecs_query_t) }
    }

    /// Skip current table.
    /// This indicates to the query that the data in the current table is not
    /// modified. By default, iterating a table with a query will mark the
    /// iterated components as dirty if they are annotated with `InOut` or Out.
    ///
    /// When this operation is invoked, the components of the current table will
    /// not be marked dirty.
    pub fn skip(&mut self) {
        unsafe { sys::ecs_iter_skip(self.iter) };
    }

    /// # Returns
    ///
    /// Return group id for current table
    ///
    /// # Safety
    ///
    /// grouped queries only
    // TODO: this should be &self after I upgrade flecs
    pub fn group_id(&self) -> sys::ecs_id_t {
        unsafe { sys::ecs_iter_get_group(self.iter) }
    }

    #[inline(always)]
    pub(crate) fn get_field_internal<T>(&self, index: i8) -> Option<Field<'a, T>> {
        let (array, is_shared, count) = self.field_internal_parts::<T>(index);
        if count == 0 {
            return None;
        }
        // SAFETY: we already validated index/type before calling
        Some(Field::new(array, is_shared))
    }

    #[inline(always)]
    pub(crate) fn field_internal<T>(&self, index: i8) -> Field<'a, T> {
        let (array, is_shared, count) = self.field_internal_parts::<T>(index);
        if count == 0 {
            panic!(
                "field_internal: no values at index {} — ensure the field exists and has entries",
                index
            );
        }
        // SAFETY: we already validated index/type before calling
        Field::new(array, is_shared)
    }

    #[inline(always)]
    fn field_internal_parts<T>(&self, index: i8) -> (*mut T, bool, usize) {
        let is_shared = !self.is_self(index);
        // If a shared column is retrieved with 'column', there will only be a
        // single value. Ensure that the application does not accidentally read
        // out of bounds.
        let count = if is_shared {
            1
        } else {
            // If column is owned, there will be as many values as there are
            // entities.
            self.count()
        };

        let array = ecs_field::<T>(self.iter, index);

        (array, is_shared, count)
    }

    #[inline(always)]
    fn field_untyped_internal_parts(&self, index: i8) -> (*mut c_void, bool, usize, usize) {
        let size = unsafe { sys::ecs_field_size(self.iter, index) };
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

        let array = ecs_field_w_size(self.iter, size, index);

        (array, is_shared, count, size)
    }

    /// the “Option” version
    #[inline(always)]
    pub(crate) fn get_field_internal_mut<T>(&self, index: i8) -> Option<FieldMut<'a, T>> {
        let (array, is_shared, count) = self.field_internal_parts::<T>(index);
        if count == 0 {
            return None;
        }
        // SAFETY: we already validated index/type before calling
        Some(FieldMut::new(array, is_shared))
    }

    /// the “panic” version
    #[inline(always)]
    pub(crate) fn field_internal_mut<T>(&self, index: i8) -> FieldMut<'a, T> {
        let (array, is_shared, count) = self.field_internal_parts::<T>(index);
        if count == 0 {
            panic!(
                "field_internal: no values at index {} — ensure the field exists and has entries",
                index
            );
        }
        // SAFETY: same as above
        FieldMut::new(array, is_shared)
    }

    pub(crate) fn field_untyped_internal(&self, index: i8) -> FieldUntyped {
        let (array, is_shared, count, size) = self.field_untyped_internal_parts(index);

        if count == 0 {
            panic!(
                "field_untyped_internal: no values at index {} — ensure the field exists and has entries",
                index
            );
        }

        FieldUntyped::new(array, size, count, is_shared)
    }

    pub(crate) fn get_field_untyped_internal(&self, index: i8) -> Option<FieldUntyped> {
        let (array, is_shared, count, size) = self.field_untyped_internal_parts(index);

        if count == 0 {
            return None;
        }

        Some(FieldUntyped::new(array, size, count, is_shared))
    }

    pub(crate) fn field_untyped_internal_mut(&self, index: i8) -> FieldUntypedMut {
        let (array, is_shared, count, size) = self.field_untyped_internal_parts(index);

        if count == 0 {
            panic!(
                "field_untyped_internal_mut: no values at index {} — ensure the field exists and has entries",
                index
            );
        }

        FieldUntypedMut::new(array, size, count, is_shared)
    }

    pub(crate) fn get_field_untyped_internal_mut(&self, index: i8) -> Option<FieldUntypedMut> {
        let (array, is_shared, count, size) = self.field_untyped_internal_parts(index);

        if count == 0 {
            return None;
        }

        Some(FieldUntypedMut::new(array, size, count, is_shared))
    }

    pub(crate) fn field_at_untyped_internal(&self, index: i8, row: usize) -> *const c_void {
        let size = unsafe { sys::ecs_field_size(self.iter, index) };
        unsafe { sys::ecs_field_at_w_size(self.iter, size, index, row as i32) }
    }

    pub(crate) fn field_at_untyped_internal_mut(&self, index: i8, row: usize) -> *mut c_void {
        let size = unsafe { sys::ecs_field_size(self.iter, index) };
        unsafe { sys::ecs_field_at_w_size(self.iter, size, index, row as i32) }
    }

    pub(crate) fn field_at_internal<T>(&'a self, index: i8, row: usize) -> *const T::UnderlyingType
    where
        T: ComponentId,
    {
        unsafe {
            sys::ecs_field_at_w_size(
                self.iter,
                const { core::mem::size_of::<T::UnderlyingType>() },
                index,
                row as i32,
            ) as *mut T::UnderlyingType
        }
    }

    pub(crate) fn field_at_internal_mut<T>(
        &'a self,
        index: i8,
        row: usize,
    ) -> *mut T::UnderlyingType
    where
        T: ComponentId,
    {
        unsafe {
            sys::ecs_field_at_w_size(
                self.iter,
                const { core::mem::size_of::<T::UnderlyingType>() },
                index,
                row as i32,
            ) as *mut T::UnderlyingType
        }
    }

    /// Forward to each.
    /// If a system has an each callback registered, this operation will forward
    /// the current iterator to the each callback.
    pub fn each(&mut self) {
        if let Some(each) = self.iter.callback {
            unsafe {
                each(self.iter);
            }
        }
    }

    /// Iterate targets for pair field.
    ///
    /// # Arguments
    ///
    /// * index: the field index
    /// * func: callback invoked for each target with the signature fn(EntityView entity)
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let likes = world.entity();
    /// let pizza = world.entity();
    /// let salad = world.entity();
    /// let alice = world.entity().add((likes, pizza)).add((likes, salad));
    ///
    /// let q = world
    ///     .query::<()>()
    ///     .with((likes, id::<flecs::Any>()))
    ///     .build();
    ///
    /// let mut count = 0;
    /// let mut tgt_count = 0;
    ///
    /// q.run(|mut it| {
    ///
    ///     while it.next() {
    ///        for row in it.iter() {
    ///             let e = it.entity(row).unwrap();
    ///             assert_eq!(e, alice);
    ///     
    ///             it.targets(0, |tgt| {
    ///                 if tgt_count == 0 {
    ///                     assert_eq!(tgt, pizza);
    ///                 }
    ///                 if tgt_count == 1 {
    ///                     assert_eq!(tgt, salad);
    ///                 }
    ///                 tgt_count += 1;
    ///             });
    ///     
    ///             count += 1;
    ///        }
    ///     }
    /// });
    ///
    /// assert_eq!(count, 1);
    /// assert_eq!(tgt_count, 2);
    /// ```
    pub fn targets(&self, index: i8, mut func: impl FnMut(EntityView)) {
        ecs_assert!(!self.iter.table.is_null(), FlecsErrorCode::InvalidOperation);
        ecs_assert!(
            index < self.iter.field_count,
            FlecsErrorCode::InvalidOperation
        );
        ecs_assert!(
            unsafe { sys::ecs_field_is_set(self.iter, index) },
            FlecsErrorCode::InvalidOperation
        );
        let table_type = unsafe { sys::ecs_table_get_type(self.iter.table) };
        let table_record = unsafe { *self.iter.trs.add(index as usize) };
        let mut i = unsafe { (*table_record).index };
        let end = i + unsafe { (*table_record).count };
        while i < end {
            let id = unsafe { *(*table_type).array.add(i as usize) };
            ecs_assert!(
                ecs_is_pair(id),
                FlecsErrorCode::InvalidParameter,
                "field does not match a pair"
            );
            let target = EntityView::new_from(
                self.world(),
                ecs_second(id, unsafe { WorldRef::from_ptr(self.iter.world) }),
            );
            func(target);
            i += 1;
        }
    }
}

impl<P> TableIter<'_, true, P>
where
    P: ComponentId,
{
    /// Progress iterator.
    /// this operation is not available on functions where the iterator is not from `.run()`
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn next(&mut self) -> bool {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        if self.iter.flags & sys::EcsIterIsValid != 0 && !self.iter.table.is_null() {
            unsafe {
                sys::ecs_table_unlock(self.iter.world, self.iter.table);
            };
        }

        let result = match self.iter.next {
            Some(next) => {
                let r = unsafe { next(self.iter) };
                self.count = self.iter.count as usize;
                r
            }
            None => {
                // no more items → clear valid and return
                self.iter.flags &= !sys::EcsIterIsValid;
                return false;
            }
        };

        self.iter.flags |= sys::EcsIterIsValid;
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        {
            if result && !self.iter.table.is_null() {
                unsafe {
                    sys::ecs_table_lock(self.iter.world, self.iter.table);
                };
            }
        }

        result
    }

    /// Free iterator resources.
    /// This operation only needs to be called when the iterator is not iterated
    /// until completion (e.g. the last call to `next()` did not return false).
    ///
    /// Failing to call this operation on an unfinished iterator will throw a
    /// `fatal LEAK_DETECTED` error.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.new_query::<&mut Position>().run(|it| {
    ///     // this will ensure that the iterator is freed and no assertion will happen
    ///     it.fini();
    /// });
    /// ```
    pub fn fini(self) {
        #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
        if self.iter.flags & sys::EcsIterIsValid != 0 && !self.iter.table.is_null() {
            unsafe {
                sys::ecs_table_unlock(self.iter.world, self.iter.table);
            };
        }

        unsafe {
            sys::ecs_iter_fini(self.iter);
        }
    }
}

#[inline(always)]
pub(crate) fn table_lock(_world_ptr: *mut sys::ecs_world_t, _table_ptr: *mut sys::ecs_table_t) {
    #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    unsafe {
        sys::ecs_table_lock(_world_ptr, _table_ptr);
    }
}

#[inline(always)]
pub(crate) fn table_unlock(_world_ptr: *mut sys::ecs_world_t, _table_ptr: *mut sys::ecs_table_t) {
    #[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
    unsafe {
        sys::ecs_table_unlock(_world_ptr, _table_ptr);
    }
}
