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
    c_binding::bindings::{ecs_filter_desc_t, ecs_flags32_t},
    c_types::IdT,
    component_registration::{CachedComponentData, ComponentType, Enum},
    enum_type::CachedEnumData,
    term::{Term, TermBuilder},
    utility::{functions::type_to_inout, traits::InOutType},
};

trait FilterBuilder: TermBuilder {
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

    fn term(mut self) -> Self {
        ecs_assert!(
            unsafe { ecs_term_is_initialized(self.get_term()) },
            FlecsErrorCode::InvalidOperation,
            "FilterBuilder::term() called without initializing term"
        );

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
        self
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

        self = self.term();

        *self.get_term_index() = prev_index;

        ecs_assert!(
            unsafe { ecs_term_is_initialized(self.get_term()) },
            FlecsErrorCode::InvalidOperation,
            "term_at() called without initializing term"
        );

        self
    }

    fn arg(self, index: i32) -> Self {
        self.term_at(index)
    }

    fn term_with<T: InOutType>(mut self) -> Self {
        self = self.term();
        *self.get_term() = Term::new_only_id(T::Type::get_id(self.get_world())).move_raw_term();
        self.get_term().inout = type_to_inout::<T>() as i32;
        self
    }

    fn term_with_id(mut self, id: IdT) -> Self {
        self = self.term();
        *self.get_term() = Term::new_only_id(id).move_raw_term();
        self
    }

    fn term_with_name(mut self, name: &str) -> Self {
        self = self.term();
        *self.get_term() = Term::default().first_name(name).move_raw_term();
        self
    }

    fn term_with_pair_ids(mut self, rel: IdT, target: IdT) -> Self {
        self = self.term();
        *self.get_term() = Term::new_only_rel_target(rel, target).move_raw_term();
        self
    }

    fn term_with_pair_names(mut self, rel: &str, target: &str) -> Self {
        self = self.term();
        *self.get_term() = Term::default()
            .first_name(rel)
            .second_name(target)
            .move_raw_term();
        self
    }

    fn term_with_pair_id_name(mut self, rel: IdT, target: &str) -> Self {
        self = self.term();
        *self.get_term() = Term::new_only_id(rel).second_name(target).move_raw_term();
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

    fn term_with_pair<Rel: CachedComponentData, Target: CachedComponentData>(mut self) -> Self {
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
        self = self.term();
        *self.get_term() = term.move_raw_term();
        self
    }
}
