use std::default;

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
    filter::{Filter, Filterable},
    iterable::Iterable,
    term::{Term, TermBuilder},
    utility::{functions::type_to_inout, traits::InOutType},
};

pub struct FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    desc: ecs_filter_desc_t,
    expr_count: i32,
    term_index: i32,
    term: Term,
    world: *mut WorldT,
    next_term_index: i32,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    pub fn new(world: *mut WorldT) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            expr_count: 0,
            term_index: 0,
            term: Term::new_only_world(world),
            world,
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };
        T::populate(&mut obj);
        //T::register_ids_descriptor(world, &mut obj.desc);
        obj
    }

    pub fn new_named(world: *mut WorldT, name: &str) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            expr_count: 0,
            term_index: 0,
            term: Term::default(),
            world,
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };
        T::populate(&mut obj);
        //T::register_ids_descriptor(world, &mut obj.desc);
        let mut desc = ecs_entity_desc_t::default();
        desc.name = std::ffi::CString::new(name).unwrap().into_raw();
        desc.sep = SEPARATOR.as_ptr();
        desc.root_sep = SEPARATOR.as_ptr();
        obj.desc.entity = unsafe { ecs_entity_init(world, &mut desc) };
        obj
    }

    pub fn new_with_desc(
        world: *mut WorldT,
        desc: *mut ecs_filter_desc_t,
        term_index: i32,
    ) -> Self {
        Self {
            desc: unsafe { *desc },
            expr_count: 0,
            term_index,
            term: Term::default(),
            world,
            next_term_index: 0,
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

impl<'a, T> Filterable for FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    fn get_world(&self) -> *mut WorldT {
        self.world
    }

    fn current_term(&mut self) -> &mut TermT {
        self.current_term()
    }

    fn next_term(&mut self) {
        self.next_term()
    }
}

impl<'a, T> FilterBuilderImpl for FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    fn get_desc(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.desc
    }

    fn get_expr_count(&mut self) -> &mut i32 {
        &mut self.expr_count
    }

    fn get_term_index(&mut self) -> &mut i32 {
        &mut self.next_term_index
    }
}

impl<'a, T> TermBuilder for FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    fn get_world(&self) -> *mut super::c_types::WorldT {
        self.world
    }

    fn get_term(&mut self) -> &mut Term {
        &mut self.term
    }

    fn get_raw_term(&mut self) -> *mut super::c_types::TermT {
        self.term.term_ptr
    }

    fn get_term_id(&mut self) -> *mut super::c_types::TermIdT {
        self.term.term_id
    }
}

impl<'a, T> Builder for FilterBuilder<'a, T>
where
    T: Iterable<'a>,
{
    type BuiltType = Filter<'a, T>;

    fn build(mut self) -> Self::BuiltType {
        Filter::<'a, T>::new_from_desc(TermBuilder::get_world(&self), &mut self.desc as *mut _)
    }
}

pub trait FilterBuilderImpl: TermBuilder {
    fn get_desc(&mut self) -> &mut ecs_filter_desc_t;

    fn get_expr_count(&mut self) -> &mut i32;

    fn get_term_index(&mut self) -> &mut i32;

    /// set itself to be instanced
    ///
    /// # C++ API Equivalent
    ///
    /// `filter_builder_i::instanced`
    fn instanced(mut self) -> Self {
        self.get_desc().instanced = true;
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
    fn filter_flags(mut self, flags: ecs_flags32_t) -> Self {
        self.get_desc().flags |= flags;
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
    fn expr(mut self, expr: &str) -> Self {
        ecs_assert!(
            *self.get_expr_count() == 0,
            FlecsErrorCode::InvalidOperation,
            "filter_builder::expr() called more than once"
        );

        self.get_desc().expr = std::ffi::CString::new(expr).unwrap().into_raw();
        *self.get_expr_count() += 1;
        self
    }

    fn with(self, with: With) -> Self {
        match with {
            With::Id(id) => self.term_with_id(id),
            With::Name(name) => self.term_with_name(name),
            With::PairIds(rel, target) => self.term_with_pair_ids(rel, target),
            With::PairNames(rel, target) => self.term_with_pair_names(rel, target),
            With::PairIdName(rel, target) => self.term_with_pair_id_name(rel, target),
            With::Term(term) => self.term_with_term(term),
        }
    }

    fn with_pair_id<Rel: CachedComponentData>(self, target: IdT) -> Self {
        self.term_with_pair_id::<Rel>(target)
    }

    fn with_pair_name<Rel: CachedComponentData>(self, target: &str) -> Self {
        self.term_with_pair_name::<Rel>(target)
    }

    fn with_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        self,
        value: T,
    ) -> Self {
        self.term_with_enum(value)
    }

    fn with_pair<Rel: CachedComponentData, Target: CachedComponentData>(self) -> Self {
        self.term_with_pair::<Rel, Target>()
    }

    fn with_type<T: InOutType>(self) -> Self {
        self.term_with::<T>()
    }

    fn without(self, without: Without) -> Self {
        match without {
            Without::Id(id) => self.term_with_id(id).not(),
            Without::Name(name) => self.term_with_name(name).not(),
            Without::PairIds(rel, target) => self.term_with_pair_ids(rel, target).not(),
            Without::PairNames(rel, target) => self.term_with_pair_names(rel, target).not(),
            Without::PairIdName(rel, target) => self.term_with_pair_id_name(rel, target).not(),
            Without::Term(term) => self.term_with_term(term).not(),
        }
    }

    fn without_pair_id<Rel: CachedComponentData>(self, target: IdT) -> Self {
        self.term_with_pair_id::<Rel>(target).not()
    }

    fn without_pair_name<Rel: CachedComponentData>(self, target: &str) -> Self {
        self.term_with_pair_name::<Rel>(target).not()
    }

    fn without_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        self,
        value: T,
    ) -> Self {
        self.term_with_enum(value).not()
    }

