//! Filters are cheaper to create, but slower to iterate than queries.
use std::{ffi::CStr, os::raw::c_void, ptr};

use crate::{
    core::FlecsErrorCode,
    ecs_assert,
    sys::{
        ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t, ecs_flags32_t, ecs_inout_kind_t,
        ecs_os_api, ecs_term_is_initialized, ecs_term_t, EcsWildcard, FLECS_TERM_DESC_MAX,
    },
};

use super::{
    builder::Builder,
    c_types::{IdT, TermT, WorldT, SEPARATOR},
    component_registration::{CachedComponentData, ComponentType, Enum},
    enum_type::CachedEnumData,
    filter::Filter,
    iterable::{Filterable, Iterable},
    term::{Term, TermBuilder, TermType},
    type_to_inout,
    world::World,
    InOutType,
};

/// Filters are cheaper to create, but slower to iterate than queries.
pub struct FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    pub desc: ecs_filter_desc_t,
    expr_count: i32,
    term: Term,
    pub world: World,
    pub next_term_index: i32,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> FilterBuilder<'a, T>
where
    T: Iterable<'a>,
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

impl<'a, T> Filterable for FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    fn current_term(&mut self) -> &mut TermT {
        let next_term_index = self.next_term_index;
        &mut self.get_desc_filter().terms[next_term_index as usize]
    }

    fn next_term(&mut self) {
        self.next_term();
    }
}

impl<'a, T> FilterBuilderImpl for FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_filter(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.desc
    }

    #[inline]
    fn get_expr_count(&mut self) -> &mut i32 {
        &mut self.expr_count
    }

    #[inline]
    fn get_term_index(&mut self) -> &mut i32 {
        &mut self.next_term_index
    }
}

impl<'a, T> TermBuilder for FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_world(&self) -> *mut WorldT {
        self.world.raw_world
    }

    #[inline]
    fn get_term(&mut self) -> &mut Term {
        &mut self.term
    }

    #[inline]
    fn get_raw_term(&mut self) -> *mut super::c_types::TermT {
        self.term.term_ptr
    }

    #[inline]
    fn get_term_id(&mut self) -> *mut super::c_types::TermIdT {
        self.term.term_id
    }
}

impl<'a, T> Builder for FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type BuiltType = Filter<'a, T>;

    #[inline]
    fn build(&mut self) -> Self::BuiltType {
        Filter::<'a, T>::new_from_desc(&self.world, &mut self.desc as *mut _)
    }
}

pub trait FilterBuilderImpl: TermBuilder {
    fn get_desc_filter(&mut self) -> &mut ecs_filter_desc_t;

    fn get_expr_count(&mut self) -> &mut i32;

    fn get_term_index(&mut self) -> &mut i32;

