//! Filters are cheaper to create, but slower to iterate than queries.
use std::{ffi::CStr, os::raw::c_void, ptr};

use super::{
    builder::Builder,
    c_types::{TermT, SEPARATOR},
    component_registration::{ComponentId, ComponentType, Enum},
    filter::Filter,
    iterable::{Filterable, Iterable},
    term::{Term, TermBuilder},
    type_to_inout,
    world::World,
    CachedEnumData, IdT, InOutType, IntoComponentId, IntoEntityId, IntoEntityIdExt, WorldT,
    ECS_WILDCARD,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::{core::FlecsErrorCode, sys::ecs_term_is_initialized};

use crate::{
    ecs_assert,
    sys::{
        ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t, ecs_flags32_t, ecs_inout_kind_t,
        ecs_os_api, ecs_term_t, FLECS_TERM_DESC_MAX,
    },
};

/// Filters are cheaper to create, but slower to iterate than queries.
pub struct FilterBuilder<T>
where
    T: Iterable,
{
    pub desc: ecs_filter_desc_t,
    expr_count: i32,
    pub(crate) term: Term,
    pub world: World,
    pub next_term_index: i32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FilterBuilder<T>
where
    T: Iterable,
{
    /// Create a new filter builder.
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter builder in.
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder::filter_builder`
    #[doc(alias = "filter_builder::filter_builder")]
    pub fn new(world: &World) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            expr_count: 0,
            term: Term::new_world_only(world),
            world: world.clone(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };

        let entity_desc = ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }

    /// Create a new filter builder with a name.
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter builder in.
    /// * `name`: the name of the filter.
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder::filter_builder`
    #[doc(alias = "filter_builder::filter_builder")]
    pub fn new_named(world: &World, name: &CStr) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            expr_count: 0,
            term: Term::default(),
            world: world.clone(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };

        let entity_desc = ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        T::populate(&mut obj);
        obj
    }

    /// Create a new filter builder from a filter description.
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter builder in.
    /// * `desc`: the filter description to create the filter builder from.
    /// * `term_index`: the term index to create the filter builder from.
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder::filter_builder`
    #[doc(alias = "filter_builder::filter_builder")]
    pub fn new_from_desc(world: &World, desc: &mut ecs_filter_desc_t, term_index: i32) -> Self {
        Self {
            desc: *desc,
            expr_count: 0,
            term: Term::default(),
            world: world.clone(),
            next_term_index: term_index,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Increment the term index
    pub fn next_term(&mut self) {
        self.next_term_index += 1;
    }
}

impl<T> Filterable for FilterBuilder<T>
where
    T: Iterable,
{
    fn current_term(&mut self) -> &mut TermT {
        unsafe { &mut *self.term.term_ptr }
    }

    fn next_term(&mut self) {
        self.next_term();
    }
}

impl<T> FilterBuilderImpl for FilterBuilder<T>
where
    T: Iterable,
{
    #[inline]
    fn desc_filter_mut(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.desc
    }

    #[inline]
    fn expr_count_mut(&mut self) -> &mut i32 {
        &mut self.expr_count
    }

    #[inline]
    fn term_index_mut(&mut self) -> &mut i32 {
        &mut self.next_term_index
    }
}

impl<T> TermBuilder for FilterBuilder<T>
where
    T: Iterable,
{
    #[inline]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world.raw_world
    }

    #[inline]
    fn term_mut(&mut self) -> &mut Term {
        &mut self.term
    }

    #[inline]
    fn term_ptr_mut(&mut self) -> *mut super::c_types::TermT {
        self.term.term_ptr
    }

    #[inline]
    fn term_id_ptr_mut(&mut self) -> *mut super::c_types::TermIdT {
        self.term.term_id_ptr
    }
}

impl<T> Builder for FilterBuilder<T>
where
    T: Iterable,
{
    type BuiltType = Filter<T>;

    #[inline]
    fn build(&mut self) -> Self::BuiltType {
        Filter::<T>::new_from_desc(&self.world, &mut self.desc as *mut _)
    }
}

pub trait FilterBuilderImpl: TermBuilder {
    fn desc_filter_mut(&mut self) -> &mut ecs_filter_desc_t;

    fn expr_count_mut(&mut self) -> &mut i32;

    fn term_index_mut(&mut self) -> &mut i32;

    /// set itself to be instanced
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::instanced`
    #[doc(alias = "filter_builder_i::instanced")]
    fn instanced(&mut self) -> &mut Self {
        self.desc_filter_mut().instanced = true;
        self
    }

    /// set filter flags
    ///
    /// # Arguments
    ///
    /// * `flags` - the flags to set
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::filter_flags`
    #[doc(alias = "filter_builder_i::filter_flags")]
    fn filter_flags(&mut self, flags: ecs_flags32_t) -> &mut Self {
        self.desc_filter_mut().flags |= flags;
        self
    }

    /// set expression
    ///
    /// # Arguments
    ///
    /// * `expr` - the expression to set
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::expr`
    #[doc(alias = "filter_builder_i::expr")]
    fn expr(&mut self, expr: &CStr) -> &mut Self {
        ecs_assert!(
            *self.expr_count_mut() == 0,
            FlecsErrorCode::InvalidOperation,
            "filter_builder::expr() called more than once"
        );

        self.desc_filter_mut().expr = expr.as_ptr();
        *self.expr_count_mut() += 1;
        self
    }

    /// set term
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with(&mut self, with: FilterType) -> &mut Self {
        match with {
            FilterType::Id(id) => self.term_with_id(id).inout_none(),
            FilterType::Name(name) => self.term_with_name(name).inout_none(),
            FilterType::PairIds(first, second) => self.term_with_id((first, second)).inout_none(),
            FilterType::PairNames(first, second) => {
                self.term_with_pair_names(first, second).inout_none()
            }
            FilterType::PairIdName(first, second) => {
                self.term_with_pair_id_name(first, second).inout_none()
            }
            FilterType::Term(term) => self.term_with_term(term).inout_none(),
        }
    }

    /// set term with pair id
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with_pair_first<First: ComponentId>(&mut self, second: impl IntoEntityId) -> &mut Self {
        self.term_with_pair_first::<First>(second)
    }

    /// set term with pair name
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with_pair_name<First: ComponentId>(&mut self, second: &'static CStr) -> &mut Self {
        self.term_with_pair_name::<First>(second)
    }

    /// set term with enum
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.term_with_enum(value)
    }

    /// set term with enum wildcard
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with_enum_wildcard<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
    ) -> &mut Self {
        self.term_with_pair_first::<T>(ECS_WILDCARD)
    }

    /// set term with type
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with_type<T: InOutType>(&mut self) -> &mut Self {
        self.term_with::<T>()
    }

    /// set term without Id or Name or Pair or Term
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without(&mut self, without: FilterType) -> &mut Self {
        match without {
            FilterType::Id(id) => self.term_with_id(id).not(),
            FilterType::Name(name) => self.term_with_name(name).not(),
            FilterType::PairIds(first, second) => self.term_with_id((first, second)).not(),
            FilterType::PairNames(first, second) => self.term_with_pair_names(first, second).not(),
            FilterType::PairIdName(first, second) => {
                self.term_with_pair_id_name(first, second).not()
            }
            FilterType::Term(term) => self.term_with_term(term).not(),
        }
    }

    /// set term without pair id
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without_pair_id<First: ComponentId>(&mut self, second: impl IntoEntityId) -> &mut Self {
        self.term_with_pair_first::<First>(second).not()
    }

    /// set term without pair name
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without_pair_name<First: ComponentId>(&mut self, second: &'static CStr) -> &mut Self {
        self.term_with_pair_name::<First>(second).not()
    }

    /// set term without enum
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.term_with_enum(value).not()
    }

    /// set term without pair
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without_pair<First: ComponentId, Second: ComponentId>(&mut self) -> &mut Self {
        let world = self.world_ptr_mut();
        self.term_with_id((First::get_id(world), Second::get_id(world)))
            .not()
    }

    /// set term without type
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without_type<T: InOutType>(&mut self) -> &mut Self {
        self.term_with::<T>().not()
    }

    /// Term notation for more complex query features
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term(&mut self) {
        ecs_assert!(
            if !self.term_ptr_mut().is_null() {
                unsafe { ecs_term_is_initialized(self.term_ptr_mut()) }
            } else {
                true
            },
            FlecsErrorCode::InvalidOperation,
            "FilterBuilder::term() called without initializing term"
        );

        let term_index = *self.term_index_mut();
        if term_index >= FLECS_TERM_DESC_MAX as i32 {
            let desc = self.desc_filter_mut();
            let size_term = std::mem::size_of::<ecs_term_t>();
            if term_index == FLECS_TERM_DESC_MAX as i32 {
                unsafe {
                    desc.terms_buffer =
                        ecs_os_api.calloc_.unwrap()(size_term as i32 * term_index + 1)
                            as *mut ecs_term_t;
                    // SAFETY: The following conditions must hold:
                    // - `src` and `dst` must not overlap.
                    // - `src` and `dst` must be valid for reads and writes of `src.len()` elements.
                    // - `src.len()` must be equal to `dst.len()`.
                    ptr::copy_nonoverlapping(
                        desc.terms.as_ptr() as *mut c_void,
                        desc.terms_buffer as *mut _,
                        size_term * term_index as usize,
                    );
                    ptr::write_bytes(
                        desc.terms.as_mut_ptr() as *mut _,
                        0,
                        size_term * FLECS_TERM_DESC_MAX as usize,
                    );
                }
            } else {
                desc.terms_buffer = unsafe {
                    ecs_os_api.realloc_.unwrap()(
                        desc.terms_buffer as *mut _,
                        size_term as i32 * term_index,
                    ) as *mut ecs_term_t
                };
            }
            desc.terms_buffer_count = term_index + 1;
            let term_to_set = unsafe { desc.terms_buffer.add(term_index as usize) };
            self.set_term(term_to_set);
        } else {
            let term_to_set = unsafe {
                self.desc_filter_mut()
                    .terms
                    .as_mut_ptr()
                    .add(term_index as usize)
            };
            self.set_term(term_to_set);
        }
        *self.term_index_mut() += 1;
    }

    /// Term notation for more complex query features
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term_at`
    #[doc(alias = "filter_builder_i::term_at")]
    fn term_at(&mut self, index: i32) -> &mut Self {
        ecs_assert!(
            index > 0,
            FlecsErrorCode::InvalidParameter,
            "term_at() called with invalid index"
        );

        let term_index = *self.term_index_mut();
        let prev_index = term_index;

        *self.term_index_mut() = index - 1;
        self.term();

        *self.term_index_mut() = prev_index;

        ecs_assert!(
            unsafe { ecs_term_is_initialized(self.term_ptr_mut()) },
            FlecsErrorCode::InvalidOperation,
            "term_at() called without initializing term"
        );

        self
    }

    fn arg(&mut self, index: i32) -> &mut Self {
        self.term_at(index)
    }

    fn write(&mut self) -> &mut Self {
        self.term_mut().write_();
        self
    }

    fn write_type<T: InOutType>(&mut self) -> &mut Self {
        self.term_with::<T>();
        FilterBuilderImpl::write(self)
    }

    fn write_id(&mut self, id: impl IntoEntityIdExt) -> &mut Self {
        self.term_with_id(id);
        FilterBuilderImpl::write(self)
    }

    /// set term with Id
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_id(&mut self, id: impl IntoEntityIdExt) -> &mut Self {
        self.term();
        unsafe {
            *self.term_ptr_mut() = Term::new_id(None, id).move_raw_term();
        }
        self
    }

    /// set term with Component or pair
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with<T: InOutType>(&mut self) -> &mut Self {
        if <T::Type as IntoComponentId>::IS_PAIR {
            let world = self.world_ptr_mut();
            self.term_with_id(<T::Type as IntoComponentId>::get_id(world));
        } else {
            self.term();

            unsafe {
                *self.term_ptr_mut() = Term::new_id(
                    None,
                    <T::Type as IntoComponentId>::get_id(self.world_ptr_mut()),
                )
                .move_raw_term();
                (*self.term_ptr_mut()).inout = type_to_inout::<T>() as ecs_inout_kind_t;
            }
        }
        self
    }

    /// set term with Name
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_name(&mut self, name: &'static CStr) -> &mut Self {
        self.term();
        unsafe {
            *self.term_ptr_mut() = Term::default().select_first_name(name).move_raw_term();
        }
        self
    }

    /// set term with Pair Names
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_pair_names(&mut self, first: &'static CStr, second: &'static CStr) -> &mut Self {
        self.term();
        unsafe {
            *self.term_ptr_mut() = Term::default()
                .select_first_name(first)
                .select_second_name(second)
                .move_raw_term();
        }
        self
    }

    /// set term with Pair Id Name
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_pair_id_name(
        &mut self,
        first: impl IntoEntityId,
        second: &'static CStr,
    ) -> &mut Self {
        self.term();
        unsafe {
            *self.term_ptr_mut() = Term::new_id(None, first.get_id())
                .select_second_name(second)
                .move_raw_term();
        }
        self
    }

    /// set term with Pair
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_pair_first<First: ComponentId>(&mut self, second: impl IntoEntityId) -> &mut Self {
        let world = self.world_ptr_mut();
        self.term_with_id((First::get_id(world), second))
    }

    /// set term with Pair
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_pair_name<First: ComponentId>(&mut self, second: &'static CStr) -> &mut Self {
        let world = self.world_ptr_mut();
        self.term_with_id(First::get_id(world))
            .select_second_name(second)
    }

    /// set term with enum
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        let enum_id = T::get_id(self.world_ptr_mut());
        // SAFETY: we know that the enum_value is a valid because of the T::get_id call
        let enum_field_id = value.get_id_variant(self.world_ptr_mut());
        self.term_with_id((enum_id, enum_field_id))
    }

    /// set term with term
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_term(&mut self, mut term: Term) -> &mut Self {
        self.term();
        unsafe {
            *self.term_ptr_mut() = term.move_raw_term();
        }
        self
    }
}

pub enum FilterType {
    Id(IdT),
    Name(&'static CStr),
    PairIds(IdT, IdT),
    PairNames(&'static CStr, &'static CStr),
    PairIdName(IdT, &'static CStr),
    Term(Term),
}
