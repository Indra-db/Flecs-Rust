use std::ffi::{c_void, CStr};

use flecs_ecs_sys::{
    ecs_get_entity, ecs_iter_set_var, ecs_iter_set_var_as_range, ecs_query_set_group,
    ecs_rule_find_var, ecs_rule_iter_t,
};

use super::{
    ComponentId, Entity, FilterT, FromWorldPtr, IntoEntityId, IntoTableRange, IntoWorld, IterAPI,
    IterOperations, IterT, Iterable, World, WorldRef,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;
use crate::ecs_assert;

pub struct IterIterable<'a, T>
where
    T: Iterable,
{
    iter: IterT,
    iter_next: unsafe extern "C" fn(*mut IterT) -> bool,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> IterIterable<'a, T>
where
    T: Iterable,
{
    pub fn new(iter: IterT, iter_next: unsafe extern "C" fn(*mut IterT) -> bool) -> Self {
        Self {
            iter,
            iter_next,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Limit results to tables with specified group id (grouped queries only)
    ///
    /// # Arguments
    ///
    /// * `group_id`: the group id to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_group`
    #[doc(alias = "iter_iterable::set_group")]
    pub fn set_group_id(&mut self, group_id: impl IntoEntityId) {
        unsafe { ecs_query_set_group(&mut self.iter, group_id.get_id()) }
    }

    /// Limit results to tables with specified group id (grouped queries only)
    ///
    /// # Type parameters
    ///
    /// * `Group`: the group to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_group`
    #[doc(alias = "iter_iterable::set_group")]
    pub fn set_group<Group: ComponentId>(&mut self) -> &Self {
        let world = unsafe { Option::<WorldRef>::from_ptr(self.iter.real_world) };
        unsafe { ecs_query_set_group(&mut self.iter, Group::get_id(world)) }
        self
    }

    /// set variable of iter
    ///
    /// # Arguments
    ///
    /// * `var_id`: the variable id to set
    ///
    /// * `value`: the value to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_var`
    #[doc(alias = "iter_iterable::set_var")]
    pub fn set_var(&mut self, var_id: i32, value: impl IntoEntityId) -> &mut Self {
        ecs_assert!(var_id != -1, FlecsErrorCode::InvalidParameter, 0);
        unsafe { ecs_iter_set_var(&mut self.iter, var_id, value.get_id()) };
        self
    }

    /// set variable of iter as range
    ///
    /// # Arguments
    ///
    /// * `var_id`: the variable id to set
    ///
    /// * `range`: the range to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_var_as_range`
    #[doc(alias = "iter_iterable::set_var_as_range")]
    pub fn set_var_as_range(&mut self, var_id: i32, range: impl IntoTableRange) -> &mut Self {
        ecs_assert!(var_id != -1, FlecsErrorCode::InvalidParameter, 0);
        unsafe { ecs_iter_set_var_as_range(&mut self.iter, var_id, &range.table_range_raw()) };
        self
    }

    /// set variable for rule iter
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the variable to set
    /// * `value`: the value to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_var`
    #[doc(alias = "iter_iterable::set_var_rule")]
    #[cfg(feature = "flecs_rules")]
    pub fn set_var_rule(&mut self, name: &CStr, value: impl IntoEntityId) -> &mut Self {
        let rit: *mut ecs_rule_iter_t = unsafe { &mut self.iter.priv_.iter.rule };
        let var_id = unsafe { ecs_rule_find_var((*rit).rule, name.as_ptr()) };
        ecs_assert!(
            var_id != -1,
            FlecsErrorCode::InvalidParameter,
            name.to_str().unwrap()
        );
        unsafe { ecs_iter_set_var(&mut self.iter, var_id, value.get_id()) };
        self
    }
}

impl<'a, T> IterOperations for IterIterable<'a, T>
where
    T: Iterable,
{
    fn retrieve_iter(&self) -> IterT {
        self.iter
    }

    fn iter_next(&self, iter: &mut IterT) -> bool {
        unsafe { (self.iter_next)(iter) }
    }

    fn filter_ptr(&self) -> *const FilterT {
        self.iter.query
    }

    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut IterT) -> bool {
        self.iter_next
    }
}

impl<'a, T> IterAPI<'a, T> for IterIterable<'a, T>
where
    T: Iterable,
{
    fn as_entity(&self) -> Entity<'a> {
        let world = unsafe { Option::<WorldRef>::from_ptr(self.iter.real_world) };
        Entity::new_from_existing(world, unsafe {
            ecs_get_entity(self.iter.query as *const c_void)
        })
    }
}

impl<'a, T> IntoWorld<'a> for IterIterable<'a, T>
where
    T: Iterable,
{
    fn get_world(&self) -> Option<&'a World> {
        unsafe { Option::<WorldRef>::from_ptr(self.iter.real_world) }.get_world()
    }
}

// TODO : worker_iterable and page_iterable not implemented yet
