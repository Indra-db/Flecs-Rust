use std::ffi::CString;

use crate::{core::utility::errors::FlecsErrorCode, ecs_assert};

use super::{
    c_types::{
        EntityT, Flags32T, IdT, InOutKind, OperKind, TermIdT, TermT, WorldT, ECS_CASCADE,
        ECS_FILTER, ECS_IS_ENTITY, ECS_IS_NAME, ECS_IS_VARIABLE, ECS_PARENT, ECS_SELF, ECS_UP,
    },
    component_registration::CachedComponentData,
};

/// A term identifier describes a single identifier in a term. Identifier
/// descriptions can reference entities by id, name or by variable, which means
/// the entity will be resolved when the term is evaluated.
///
/// A term is a single element of a query expression.
struct Term {
    term_id: *mut TermIdT,
    term: *mut TermT,
    world: *mut WorldT,
}

impl Term {
    pub fn new(world: *mut WorldT, term_ptr: *mut TermT) -> Self {
        let mut term = Self {
            world,
            term_id: std::ptr::null_mut(),
            term: std::ptr::null_mut(),
        };
        term.set_term(term_ptr);
        term
    }

    fn set_term(&mut self, term: *mut TermT) {
        self.term = term;
        if !term.is_null() {
            unsafe {
                (*self.term_id) = (*self.term).src; // default to subject
            }
        } else {
            unsafe {
                self.term_id = std::ptr::null_mut();
            }
        }
    }

    fn assert_term_id(&self) {
        ecs_assert!(
            self.term_id != std::ptr::null_mut(),
            FlecsErrorCode::InvalidParameter,
            "no active term (call .term() first"
        );
    }

    fn assert_term(&self) {
        ecs_assert!(
            self.term != std::ptr::null_mut(),
            FlecsErrorCode::InvalidParameter,
            "no active term (call .term() first"
        );
    }

    /// The self flag indicates the term identifier itself is used
    pub fn self_term(self) -> Self {
        self.assert_term_id();
        unsafe { (*self.term_id).flags |= ECS_SELF };
        self
    }

    /// The up flag indicates that the term identifier may be substituted by
    /// traversing a relationship upwards. For example: substitute the identifier
    /// with its parent by traversing the ChildOf relationship.
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The optional relationship to traverse.
    pub fn up_id(self, traverse_relationship: Option<EntityT>) -> Self {
        self.assert_term_id();
        unsafe { (*self.term_id).flags |= ECS_UP };
        if let Some(trav_rel) = traverse_relationship {
            unsafe { (*self.term_id).trav = trav_rel };
        }
        self
    }

    /// The up flag indicates that the term identifier may be substituted by
    /// traversing a relationship upwards. For example: substitute the identifier
    /// with its parent by traversing the ChildOf relationship.
    ///
    /// # Type Arguments
    ///
    /// * `TravRel` - The relationship to traverse.
    pub fn up<TravRel: CachedComponentData>(self) -> Self {
        self.assert_term_id();
        unsafe {
            (*self.term_id).flags |= ECS_UP;
            (*self.term_id).trav = TravRel::get_id(self.world)
        };
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for flecs::query
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The optional relationship to traverse.
    pub fn cascade_id(self, traverse_relationship: Option<EntityT>) -> Self {
        self.assert_term_id();
        unsafe { (*self.term_id).flags |= ECS_CASCADE };
        if let Some(trav_rel) = traverse_relationship {
            unsafe { (*self.term_id).trav = trav_rel };
        }
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for flecs::query
    ///
    /// # Type Arguments
    ///
    /// * `TravRel` - The relationship to traverse.
    pub fn cascade<TravRel: CachedComponentData>(self) -> Self {
        self.assert_term_id();
        unsafe {
            (*self.term_id).flags |= ECS_CASCADE;
            (*self.term_id).trav = TravRel::get_id(self.world)
        };
        self
    }

    /// the parent flag is short for up (flecs::ChildOf)
    pub fn parent(self) -> Self {
        self.assert_term_id();
        unsafe {
            (*self.term_id).flags |= ECS_PARENT;
        };
        self
    }

    /// Specify relationship to traverse, and flags to indicate direction
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The relationship to traverse.
    /// * `flags` - The direction to traverse.
    pub fn trav(self, traverse_relationship: EntityT, flags: Flags32T) -> Self {
        self.assert_term_id();
        unsafe {
            (*self.term_id).trav = traverse_relationship;
            (*self.term_id).flags |= flags;
        };
        self
    }

    /// Specify value of identifier by id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    pub fn id(self, id: EntityT) -> Self {
        self.assert_term_id();
        unsafe {
            (*self.term_id).id = id;
        };
        self
    }

    /// Specify value of identifier by id, same as id()
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    pub fn term(self, id: IdT) -> Self {
        self.id(id)
    }

    /// Specify value of identifier by id. Amost the same as id(entity), but this
    /// operation explicitly sets the flecs::IsEntity flag. This forces the id to
    /// be interpreted as entity, whereas not setting the flag would implicitly
    /// convert ids for builtin variables such as flecs::This to a variable.
    ///
    /// This function can also be used to disambiguate id(0), which would match
    /// both id(EntityT) and id(&str).
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    pub fn entity(self, id: EntityT) -> Self {
        self.assert_term_id();
        unsafe {
            (*self.term_id).flags |= ECS_IS_ENTITY;
            (*self.term_id).id = id;
        };
        self
    }

    /// Specify value of identifier by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    pub fn name(self, name: &str) -> Self {
        self.assert_term_id();
        let c_name = CString::new(name).unwrap();
        unsafe {
            let leak_name = CString::into_raw(c_name);
            (*self.term_id).name = leak_name as *mut i8;
            (*self.term_id).flags |= ECS_IS_NAME;
        };
        self
    }

