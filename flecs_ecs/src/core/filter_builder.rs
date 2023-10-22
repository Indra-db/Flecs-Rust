use libc::{c_void, memcpy, memset};

use crate::{
    core::{
        c_binding::bindings::{
            ecs_os_api, ecs_term_is_initialized, ecs_term_t, FLECS_TERM_DESC_MAX,
        },
        utility::errors::FlecsErrorCode,
    },
    ecs_assert,
};

use super::{
    builder::Builder,
    c_binding::bindings::{ecs_entity_desc_t, ecs_entity_init, ecs_filter_desc_t, ecs_flags32_t},
    c_types::{IdT, TermT, WorldT, SEPARATOR},
    component_registration::{CachedComponentData, ComponentType, Enum},
    enum_type::CachedEnumData,
    filter::Filter,
    iterable::{Filterable, Iterable},
    term::{Term, TermBuilder, With as TermWith},
    utility::{functions::type_to_inout, traits::InOutType},
    world::World,
};

pub struct FilterBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    pub desc: ecs_filter_desc_t,
    expr_count: i32,
    term: Term,
    pub world: &'w World,
    next_term_index: i32,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, 'w, T> FilterBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    pub fn new(world: &'w World) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            expr_count: 0,
            term: Term::new_world_only(world),
            world,
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };
        T::populate(&mut obj);
        obj
    }

    pub fn new_named(world: &'w World, name: &str) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            expr_count: 0,
            term: Term::default(),
            world,
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };
        T::populate(&mut obj);

        let entity_desc = ecs_entity_desc_t {
            name: std::ffi::CString::new(name).unwrap().into_raw(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { ecs_entity_init(world.raw_world, &entity_desc) };
        obj
    }

    pub fn new_with_desc(world: &'w World, desc: &mut ecs_filter_desc_t, term_index: i32) -> Self {
        Self {
            desc: *desc,
            expr_count: 0,
            term: Term::default(),
            world,
            next_term_index: term_index,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn current_term(&mut self) -> &mut TermT {
        &mut self.desc.terms[self.next_term_index as usize]
    }

    pub fn next_term(&mut self) {
        self.next_term_index += 1;
    }
}

impl<'a, 'w, T> Filterable for FilterBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    fn get_world(&self) -> *mut WorldT {
        self.world.raw_world
    }

    fn current_term(&mut self) -> &mut TermT {
        self.current_term()
    }

    fn next_term(&mut self) {
        self.next_term()
    }
}

impl<'a, 'w, T> FilterBuilderImpl for FilterBuilder<'a, 'w, T>
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

impl<'a, 'w, T> TermBuilder for FilterBuilder<'a, 'w, T>
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

impl<'a, 'w, T> Builder for FilterBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    type BuiltType = Filter<'a, 'w, T>;

    #[inline]
    fn build(&mut self) -> Self::BuiltType {
        Filter::<'a, 'w, T>::new_from_desc(self.world, &mut self.desc as *mut _)
    }
}

pub trait FilterBuilderImpl: TermBuilder {
    fn get_desc_filter(&mut self) -> &mut ecs_filter_desc_t;

    fn get_expr_count(&mut self) -> &mut i32;

    fn get_term_index(&mut self) -> &mut i32;

    /// set itself to be instanced
    ///
    /// # C++ API Equivalent
    ///
    /// `filter_builder_i::instanced`
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
    /// # C++ API Equivalent
    ///
    /// `filter_builder_i::filter_flags`
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
    /// # C++ API Equivalent
    ///
    /// `filter_builder_i::expr`
    fn expr(&mut self, expr: &str) -> &mut Self {
        ecs_assert!(
            *self.get_expr_count() == 0,
            FlecsErrorCode::InvalidOperation,
            "filter_builder::expr() called more than once"
        );

        self.get_desc_filter().expr = std::ffi::CString::new(expr).unwrap().into_raw();
        *self.get_expr_count() += 1;
        self
    }

    fn with(&mut self, with: With) -> &mut Self {
        match with {
            With::Id(id) => self.term_with_id(id),
            With::Name(name) => self.term_with_name(name),
            With::PairIds(rel, target) => self.term_with_pair_ids(rel, target),
            With::PairNames(rel, target) => self.term_with_pair_names(rel, target),
            With::PairIdName(rel, target) => self.term_with_pair_id_name(rel, target),
            With::Term(term) => self.term_with_term(term),
        }
    }

    fn with_pair_id<Rel: CachedComponentData>(&mut self, target: IdT) -> &mut Self {
        self.term_with_pair_id::<Rel>(target)
    }

