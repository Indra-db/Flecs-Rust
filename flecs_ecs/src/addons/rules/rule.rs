use std::{ffi::CStr, ops::Deref, os::raw::c_void};

use crate::core::*;
use crate::sys;

pub struct Rule<'a, T>
where
    T: Iterable,
{
    rule: *mut sys::ecs_rule_t,
    pub world: WorldRef<'a>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> Deref for Rule<'a, T>
where
    T: Iterable,
{
    type Target = *mut sys::ecs_rule_t;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.rule
    }
}

impl<'a, T> Drop for Rule<'a, T>
where
    T: Iterable,
{
    fn drop(&mut self) {
        if !self.rule.is_null() {
            unsafe {
                sys::ecs_rule_fini(self.rule);
            }
        }
    }
}

impl<'a, T> Rule<'a, T>
where
    T: Iterable,
{
    /// Create a new rule
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the rule in
    /// * `rule` - The rule to create the rule from
    ///
    /// # See also
    ///
    /// * C++ API: `rule_base::rule`
    #[doc(alias = "rule_base::rule")]
    pub fn new(world: impl IntoWorld<'a>, rule: *mut sys::ecs_rule_t) -> Self {
        Self {
            world: world.world(),
            rule,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new rule from a rule descriptor
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the rule in
    /// * `desc` - The rule descriptor to create the rule from
    ///
    /// # See also
    ///
    /// * C++ API: `rule_base::rule`
    #[doc(alias = "rule_base::rule")]
    pub fn new_from_desc(world: impl IntoWorld<'a>, desc: &mut sys::ecs_filter_desc_t) -> Self {
        let obj = Self {
            world: world.world(),
            rule: unsafe { sys::ecs_rule_init(world.world_ptr_mut(), desc) },
            _phantom: std::marker::PhantomData,
        };
        if !desc.terms_buffer.is_null() {
            unsafe {
                if let Some(free_func) = sys::ecs_os_api.free_ {
                    free_func(desc.terms_buffer as *mut _);
                }
            }
        }
        obj
    }

    /// Returns whether the rule is valid
    ///
    /// # See also
    ///
    /// * C++ API: `rule_base::is_valid`
    #[doc(alias = "rule_base::is_valid")]
    pub fn is_valid(&self) -> bool {
        !self.rule.is_null()
    }

    /// Free the rule
    ///
    /// # See also
    ///
    /// * C++ API: `rule_base::destruct`
    #[doc(alias = "rule_base::destruct")]
    pub fn destruct(self) {
        //drops the rule
    }

    /// Returns the filter of the rule
    ///
    /// # See also
    ///
    /// * C++ API: `rule_base::filter`
    #[doc(alias = "rule_base::filter")]
    pub fn filter(&self) -> FilterView<T> {
        FilterView::new(self.world, unsafe { sys::ecs_rule_get_filter(self.rule) })
    }

    /// Converts this rule to a string that can be used to aid debugging
    /// the behavior of the rule.
    ///
    /// # See also
    ///
    /// * C++ API: `rule_base::rule_str`
    #[doc(alias = "rule_base::rule_str")]
    pub fn to_rule_string(&self) -> String {
        let str: *mut i8 = unsafe { sys::ecs_rule_str(self.rule) };
        let rust_string = String::from(unsafe { std::ffi::CStr::from_ptr(str).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = sys::ecs_os_api.free_ {
                free_func(str as *mut _);
            }
        }
        rust_string
    }

    pub fn find_var(&self, name: &CStr) -> i32 {
        unsafe { sys::ecs_rule_find_var(self.rule, name.as_ptr()) }
    }
}

impl<'a, T> IterAPI<'a, T> for Rule<'a, T>
where
    T: Iterable,
{
    fn as_entity(&self) -> EntityView<'a> {
        EntityView::new_from(self.world, unsafe {
            sys::ecs_get_entity(self.rule as *const c_void)
        })
    }
}

impl<'a, T: Iterable> IntoWorld<'a> for Rule<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a, T> IterOperations for Rule<'a, T>
where
    T: Iterable,
{
    fn retrieve_iter(&self) -> crate::core::IterT {
        unsafe { sys::ecs_rule_iter(self.world.world_ptr_mut(), self.rule) }
    }

    fn iter_next(&self, iter: &mut crate::core::IterT) -> bool {
        unsafe { sys::ecs_rule_next(iter) }
    }

    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut crate::core::IterT) -> bool {
        sys::ecs_rule_next
    }

    fn filter_ptr(&self) -> *const crate::core::FilterT {
        unsafe { sys::ecs_rule_get_filter(self.rule) }
    }
}