    /// Specify identifier is a variable (resolved at query evaluation time)
    ///
    /// # Arguments
    ///
    /// * `var_name` - The name of the variable.
    pub fn var(self, var_name: &str) -> Self {
        self.assert_term_id();
        let c_name = CString::new(var_name).unwrap();
        unsafe {
            let leak_name = CString::into_raw(c_name);
            (*self.term_id).flags |= ECS_IS_VARIABLE;
            (*self.term_id).name = leak_name as *mut i8;
        };
        self
    }

    /// Override term id flags
    ///
    /// # Arguments
    ///
    /// * `flags` - The flags to set.
    pub fn flags(self, flags: Flags32T) -> Self {
        self.assert_term_id();
        unsafe {
            (*self.term_id).flags = flags;
        };
        self
    }

    /// Call prior to setting values for src identifier
    pub fn setup_src(self) -> Self {
        self.assert_term();
        unsafe {
            (*self.term_id) = (*self.term).src;
        };
        self
    }

    /// Call prior to setting values for first identifier. This is either the
    /// component identifier, or first element of a pair (in case second is
    /// populated as well).
    pub fn setup_first(self) -> Self {
        self.assert_term();
        unsafe {
            (*self.term_id) = (*self.term).first;
        };
        self
    }

    /// Call prior to setting values for second identifier. This is the second
    /// element of a pair. Requires that first() is populated as well.
    pub fn setup_second(self) -> Self {
        self.assert_term();
        unsafe {
            (*self.term_id) = (*self.term).second;
        };
        self
    }

