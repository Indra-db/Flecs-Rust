//! Iterators used to iterate over tables and table rows in [`Query`], [`System`][crate::addons::system::System] and [`Observer`].
use core::marker::PhantomData;
use core::{ffi::CStr, ffi::c_void, ptr::NonNull};

use crate::core::*;
use crate::sys;

use super::field::{FieldAt, FieldAtMut, FieldMut};

/// Field fetching errors when using [`TableIter::get()`]
pub enum FieldError {
    InvalidIndex,
    WrongType,
    Locked,
}

pub struct TableIter<'a, const IS_RUN: bool = true, P = ()> {
    pub(crate) iter: &'a mut sys::ecs_iter_t,
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
    /// * C++ API: `iter::iter`
    /// # Safety
    /// - caller must ensure that iter.param points to type T
    pub unsafe fn new(iter: &'a mut sys::ecs_iter_t) -> Self {
        Self {
            iter,
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
    ///             let pos = it.field::<Position>(0).unwrap();
    ///             assert_eq!(pos[0].x, 1.0);
    ///             assert_eq!(pos[0].y, 2.0);
    ///         }
    ///     }
    /// });
    /// ```
    pub fn iter(&self) -> TableRowIter<IS_RUN, P> {
        TableRowIter {
            iter: self,
            index: 0,
        }
    }

    /// Wrap the system id in the iterator in an [`EntityView`] object.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::system`
    #[doc(alias = "iter::system")]
    pub fn system(&self) -> EntityView<'a> {
        EntityView::new_from(self.world(), self.iter.system)
    }

    /// Wrap the event id in the iterator in an [`EntityView`] object.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::event`
    #[doc(alias = "iter::event")]
    pub fn event(&self) -> EntityView<'a> {
        EntityView::new_from(self.world(), self.iter.event)
    }

    /// Wrap the event id in the iterator in an [`IdView`] object.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::event_id`
    #[doc(alias = "iter::event_id")]
    pub fn event_id(&self) -> IdView<'a> {
        IdView::new_from_id(self.world(), self.iter.event_id)
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
    pub fn entity(&self, row: usize) -> EntityView<'a> {
        unsafe { EntityView::new_from(self.real_world(), *self.iter.entities.add(row)) }
    }

    /// Return a mut reference to the raw iterator object.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::c_ptr`
    #[doc(alias = "iter::c_ptr")]
    pub fn iter_mut(&mut self) -> &mut sys::ecs_iter_t {
        self.iter
    }

    /// Return the count of entities in the iterator.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::count`
    #[doc(alias = "iter::count")]
    pub fn count(&self) -> usize {
        //TODO soft assert
        // ecs_check(iter_->flags & EcsIterIsValid, ECS_INVALID_PARAMETER,
        //     "operation invalid before calling next()");
        self.iter.count as usize
    }

    /// Return the delta time stored in the iterator.
    ///
    /// This is the time since the last frame.
    ///
    /// # See also
    ///
    /// * [`TableIter::delta_system_time()`]
    /// * C++ API: `iter::delta_time`
    #[doc(alias = "iter::delta_time")]
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
    /// * C++ API: `iter::delta_system_time`
    #[doc(alias = "iter::delta_system_time")]
    pub fn delta_system_time(&self) -> FTime {
        self.iter.delta_system_time
    }

    /// Return the table stored in the iterator as an `Archetype` object
    ///
    /// # See also
    ///
    /// * C++ API: `iter::type`
    #[doc(alias = "iter::type")]
    pub fn archetype(&self) -> Option<Archetype<'a>> {
        self.table().map(|t| t.archetype())
    }

    /// # See also
    ///
    /// * C++ API: `iter::table`
    #[doc(alias = "iter::table")]
    pub fn table(&self) -> Option<Table<'a>> {
        NonNull::new(self.iter.table).map(|ptr| Table::new(self.real_world(), ptr))
    }

    /// # See also
    ///
    /// * C++ API: `iter::other_table`
    #[doc(alias = "iter::table")]
    pub fn other_table(&self) -> Option<Table<'a>> {
        NonNull::new(self.iter.other_table).map(|ptr| Table::new(self.real_world(), ptr))
    }

    /// # See also
    ///
    /// * C++ API: `iter::range`
    #[doc(alias = "iter::range")]
    pub fn range(&self) -> Option<TableRange<'a>> {
        self.table()
            .map(|t| TableRange::new(t, self.iter.offset, self.iter.count))
    }

    /// Get the variable of the iterator
    ///
    /// # Arguments
    ///
    /// * `var_id` - The variable id
    ///
    /// # See also
    ///
    /// * C++ API: `iter::get_var`
    #[doc(alias = "iter::get_var")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `iter::get_var`
    #[doc(alias = "iter::get_var")]
    pub fn get_var_by_name(&self, name: &str) -> EntityView<'a> {
        let name = compact_str::format_compact!("{}\0", name);

        let world = self.world();
        let rule_query = unsafe { self.iter.priv_.iter.query.query };
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
    /// * C++ API: `iter::ctx`
    ///
    /// # Safety
    /// - caller must ensure the ctx variable was set to a type accessible as C and is not aliased
    #[doc(alias = "iter::ctx")]
    pub unsafe fn context<T>(&mut self) -> &'a mut T {
        unsafe { &mut *(self.iter.ctx as *mut T) }
    }

    /// Access ctx.
    /// ctx contains the context pointer assigned to a system
    ///
    /// # See also
    ///
    /// * C++ API: `iter::ctx`
    #[doc(alias = "iter::ctx")]
    pub fn context_ptr(&self) -> *mut c_void {
        self.iter.ctx
    }

    /// Access param.
    /// param contains the pointer passed to the param argument of `system::run`
    ///
    /// # Safety
    ///
    /// - Caller must ensure the type is correct when accessing the pointer.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::param`
    #[doc(alias = "iter::param")]
    pub unsafe fn param_untyped(&self) -> *mut c_void {
        self.iter.param
    }

    /// Access param.
    /// param contains the pointer passed to the param argument of `system::run` or the event payload
    ///
    /// There is no `param_mut` function to discourage the user from making changes to events sent to observers as
    /// order of execution is not defined.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::param`
    #[doc(alias = "iter::param")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `iter::is_self`
    #[doc(alias = "iter::is_self")]
    pub fn is_self(&self, index: i8) -> bool {
        unsafe { sys::ecs_field_is_self(self.iter, index) }
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
    pub fn is_set(&self, index: i8) -> bool {
        unsafe { sys::ecs_field_is_set(self.iter, index) }
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
    pub fn is_readonly(&self, index: i8) -> bool {
        unsafe { sys::ecs_field_is_readonly(self.iter, index) }
    }

    /// Number of fields in iterator.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::field_count`
    #[doc(alias = "iter::field_count")]
    pub fn field_count(&self) -> i8 {
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
    pub fn size(&self, index: i8) -> usize {
        unsafe { sys::ecs_field_size(self.iter, index) }
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
    pub fn src(&self, index: usize) -> EntityView<'a> {
        unsafe { EntityView::new_from(self.world(), sys::ecs_field_src(self.iter, index as i8)) }
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
    pub fn id(&self, index: i8) -> IdView<'a> {
        unsafe { IdView::new_from_id(self.world(), sys::ecs_field_id(self.iter, index)) }
    }

    /// Obtain pair id matched for field.
    /// This operation will return `None` if the field is not a pair.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::pair`
    #[doc(alias = "iter::pair")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `iter::column_index`
    #[doc(alias = "iter::column_index")]
    pub fn column_index(&self, index: i8) -> i32 {
        unsafe { sys::ecs_field_column(self.iter, index) }
    }

    /// Obtain term that triggered an observer
    ///
    /// # See also
    ///
    /// * C++ API: `iter::term_index`
    pub fn term_index(&self) -> i8 {
        self.iter.term_index
    }

    /// Convert current iterator result to string
    ///
    /// # See also
    ///
    /// * C++ API: `iter::str`
    #[doc(alias = "iter::str")]
    pub fn to_str(&self) -> &'a CStr {
        let c_str = unsafe { sys::ecs_iter_str(self.iter) };
        ecs_assert!(!c_str.is_null(), FlecsErrorCode::InvalidParameter);
        unsafe { CStr::from_ptr(c_str) }
    }

    /// Get read access to field data.
    ///
    /// if the field is readonly, the function will assert.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it casts `c_void` data to a type T expecting
    /// The user to pass the correct index and type + the index being within bounds + optional data
    /// to be valid.
    ///
    /// This function should not be used in `each_iter()` callbacks, unless it is to
    /// access a shared field. For access to non-shared fields in `each_iter()`, use
    /// `field_at`.
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
    pub unsafe fn field_unchecked<T>(&self, index: i8) -> Field<T, true> {
        ecs_assert!(
            index < self.iter.field_count,
            FlecsErrorCode::InvalidParameter,
            index
        );
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );
        let id = unsafe { *self.iter.ids.add(index as usize) };
        unsafe { self.field_internal::<T, true>(index, Entity(id)).unwrap() }
    }

    /// Get write access to field data.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it casts `c_void` data to a type T expecting
    /// The user to pass the correct index and type + the index being within bounds + optional data
    /// to be valid.
    ///
    /// This function should not be used in `each_iter()` callbacks, unless it is to
    /// access a shared field. For access to non-shared fields in `each_iter()`, use
    /// `field_at`.
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
    pub unsafe fn field_unchecked_mut<T>(&self, index: i8) -> FieldMut<T, true> {
        ecs_assert!(
            index < self.iter.field_count,
            FlecsErrorCode::InvalidParameter,
            index
        );
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );
        ecs_assert!(
            !unsafe { sys::ecs_field_is_readonly(self.iter, index) },
            FlecsErrorCode::AccessViolation,
            "field is readonly, check if your specified query terms are set &mut"
        );
        let id = unsafe { *self.iter.ids.add(index as usize) };
        unsafe {
            self.field_internal_mut::<T, true>(index, Entity(id))
                .unwrap()
        }
    }

    fn field_checked_mut_lockless<T: ComponentId>(
        &self,
        index: i8,
    ) -> Option<FieldMut<T::UnderlyingType, false>> {
        let id = <T::UnderlyingType as ComponentId>::id(self.world());

        if index > self.iter.field_count {
            return None;
        }

        let term_id = unsafe { sys::ecs_field_id(self.iter, index) };
        let is_pair = unsafe { sys::ecs_id_is_pair(term_id) };
        let is_id_correct = id == term_id;

        if is_id_correct || is_pair {
            return unsafe {
                self.field_internal_mut::<T::UnderlyingType, false>(index, Entity(term_id))
            };
        }

        None
    }

    fn field_checked_mut<T: ComponentId>(
        &self,
        index: i8,
    ) -> Option<FieldMut<T::UnderlyingType, true>> {
        let id = <T::UnderlyingType as ComponentId>::id(self.world());

        if index > self.iter.field_count {
            return None;
        }

        let term_id = unsafe { sys::ecs_field_id(self.iter, index) };
        let is_pair = unsafe { sys::ecs_id_is_pair(term_id) };
        let is_id_correct = id == term_id;

        if is_id_correct || is_pair {
            return unsafe {
                self.field_internal_mut::<T::UnderlyingType, true>(index, Entity(term_id))
            };
        }

        None
    }

    fn field_checked_mut_result<T: ComponentId>(
        &self,
        index: i8,
    ) -> Result<FieldMut<T::UnderlyingType, true>, FieldError> {
        let id = <T::UnderlyingType as ComponentId>::id(self.world());

        if index > self.iter.field_count {
            return Err(FieldError::InvalidIndex);
        }

        let term_id = unsafe { sys::ecs_field_id(self.iter, index) };
        let is_pair = unsafe { sys::ecs_id_is_pair(term_id) };
        let is_id_correct = id == term_id;

        if is_id_correct || is_pair {
            return unsafe {
                self.field_internal_mut_result::<T::UnderlyingType, true>(index, Entity(term_id))
            };
        }

        Err(FieldError::WrongType)
    }

    fn field_at_checked_mut<T: ComponentId>(
        &self,
        index: i8,
        row: usize,
    ) -> Option<FieldAtMut<T::UnderlyingType>> {
        let id = <T::UnderlyingType as ComponentId>::id(self.world());

        if index > self.iter.field_count {
            return None;
        }

        let term_id = unsafe { sys::ecs_field_id(self.iter, index) };
        let is_pair = unsafe { sys::ecs_id_is_pair(term_id) };
        let is_id_correct = id == term_id;

        if is_id_correct || is_pair {
            return unsafe {
                self.field_at_internal_mut::<T::UnderlyingType>(index, row, Entity(term_id))
            };
        }

        None
    }

    fn field_checked<T: ComponentId, const LOCK: bool>(
        &self,
        index: i8,
    ) -> Option<Field<T::UnderlyingType, LOCK>> {
        let id = <T::UnderlyingType as ComponentId>::id(self.world());

        if index > self.iter.field_count {
            return None;
        }

        let term_id = unsafe { sys::ecs_field_id(self.iter, index) };
        let is_pair = unsafe { sys::ecs_id_is_pair(term_id) };
        let is_id_correct = id == term_id;

        if is_id_correct || is_pair {
            return unsafe {
                self.field_internal::<T::UnderlyingType, LOCK>(index, Entity(term_id))
            };
        }

        None
    }

    fn field_checked_result<T: ComponentId, const LOCK: bool>(
        &self,
        index: i8,
    ) -> Result<Field<T::UnderlyingType, LOCK>, FieldError> {
        let id = <T::UnderlyingType as ComponentId>::id(self.world());

        if index > self.iter.field_count {
            return Err(FieldError::InvalidIndex);
        }

        let term_id = unsafe { sys::ecs_field_id(self.iter, index) };
        let is_pair = unsafe { sys::ecs_id_is_pair(term_id) };
        let is_id_correct = id == term_id;

        if is_id_correct || is_pair {
            return unsafe {
                self.field_internal_result::<T::UnderlyingType, LOCK>(index, Entity(term_id))
            };
        }

        Err(FieldError::WrongType)
    }

    fn field_at_checked<T: ComponentId>(
        &self,
        index: i8,
        row: usize,
    ) -> Option<FieldAt<T::UnderlyingType>> {
        let id = <T::UnderlyingType as ComponentId>::id(self.world());

        if index > self.iter.field_count {
            return None;
        }

        let term_id = unsafe { sys::ecs_field_id(self.iter, index) };
        let is_pair = unsafe { sys::ecs_id_is_pair(term_id) };
        let is_id_correct = id == term_id;

        if is_id_correct || is_pair {
            return unsafe {
                self.field_at_internal::<T::UnderlyingType>(index, row, Entity(term_id))
            };
        }

        None
    }

    /// Get read access to field data.
    /// If the matched id for the specified field does not match with the provided
    /// type or if the field is readonly, the function will assert.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the field at `index` is accessible as `T`
    ///
    /// This function should not be used in `each_iter()` callbacks, unless it is to
    /// access a shared field. For access to non-shared fields in `each_iter()`, use
    /// `field_at`.
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
    pub fn field<T: ComponentId>(&self, index: i8) -> Option<Field<T::UnderlyingType, true>> {
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );

        self.field_checked::<T, true>(index)
    }

    /// Get read access to field data.
    /// If the matched id for the specified field does not match with the provided
    /// type or if the field is readonly, the function will assert.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the field at `index` is accessible as `T`
    ///
    /// This function should not be used in `each_iter()` callbacks, unless it is to
    /// access a shared field. For access to non-shared fields in `each_iter()`, use
    /// `field_at`.
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
    pub(crate) fn field_result<T: ComponentId>(
        &self,
        index: i8,
    ) -> Result<Field<T::UnderlyingType, true>, FieldError> {
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );

        self.field_checked_result::<T, true>(index)
    }

    pub(crate) fn field_lockless<T: ComponentId>(
        &self,
        index: i8,
    ) -> Option<Field<T::UnderlyingType, false>> {
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );

        self.field_checked::<T, false>(index)
    }

    /// Get read/write access to field data.
    /// If the matched id for the specified field does not match with the provided
    /// type or if the field is readonly, the function will assert.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the field at `index` is accessible as `T`
    ///
    /// This function should not be used in `each_iter()` callbacks, unless it is to
    /// access a shared field. For access to non-shared fields in `each_iter()`, use
    /// `field_at`.
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
    pub fn field_mut<T: ComponentId>(
        &self,
        index: i8,
    ) -> Option<FieldMut<T::UnderlyingType, true>> {
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );
        ecs_assert!(
            !unsafe { sys::ecs_field_is_readonly(self.iter, index) },
            FlecsErrorCode::AccessViolation,
            "field is readonly, check if your specified query terms are set &mut"
        );

        self.field_checked_mut::<T>(index)
    }

    /// Get read/write access to field data.
    /// If the matched id for the specified field does not match with the provided
    /// type or if the field is readonly, the function will assert.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the field at `index` is accessible as `T`
    ///
    /// This function should not be used in `each_iter()` callbacks, unless it is to
    /// access a shared field. For access to non-shared fields in `each_iter()`, use
    /// `field_at`.
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
    pub fn field_mut_result<T: ComponentId>(
        &self,
        index: i8,
    ) -> Result<FieldMut<T::UnderlyingType, true>, FieldError> {
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );
        ecs_assert!(
            !unsafe { sys::ecs_field_is_readonly(self.iter, index) },
            FlecsErrorCode::AccessViolation,
            "field is readonly, check if your specified query terms are set &mut"
        );

        self.field_checked_mut_result::<T>(index)
    }

    pub(crate) fn field_mut_lockless<T: ComponentId>(
        &self,
        index: i8,
    ) -> Option<FieldMut<T::UnderlyingType, false>> {
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0)
                || unsafe { sys::ecs_field_src(self.iter, index) != 0 },
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );
        ecs_assert!(
            !unsafe { sys::ecs_field_is_readonly(self.iter, index) },
            FlecsErrorCode::AccessViolation,
            "field is readonly, check if your specified query terms are set &mut"
        );

        self.field_checked_mut_lockless::<T>(index)
    }

    /// Get unchecked access to field data.
    /// Unchecked access is required when a system does not know the type of a field at compile time.
    /// This function may be used to access shared fields when row is set to 0.
    ///
    /// # Safety
    ///
    /// This function should not be used in `each_iter()` callbacks, unless it is to
    /// access a shared field. For access to non-shared fields in `each_iter()`, use
    /// `field_at`.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    ///
    /// # Returns
    ///
    /// Returns an `FieldUntyped` object that can be used to access the field data.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::field`
    #[doc(alias = "iter::field")]
    pub fn field_untyped(&self, index: i8) -> FieldUntyped {
        ecs_assert!(
            (self.iter.flags & sys::EcsIterCppEach == 0),
            FlecsErrorCode::InvalidOperation,
            "cannot .field from .each, use .field_at instead",
        );
        ecs_assert!(
            index < self.iter.field_count,
            FlecsErrorCode::InvalidParameter,
            index
        );
        self.field_untyped_internal(index)
    }

    /// Get field data for the specified index.
    /// Unchecked access is required when a system does not know the type of a field at compile time.
    /// This function may be used to access shared fields when row is set to 0.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    /// * `row` - The row index.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the field data.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::field_at`
    pub fn field_at_untyped(&self, index: i8, row: i32) -> *mut c_void {
        ecs_assert!(
            index < self.iter.field_count,
            FlecsErrorCode::InvalidParameter,
            index
        );
        if self.iter.row_fields & (1u32 << index) != 0 {
            let field = self.field_at_untyped_internal(index, row);
            return unsafe { field.array.add(0) };
        }
        let field = self.field_untyped_internal(index);
        unsafe { &mut *(field.array.add(row as usize * field.size)) }
    }

    /// Get field data for the specified index.
    /// This function may be used to access shared fields when row is set to 0.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    /// * `row` - The row index.
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of component to get the field data for
    ///
    /// # Returns
    ///
    /// An option containing a mutable reference to the field data
    ///
    /// # See also
    ///
    /// * C++ API: `iter::field_at`
    pub fn field_at_mut<T>(
        &self,
        index: i8,
        row: usize,
    ) -> Option<FieldAtMut<'_, T::UnderlyingType>>
    where
        T: ComponentId,
    {
        ecs_assert!(
            index < self.iter.field_count,
            FlecsErrorCode::InvalidParameter,
            index
        );
        ecs_assert!(
            !unsafe { sys::ecs_field_is_readonly(self.iter, index) },
            FlecsErrorCode::AccessViolation,
            "field is readonly, check if your specified query terms are set &mut"
        );
        if self.iter.row_fields & (1u32 << index) != 0 {
            self.get_field_at_internal_mut::<T>(index, row as i32)
        } else {
            self.field_at_checked_mut::<T>(index, row)
        }
    }

    /// Get field data for the specified index.
    /// This function may be used to access shared fields when row is set to 0.
    ///
    /// # Arguments
    ///
    /// * `index` - The field index.
    /// * `row` - The row index.
    ///
    /// # Type parameters
    ///
    /// * `T` - The type of component to get the field data for
    ///
    /// # Returns
    ///
    /// An option containing a reference to the field data
    ///
    /// # See also
    ///
    /// * C++ API: `iter::field_at`
    pub fn field_at<T>(&self, index: i8, row: usize) -> Option<FieldAt<T::UnderlyingType>>
    where
        T: ComponentId,
    {
        ecs_assert!(
            index < self.iter.field_count,
            FlecsErrorCode::InvalidParameter,
            index
        );

        if self.iter.row_fields & (1u32 << index) != 0 {
            self.get_field_at_internal::<T>(index, row as i32)
        } else {
            self.field_at_checked::<T>(index, row)
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
    ///     .add::<DerivedAction>()
    ///     .add::<DerivedAction2>();
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
        Id::new(id)
    }

    /// Get readonly access to entity ids of the sources.
    pub fn sources(&self) -> &[Entity] {
        unsafe {
            core::slice::from_raw_parts(
                self.iter.sources as *const Entity,
                self.iter.field_count as usize,
            )
        }
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
    pub fn entities(&self) -> &[Entity] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.iter.entities as *mut Entity,
                self.iter.count as usize,
            )
        }
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
    /// * C++ API: `iter::changed`
    #[doc(alias = "iter::changed")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `iter::skip`
    #[doc(alias = "iter::skip")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `iter::group_id`
    #[doc(alias = "iter::group_id")]
    pub fn group_id(&self) -> sys::ecs_id_t {
        self.iter.group_id
    }

    unsafe fn field_internal<T, const LOCK: bool>(
        &self,
        index: i8,
        _id: Entity,
    ) -> Option<Field<T, LOCK>> {
        unsafe {
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
            let array =
                sys::ecs_field_w_size(self.iter, core::mem::size_of::<T>(), index) as *const T;

            if array.is_null() {
                return None;
            }
            let slice = core::slice::from_raw_parts(array, count);

            #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
            {
                //does not actually do any locking
                Some(Field::<T, LOCK>::new(slice, is_shared))
            }

            #[cfg(feature = "flecs_safety_readwrite_locks")]
            {
                let tr = *self.iter.trs.add(index as usize);
                let table = (*tr).hdr.table;
                let world_ref = WorldRef::from_ptr(self.iter.world);
                Some(Field::<T, LOCK>::new(
                    slice,
                    is_shared,
                    world_ref.stage_id(),
                    (*tr).column,
                    index,
                    NonNull::new_unchecked(table),
                    &world_ref,
                ))
            }
        }
    }

    unsafe fn field_internal_result<T, const LOCK: bool>(
        &self,
        index: i8,
        _id: Entity,
    ) -> Result<Field<T, LOCK>, FieldError> {
        unsafe {
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
            let array =
                sys::ecs_field_w_size(self.iter, core::mem::size_of::<T>(), index) as *const T;

            if array.is_null() {
                return Err(FieldError::InvalidIndex);
            }
            let slice = core::slice::from_raw_parts(array, count);

            #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
            {
                //does not actually do any locking
                Ok(Field::<T, LOCK>::new(slice, is_shared))
            }

            #[cfg(feature = "flecs_safety_readwrite_locks")]
            {
                let tr = *self.iter.trs.add(index as usize);
                let table = (*tr).hdr.table;
                let world_ref = WorldRef::from_ptr(self.iter.world);
                Field::<T, LOCK>::new_result(
                    slice,
                    is_shared,
                    world_ref.stage_id(),
                    (*tr).column,
                    index,
                    NonNull::new_unchecked(table),
                    &world_ref,
                )
            }
        }
    }

    unsafe fn field_at_internal<T>(&self, index: i8, row: usize, _id: Entity) -> Option<FieldAt<T>>
    where
        T: ComponentId,
    {
        unsafe {
            let component =
                sys::ecs_field_at_w_size(self.iter, core::mem::size_of::<T>(), index, row as i32)
                    as *const T;

            if component.is_null() {
                return None;
            }
            let component_ref = &*component;

            #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
            {
                Some(FieldAt::<T>::new(component_ref))
            }

            #[cfg(feature = "flecs_safety_readwrite_locks")]
            {
                let tr = *self.iter.trs.add(index as usize);
                let idr = (*tr).hdr.cache as *mut sys::ecs_id_record_t;
                let world_ref = WorldRef::from_ptr(self.iter.world);

                Some(FieldAt::<T>::new(
                    component_ref,
                    &world_ref,
                    NonNull::new_unchecked(idr),
                ))
            }
        }
    }

    unsafe fn field_internal_mut<T, const LOCK: bool>(
        &self,
        index: i8,
        _id: Entity,
    ) -> Option<FieldMut<T, LOCK>> {
        unsafe {
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
            let array =
                sys::ecs_field_w_size(self.iter, core::mem::size_of::<T>(), index) as *mut T;

            if array.is_null() {
                return None;
            }
            let slice = core::slice::from_raw_parts_mut(array, count);

            #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
            {
                Some(FieldMut::<T, LOCK>::new(slice, is_shared))
            }

            #[cfg(feature = "flecs_safety_readwrite_locks")]
            {
                let tr = *self.iter.trs.add(index as usize);
                let table = (*tr).hdr.table;
                let world_ref = WorldRef::from_ptr(self.iter.world);
                let column_index = (*tr).column;
                Some(FieldMut::<T, LOCK>::new(
                    slice,
                    is_shared,
                    world_ref.stage_id(),
                    column_index,
                    index,
                    NonNull::new_unchecked(table),
                    &world_ref,
                ))
            }
        }
    }

    unsafe fn field_internal_mut_result<T, const LOCK: bool>(
        &self,
        index: i8,
        _id: Entity,
    ) -> Result<FieldMut<T, LOCK>, FieldError> {
        unsafe {
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
            let array =
                sys::ecs_field_w_size(self.iter, core::mem::size_of::<T>(), index) as *mut T;

            if array.is_null() {
                return Err(FieldError::InvalidIndex);
            }
            let slice = core::slice::from_raw_parts_mut(array, count);

            #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
            {
                Ok(FieldMut::<T, LOCK>::new(slice, is_shared))
            }

            #[cfg(feature = "flecs_safety_readwrite_locks")]
            {
                let tr = *self.iter.trs.add(index as usize);
                let table = (*tr).hdr.table;
                let world_ref = WorldRef::from_ptr(self.iter.world);
                let column_index = (*tr).column;
                FieldMut::<T, LOCK>::new_result(
                    slice,
                    is_shared,
                    world_ref.stage_id(),
                    column_index,
                    index,
                    NonNull::new_unchecked(table),
                    &world_ref,
                )
            }
        }
    }

    unsafe fn field_at_internal_mut<T>(
        &self,
        index: i8,
        row: usize,
        _id: Entity,
    ) -> Option<FieldAtMut<T>>
    where
        T: ComponentId,
    {
        unsafe {
            let component =
                sys::ecs_field_at_w_size(self.iter, core::mem::size_of::<T>(), index, row as i32)
                    as *mut T;

            if component.is_null() {
                return None;
            }
            let component_ref = &mut *component;

            #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
            {
                Some(FieldAtMut::<T>::new(component_ref))
            }

            #[cfg(feature = "flecs_safety_readwrite_locks")]
            {
                let world_ref = WorldRef::from_ptr(self.iter.world);
                let tr = *self.iter.trs.add(index as usize);
                let idr = (*tr).hdr.cache as *mut sys::ecs_id_record_t;
                Some(FieldAtMut::<T>::new(
                    component_ref,
                    &world_ref,
                    NonNull::new_unchecked(idr),
                ))
            }
        }
    }

    fn field_untyped_internal(&self, index: i8) -> FieldUntyped {
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

        FieldUntyped::new(
            unsafe { sys::ecs_field_w_size(self.iter, 0, index) },
            size,
            count,
            is_shared,
        )
    }

    fn field_at_untyped_internal(&self, index: i8, row: i32) -> FieldUntyped {
        let size = unsafe { sys::ecs_field_size(self.iter, index) };

        FieldUntyped::new(
            unsafe { sys::ecs_field_at_w_size(self.iter, 0, index, row) },
            size,
            1,
            false,
        )
    }

    // get field, check if correct type is used
    fn get_field_at_internal<T>(&self, index: i8, row: i32) -> Option<FieldAt<T::UnderlyingType>>
    where
        T: ComponentId,
    {
        unsafe {
            let id = <T::UnderlyingType as ComponentId>::id(self.world());

            if index > self.iter.field_count {
                return None;
            }

            let term_id = sys::ecs_field_id(self.iter, index);
            let is_pair = sys::ecs_id_is_pair(term_id);
            let is_id_correct = id == term_id;

            if is_id_correct || is_pair {
                let component = sys::ecs_field_at_w_size(
                    self.iter,
                    core::mem::size_of::<T::UnderlyingType>(),
                    index,
                    row,
                ) as *const T::UnderlyingType;

                if component.is_null() {
                    return None;
                }

                let component_ref = &*component;

                #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
                {
                    return Some(FieldAt::<T::UnderlyingType>::new(component_ref));
                }

                #[cfg(feature = "flecs_safety_readwrite_locks")]
                {
                    let tr = *self.iter.trs.add(index as usize);
                    let idr = (*tr).hdr.cache as *mut sys::ecs_id_record_t;
                    let world_ref = WorldRef::from_ptr(self.iter.world);
                    return Some(FieldAt::<T::UnderlyingType>::new(
                        component_ref,
                        &world_ref,
                        NonNull::new_unchecked(idr),
                    ));
                }
            }

            None
        }
    }

    fn get_field_at_internal_mut<T>(
        &self,
        index: i8,
        row: i32,
    ) -> Option<FieldAtMut<T::UnderlyingType>>
    where
        T: ComponentId,
    {
        unsafe {
            let id = <T::UnderlyingType as ComponentId>::id(self.world());

            if index > self.iter.field_count {
                return None;
            }

            let term_id = sys::ecs_field_id(self.iter, index);
            let is_pair = sys::ecs_id_is_pair(term_id);
            let is_id_correct = id == term_id;

            if is_id_correct || is_pair {
                let component = sys::ecs_field_at_w_size(
                    self.iter,
                    core::mem::size_of::<T::UnderlyingType>(),
                    index,
                    row,
                ) as *mut T::UnderlyingType;

                if component.is_null() {
                    return None;
                }

                let component_ref = &mut *component;

                #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
                {
                    return Some(FieldAtMut::<T::UnderlyingType>::new(component_ref));
                }

                #[cfg(feature = "flecs_safety_readwrite_locks")]
                {
                    let tr = *self.iter.trs.add(index as usize);
                    let idr = (*tr).hdr.cache as *mut sys::ecs_id_record_t;
                    let world_ref = WorldRef::from_ptr(self.iter.world);
                    return Some(FieldAtMut::<T::UnderlyingType>::new(
                        component_ref,
                        &world_ref,
                        NonNull::new_unchecked(idr),
                    ));
                }
            }

            None
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
    /// let alice = world.entity().add_id((likes, pizza)).add_id((likes, salad));
    ///
    /// let q = world.query::<()>().with_second::<flecs::Any>(likes).build();
    ///
    /// let mut count = 0;
    /// let mut tgt_count = 0;
    ///
    /// q.each_iter(|mut it, row, _| {
    ///     let e = it.entity(row);
    ///     assert_eq!(e, alice);
    ///
    ///     it.targets(0, |tgt| {
    ///         if tgt_count == 0 {
    ///             assert_eq!(tgt, pizza);
    ///         }
    ///         if tgt_count == 1 {
    ///             assert_eq!(tgt, salad);
    ///         }
    ///         tgt_count += 1;
    ///     });
    ///
    ///     count += 1;
    /// });
    ///
    /// assert_eq!(count, 1);
    /// assert_eq!(tgt_count, 2);
    /// ```
    ///
    /// # See Also
    ///
    /// * C++ API: `iter::targets`
    #[doc(alias = "iter::targets")]
    pub fn targets(&mut self, index: i8, mut func: impl FnMut(EntityView)) {
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
            let target = EntityView::new_from(self.world(), ecs_second(id));
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
    ///
    /// # Safety
    ///
    /// This operation is valid inside a `run()` callback. An example of an
    /// invalid context is inside an `each()` callback.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::next`
    #[doc(alias = "iter::next")]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.iter.flags & sys::EcsIterIsValid != 0 && !self.iter.table.is_null() {
            unsafe {
                sys::ecs_table_unlock(self.iter.world, self.iter.table);
            };
        }

        let result = {
            if let Some(next) = self.iter.next {
                //sets flag invalid
                unsafe { next(self.iter) }
            } else {
                self.iter.flags &= !sys::EcsIterIsValid;
                return false;
            }
        };

        self.iter.flags |= sys::EcsIterIsValid;
        if result && !self.iter.table.is_null() {
            unsafe {
                sys::ecs_table_lock(self.iter.world, self.iter.table);
            };
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

/// Iterator to iterate over rows in a table
pub struct TableRowIter<'a, const IS_RUN: bool, P> {
    iter: &'a TableIter<'a, IS_RUN, P>,
    index: usize,
}

impl<const IS_RUN: bool, P> Iterator for TableRowIter<'_, IS_RUN, P>
where
    P: ComponentId,
{
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