    /// set itself to be instanced
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::instanced`
    #[doc(alias = "filter_builder_i::instanced")]
    fn instanced(&mut self) -> &mut Self {
        self.get_desc_filter().instanced = true;
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
        self.get_desc_filter().flags |= flags;
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
            *self.get_expr_count() == 0,
            FlecsErrorCode::InvalidOperation,
            "filter_builder::expr() called more than once"
        );

        self.get_desc_filter().expr = expr.as_ptr();
        *self.get_expr_count() += 1;
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
            FilterType::PairIds(rel, target) => self.term_with_pair_ids(rel, target).inout_none(),
            FilterType::PairNames(rel, target) => {
                self.term_with_pair_names(rel, target).inout_none()
            }
            FilterType::PairIdName(rel, target) => {
                self.term_with_pair_id_name(rel, target).inout_none()
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
    fn with_pair_id<Rel: CachedComponentData>(&mut self, target: IdT) -> &mut Self {
        self.term_with_pair_id::<Rel>(target)
    }

    /// set term with pair name
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with_pair_name<Rel: CachedComponentData>(&mut self, target: &'static CStr) -> &mut Self {
        self.term_with_pair_name::<Rel>(target)
    }

    /// set term with enum
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
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
    fn with_enum_wildcard<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &mut self,
    ) -> &mut Self {
        self.term_with_pair_id::<T>(unsafe { EcsWildcard })
    }

    /// set term with pair
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::with`
    #[doc(alias = "filter_builder_i::with")]
    fn with_pair<Rel: CachedComponentData, Target: CachedComponentData>(&mut self) -> &mut Self {
        self.term_with_pair::<Rel, Target>()
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
            FilterType::PairIds(rel, target) => self.term_with_pair_ids(rel, target).not(),
            FilterType::PairNames(rel, target) => self.term_with_pair_names(rel, target).not(),
            FilterType::PairIdName(rel, target) => self.term_with_pair_id_name(rel, target).not(),
            FilterType::Term(term) => self.term_with_term(term).not(),
        }
    }

    /// set term without pair id
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without_pair_id<Rel: CachedComponentData>(&mut self, target: IdT) -> &mut Self {
        self.term_with_pair_id::<Rel>(target).not()
    }

    /// set term without pair name
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without_pair_name<Rel: CachedComponentData>(&mut self, target: &'static CStr) -> &mut Self {
        self.term_with_pair_name::<Rel>(target).not()
    }

    /// set term without enum
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::without`
    #[doc(alias = "filter_builder_i::without")]
    fn without_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
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
    fn without_pair<Rel: CachedComponentData, Target: CachedComponentData>(&mut self) -> &mut Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), Target::get_id(world))
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
            if !self.get_raw_term().is_null() {
                unsafe { ecs_term_is_initialized(self.get_raw_term()) }
            } else {
                true
            },
            FlecsErrorCode::InvalidOperation,
            "FilterBuilder::term() called without initializing term"
        );

        let term_index = *self.get_term_index();
        if term_index >= FLECS_TERM_DESC_MAX as i32 {
            let desc = self.get_desc_filter();
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
                self.get_desc_filter()
                    .terms
                    .as_mut_ptr()
                    .add(term_index as usize)
            };
            self.set_term(term_to_set);
        }
        *self.get_term_index() += 1;
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

        let term_index = *self.get_term_index();
        let prev_index = term_index;

        *self.get_term_index() = index - 1;
        self.term();

        *self.get_term_index() = prev_index;

        ecs_assert!(
            unsafe { ecs_term_is_initialized(self.get_raw_term()) },
            FlecsErrorCode::InvalidOperation,
            "term_at() called without initializing term"
        );

        self
    }

    fn arg(&mut self, index: i32) -> &mut Self {
        self.term_at(index)
    }

    /// set term with Component
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with<T: InOutType>(&mut self) -> &mut Self {
        self.term();

        unsafe {
            *self.get_raw_term() =
                Term::new(None, TermType::Id(T::Type::get_id(self.get_world()))).move_raw_term();
            (*self.get_raw_term()).inout = type_to_inout::<T>() as ecs_inout_kind_t;
        }
        self
    }

    fn write(&mut self) -> &mut Self {
        self.get_term().write_();
        self
    }

    fn write_type<T: InOutType>(&mut self) -> &mut Self {
        self.term_with::<T>();
        FilterBuilderImpl::write(self)
    }

    fn write_pair<Rel: InOutType, Target: InOutType>(&mut self) -> &mut Self {
        self.term_with_pair::<Rel::Type, Target::Type>();
        FilterBuilderImpl::write(self)
    }

    /// set term with Id
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_id(&mut self, id: IdT) -> &mut Self {
        self.term();
        *self.get_term_index() -= 1;
        let new_term: ecs_term_t = Term::new(None, TermType::Id(id)).move_raw_term();
        let term: &mut Term = self.get_term();
        let term_ptr: *mut TermT = term.term_ptr;
        unsafe { *term_ptr = new_term };
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
            *self.get_raw_term() = Term::default().select_first_name(name).move_raw_term();
        }
        self
    }

    /// set term with Pair Ids
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_pair_ids(&mut self, rel: IdT, target: IdT) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::new(None, TermType::Pair(rel, target)).move_raw_term();
        }
        self
    }

    /// set term with Pair Names
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_pair_names(&mut self, rel: &'static CStr, target: &'static CStr) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::default()
                .select_first_name(rel)
                .select_second_name(target)
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
    fn term_with_pair_id_name(&mut self, rel: IdT, target: &'static CStr) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::new(None, TermType::Id(rel))
                .select_second_name(target)
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
    fn term_with_pair_id<Rel: CachedComponentData>(&mut self, target: IdT) -> &mut Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), target)
    }

    /// set term with Pair
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_pair_name<Rel: CachedComponentData>(
        &mut self,
        target: &'static CStr,
    ) -> &mut Self {
        let world = self.get_world();
        self.term_with_id(Rel::get_id(world))
            .select_second_name(target)
    }

    /// set term with Pair
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_pair<Rel: CachedComponentData, Target: CachedComponentData>(
        &mut self,
    ) -> &mut Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), Target::get_id(world))
    }

    /// set term with enum
    ///
    /// # See also
    ///
    /// * C++ API: `filter_builder_i::term`
    #[doc(alias = "filter_builder_i::term")]
    fn term_with_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        let enum_id = T::get_id(self.get_world());
        let enum_field_id = value.get_entity_id_from_enum_field(self.get_world());
        self.term_with_pair_ids(enum_id, enum_field_id)
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
            *self.get_raw_term() = term.move_raw_term();
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