    /// Select src identifier, initialize it with entity id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    pub fn src_id(self, id: EntityT) -> Self {
        self.setup_src();
        self.id(id)
    }

    /// Select src identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    pub fn src<T: CachedComponentData>(self) -> Self {
        self.src_id(T::get_id(self.world))
    }

    /// Select src identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    pub fn src_name(self, name: &str) -> Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_src();
        if name.starts_with('$') {
            self.var(&name[1..])
        } else {
            self.name(name)
        }
    }

    /// Select first identifier, initialize it with entity id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    pub fn first_id(self, id: EntityT) -> Self {
        self.setup_first();
        self.id(id)
    }

    /// Select first identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    pub fn first<T: CachedComponentData>(self) -> Self {
        self.first_id(T::get_id(self.world))
    }

    /// Select first identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    pub fn first_name(self, name: &str) -> Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_first();
        if name.starts_with('$') {
            self.var(&name[1..])
        } else {
            self.name(name)
        }
    }

    /// Select second identifier, initialize it with entity id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    pub fn second_id(self, id: EntityT) -> Self {
        self.setup_second();
        self.id(id)
    }

    /// Select second identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    pub fn second<T: CachedComponentData>(self) -> Self {
        self.second_id(T::get_id(self.world))
    }

    /// Select second identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    pub fn second_name(self, name: &str) -> Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_second();
        if name.starts_with('$') {
            self.var(&name[1..])
        } else {
            self.name(name)
        }
    }

    /// Set role of term
    ///
    /// # Arguments
    ///
    /// * `role` - The role to set.
    pub fn role(self, role: IdT) -> Self {
        self.assert_term();
        unsafe { (*self.term).id_flags = role };
        self
    }

    /// Set read=write access of term
    ///
    /// # Arguments
    ///
    /// * `inout` - The inout to set.
    pub fn set_inout(self, inout: InOutKind) -> Self {
        self.assert_term();
        unsafe { (*self.term).inout = inout as i32 };
        self
    }

    /// Set read/write access for stage. Use this when a system reads or writes
    /// components other than the ones provided by the query. This information
    /// can be used by schedulers to insert sync/merge points between systems
    /// where deferred operations are flushed.
    ///
    /// Setting this is optional. If not set, the value of the accessed component
    /// may be out of sync for at most one frame.
    ///
    /// # Arguments
    ///
    /// * 'inout' - The inout to set.
    pub fn inout_stage(self, inout: InOutKind) -> Self {
        self.assert_term();
        self.set_inout(inout);
        if unsafe { (*self.term).inout != OperKind::Not as i32 } {
            self.setup_src().entity(0);
        }
        self
    }

    /// Short for inout_stage(flecs::Out).
    ///  Use when system uses add, remove or set.
    ///
    pub fn write(self) -> Self {
        self.inout_stage(InOutKind::Out)
    }

    /// Short for inout_stage(flecs::In).
    /// Use when system uses get
    pub fn read(self) -> Self {
        self.inout_stage(InOutKind::In)
    }

    /// Short for inout_stage(flecs::InOut).
    /// Use when system uses get_mut
    pub fn read_write(self) -> Self {
        self.inout_stage(InOutKind::InOut)
    }

    /// short for inout(flecs::In)
    pub fn in_(self) -> Self {
        self.set_inout(InOutKind::In)
    }

    /// short for inout(flecs::Out)
    pub fn out(self) -> Self {
        self.set_inout(InOutKind::Out)
    }

    /// short for inout(flecs::InOut)
    pub fn inout(self) -> Self {
        self.set_inout(InOutKind::InOut)
    }

    /// short for inout(flecs::InOutNone)
    pub fn inout_none(self) -> Self {
        self.set_inout(InOutKind::InOutNone)
    }

    /// set operator of term
    ///
    /// # Arguments
    ///
    /// * `oper` - The operator to set.
    pub fn oper(self, oper: OperKind) -> Self {
        self.assert_term_id();
        unsafe { (*self.term).oper = oper as i32 };
        self
    }

    /// short for oper(flecs::And)
    pub fn and(self) -> Self {
        self.oper(OperKind::And)
    }

    /// short for oper(flecs::Or)
    pub fn or(self) -> Self {
        self.oper(OperKind::Or)
    }

    /// short for oper(flecs::Not)
    pub fn not(self) -> Self {
        self.oper(OperKind::Not)
    }

    /// short for oper(flecs::Optional)
    pub fn optional(self) -> Self {
        self.oper(OperKind::Optional)
    }

    /// short for oper(flecs::AndFrom)
    pub fn and_from(self) -> Self {
        self.oper(OperKind::AndFrom)
    }

    /// short for oper(flecs::OrFrom)
    pub fn or_from(self) -> Self {
        self.oper(OperKind::OrFrom)
    }

    /// short for oper(flecs::NotFrom)
    pub fn not_from(self) -> Self {
        self.oper(OperKind::NotFrom)
    }

    /// Match singleton
    pub fn singleton(mut self) -> Self {
        self.assert_term();

        ecs_assert!(
            unsafe { (*self.term).id != 0 || (*self.term).first.id != 0 },
            FlecsErrorCode::InvalidParameter,
            "no component specified for singleton"
        );

        let sid = unsafe {
            if (*self.term).id != 0 {
                (*self.term).id
            } else {
                (*self.term).first.id
            }
        };

        ecs_assert!(sid != 0, FlecsErrorCode::InvalidParameter, "invalid id");

        unsafe { (*self.term).src.id = sid };

        self
    }

    /// Filter terms are not triggered on by observers
    pub fn filter(self) -> Self {
        unsafe { (*self.term).src.flags |= ECS_FILTER };
        self
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        unsafe {
            if !(*self.term_id).name.is_null() {
                let _ = CString::from_raw((*self.term_id).name);
            }
        }
    }
}