    fn with_pair_name<Rel: CachedComponentData>(&mut self, target: &str) -> &mut Self {
        self.term_with_pair_name::<Rel>(target)
    }

    fn with_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.term_with_enum(value)
    }

    fn with_pair<Rel: CachedComponentData, Target: CachedComponentData>(&mut self) -> &mut Self {
        self.term_with_pair::<Rel, Target>()
    }

    fn with_type<T: InOutType>(&mut self) -> &mut Self {
        self.term_with::<T>()
    }

    fn without(&mut self, without: Without) -> &mut Self {
        match without {
            Without::Id(id) => self.term_with_id(id).not(),
            Without::Name(name) => self.term_with_name(name).not(),
            Without::PairIds(rel, target) => self.term_with_pair_ids(rel, target).not(),
            Without::PairNames(rel, target) => self.term_with_pair_names(rel, target).not(),
            Without::PairIdName(rel, target) => self.term_with_pair_id_name(rel, target).not(),
            Without::Term(term) => self.term_with_term(term).not(),
        }
    }

    fn without_pair_id<Rel: CachedComponentData>(&mut self, target: IdT) -> &mut Self {
        self.term_with_pair_id::<Rel>(target).not()
    }

    fn without_pair_name<Rel: CachedComponentData>(&mut self, target: &str) -> &mut Self {
        self.term_with_pair_name::<Rel>(target).not()
    }

    fn without_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.term_with_enum(value).not()
    }

    fn without_pair<Rel: CachedComponentData, Target: CachedComponentData>(&mut self) -> &mut Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), Target::get_id(world))
            .not()
    }

    fn without_type<T: InOutType>(&mut self) -> &mut Self {
        self.term_with::<T>().not()
    }

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
                    memcpy(
                        desc.terms_buffer as *mut _,
                        desc.terms.as_ptr() as *const c_void,
                        size_term * term_index as usize,
                    );
                    memset(
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

    fn term_with<T: InOutType>(&mut self) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() =
                Term::new(None, TermWith::Id(T::Type::get_id(self.get_world()))).move_raw_term();
            (*self.get_raw_term()).inout = type_to_inout::<T>() as i32;
        }
        self
    }

    fn term_with_id(&mut self, id: IdT) -> &mut Self {
        self.term();
        let new_term: ecs_term_t = Term::new(None, TermWith::Id(id)).move_raw_term();
        let term: &mut Term = self.get_term();
        let term_ptr: *mut TermT = term.term_ptr;
        unsafe { *term_ptr = new_term };
        self
    }

    fn term_with_name(&mut self, name: &str) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::default().first_name(name).move_raw_term();
        }
        self
    }

    fn term_with_pair_ids(&mut self, rel: IdT, target: IdT) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::new(None, TermWith::Pair(rel, target)).move_raw_term();
        }
        self
    }

    fn term_with_pair_names(&mut self, rel: &str, target: &str) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::default()
                .first_name(rel)
                .second_name(target)
                .move_raw_term();
        }
        self
    }

    fn term_with_pair_id_name(&mut self, rel: IdT, target: &str) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::new(None, TermWith::Id(rel))
                .second_name(target)
                .move_raw_term();
        }
        self
    }

    fn term_with_pair_id<Rel: CachedComponentData>(&mut self, target: IdT) -> &mut Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), target)
    }

    fn term_with_pair_name<Rel: CachedComponentData>(&mut self, target: &str) -> &mut Self {
        let world = self.get_world();
        self.term_with_id(Rel::get_id(world)).second_name(target)
    }

    fn term_with_pair<Rel: CachedComponentData, Target: CachedComponentData>(
        &mut self,
    ) -> &mut Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), Target::get_id(world))
    }

    fn term_with_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        let enum_id = T::get_id(self.get_world());
        let enum_field_id = value.get_entity_id_from_enum_field(self.get_world());
        self.term_with_pair_ids(enum_id, enum_field_id)
    }

    fn term_with_term(&mut self, mut term: Term) -> &mut Self {
        self.term();
        unsafe {
            *self.get_raw_term() = term.move_raw_term();
        }
        self
    }
}

pub enum With {
    Id(IdT),
    Name(&'static str),
    PairIds(IdT, IdT),
    PairNames(&'static str, &'static str),
    PairIdName(IdT, &'static str),
    Term(Term),
}

pub enum Without {
    Id(IdT),
    Name(&'static str),
    PairIds(IdT, IdT),
    PairNames(&'static str, &'static str),
    PairIdName(IdT, &'static str),
    Term(Term),
}