    fn without_pair<Rel: CachedComponentData, Target: CachedComponentData>(self) -> Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), Target::get_id(world))
            .not()
    }

    fn without_type<T: InOutType>(self) -> Self {
        self.term_with::<T>().not()
    }

    fn term(&mut self) {
        //ecs_assert!(
        //    unsafe { ecs_term_is_initialized(self.get_raw_term()) },
        //    FlecsErrorCode::InvalidOperation,
        //    "FilterBuilder::term() called without initializing term"
        //);

        let term_index = *self.get_term_index();
        if term_index >= FLECS_TERM_DESC_MAX as i32 {
            let desc = self.get_desc();
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
            let term_to_set =
                unsafe { self.get_desc().terms.as_mut_ptr().add(term_index as usize) };
            self.set_term(term_to_set);
        }
        *self.get_term_index() += 1;
    }

    fn term_at(mut self, index: i32) -> Self {
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

    fn arg(self, index: i32) -> Self {
        self.term_at(index)
    }

    fn term_with<T: InOutType>(mut self) -> Self {
        self.term();
        unsafe {
            *self.get_raw_term() =
                Term::new_only_id(T::Type::get_id(self.get_world())).move_raw_term();
            (*self.get_raw_term()).inout = type_to_inout::<T>() as i32;
        }
        self
    }

    fn term_with_id(mut self, id: IdT) -> Self {
        self.term();
        let new_term: ecs_term_t = Term::new_only_id(id).move_raw_term();
        let term: &mut Term = self.get_term();
        let term_ptr: *mut TermT = term.term_ptr;
        unsafe { *term_ptr = new_term };
        //(*to_set_term).id = 517;
        self
    }

    fn term_with_name(mut self, name: &str) -> Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::default().first_name(name).move_raw_term();
        }
        self
    }

    fn term_with_pair_ids(mut self, rel: IdT, target: IdT) -> Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::new_only_rel_target(rel, target).move_raw_term();
        }
        self
    }

    fn term_with_pair_names(mut self, rel: &str, target: &str) -> Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::default()
                .first_name(rel)
                .second_name(target)
                .move_raw_term();
        }
        self
    }

    fn term_with_pair_id_name(mut self, rel: IdT, target: &str) -> Self {
        self.term();
        unsafe {
            *self.get_raw_term() = Term::new_only_id(rel).second_name(target).move_raw_term();
        }
        self
    }

    fn term_with_pair_id<Rel: CachedComponentData>(self, target: IdT) -> Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), target)
    }

    fn term_with_pair_name<Rel: CachedComponentData>(self, target: &str) -> Self {
        let world = self.get_world();
        self.term_with_id(Rel::get_id(world)).second_name(target)
    }

    fn term_with_pair<Rel: CachedComponentData, Target: CachedComponentData>(self) -> Self {
        let world = self.get_world();
        self.term_with_pair_ids(Rel::get_id(world), Target::get_id(world))
    }

    fn term_with_enum<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        self,
        value: T,
    ) -> Self {
        let enum_id = T::get_id(self.get_world());
        let enum_field_id = value.get_entity_id_from_enum_field(self.get_world());
        self.term_with_pair_ids(enum_id, enum_field_id)
    }

    fn term_with_term(mut self, mut term: Term) -> Self {
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
