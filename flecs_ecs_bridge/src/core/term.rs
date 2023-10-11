use std::ffi::CString;

use crate::{core::utility::errors::FlecsErrorCode, ecs_assert};

use super::{
    c_binding::bindings::{
        ecs_term_copy, ecs_term_finalize, ecs_term_fini, ecs_term_is_initialized, ecs_term_move,
        ECS_ID_FLAGS_MASK,
    },
    c_types::{
        EntityT, Flags32T, IdT, InOutKind, OperKind, TermIdT, TermT, WorldT, ECS_CASCADE,
        ECS_FILTER, ECS_IS_ENTITY, ECS_IS_NAME, ECS_IS_VARIABLE, ECS_PARENT, ECS_SELF, ECS_UP,
    },
    component_registration::CachedComponentData,
    entity::Entity,
    id::Id,
    utility::functions::ecs_pair,
};

/// Struct that describes a term identifier.
///
/// A term is a single element of a query expression.
///
/// A term identifier describes a single identifier in a term. Identifier
/// descriptions can reference entities by id, name or by variable, which means
/// the entity will be resolved when the term is evaluated.
pub struct Term {
    pub term_id: TermIdT,
    pub term: TermT,
    pub world: *mut WorldT,
}

impl Default for Term {
    fn default() -> Self {
        let mut obj = Self {
            term_id: Default::default(),
            term: Default::default(),
            world: std::ptr::null_mut(),
        };
        obj.term.move_ = true;
        obj
    }
}

/// this is for copying the term
impl Clone for Term {
    fn clone(&self) -> Self {
        let mut obj = Self {
            term_id: Default::default(),
            term: Default::default(),
            world: self.world,
        };
        obj.term = unsafe { ecs_term_copy(&self.term) };
        obj.term_id = obj.term.src;
        obj
    }
}

impl Term {
    pub fn new(world: *mut WorldT, term: TermT) -> Self {
        let mut obj = Self {
            world,
            term_id: term.src, // default to subject
            term,
        };
        obj.term.move_ = false;
        obj
    }

    pub fn new_only_world(world: *mut WorldT) -> Self {
        let mut obj = Self {
            world,
            term_id: Default::default(),
            term: Default::default(),
        };
        obj.term_id = obj.term.src;
        obj.term.move_ = true;
        obj
    }

    pub fn new_id(world: *mut WorldT, id: IdT) -> Self {
        let mut obj = Self {
            world,
            term_id: Default::default(),
            term: Default::default(),
        };
        if id & ECS_ID_FLAGS_MASK as u64 != 0 {
            obj.term.id = id;
        } else {
            obj.term.first.id = id;
        }
        obj.term.move_ = false;
        obj.term_id = obj.term.src;
        obj
    }

    pub fn new_only_id(id: IdT) -> Self {
        let mut obj = Self {
            world: std::ptr::null_mut(),
            term_id: Default::default(),
            term: Default::default(),
        };
        if id & ECS_ID_FLAGS_MASK as u64 != 0 {
            obj.term.id = id;
        } else {
            obj.term.first.id = id;
        }
        obj.term.move_ = false;
        obj.term_id = obj.term.src;
        obj
    }

    pub fn new_rel_target(world: *mut WorldT, rel: EntityT, target: EntityT) -> Self {
        let mut obj = Self {
            world,
            term_id: Default::default(),
            term: Default::default(),
        };
        obj.term.id = ecs_pair(rel, target);
        obj.term.move_ = false;
        obj.term_id = obj.term.src;
        obj
    }

    pub fn new_only_rel_target(rel: EntityT, target: EntityT) -> Self {
        let mut obj = Self {
            world: std::ptr::null_mut(),
            term_id: Default::default(),
            term: Default::default(),
        };
        obj.term.id = ecs_pair(rel, target);
        obj.term.move_ = true;
        obj.term_id = obj.term.src;
        obj
    }

    pub fn new_from_type<T: CachedComponentData>(world: *mut WorldT) -> Self {
        Self::new_id(world, T::get_id(world))
    }

    pub fn new_only_from_type<T: CachedComponentData>() -> Self {
        Self::new_only_id(T::get_id(std::ptr::null_mut()))
    }

    pub fn new_from_pair_type<T: CachedComponentData, U: CachedComponentData>(
        world: *mut WorldT,
    ) -> Self {
        Self::new_rel_target(world, T::get_id(world), U::get_id(world))
    }

    pub fn new_only_from_pair_type<T: CachedComponentData, U: CachedComponentData>() -> Self {
        Self::new_only_rel_target(
            T::get_id(std::ptr::null_mut()),
            U::get_id(std::ptr::null_mut()),
        )
    }

    /// This is how you should move a term, not the default rust way
    pub fn move_term(&mut self) -> Self {
        let mut obj = Self {
            world: self.world,
            term_id: Default::default(),
            term: Default::default(),
        };
        obj.term = unsafe { ecs_term_move(&mut self.term) };
        self.reset();
        obj.term_id = obj.term.src;
        obj
    }

    pub fn reset(&mut self) {
        // we don't for certain if this causes any side effects not using the nullptr and just using the default value.
        // if it does we can use Option.
        self.term = Default::default();
        self.term_id = Default::default();
    }

    pub fn finalize(&mut self) -> i32 {
        unsafe { ecs_term_finalize(self.world, &mut self.term) }
    }

    pub fn is_set(&mut self) -> bool {
        unsafe { ecs_term_is_initialized(&self.term) }
    }

    pub fn get_id(&self) -> Id {
        Id::new_from_existing(self.world, self.term.id)
    }

    pub fn get_inout(&self) -> InOutKind {
        self.term.inout.into()
    }

    pub fn get_oper(&self) -> OperKind {
        self.term.oper.into()
    }

    pub fn get_src(&self) -> Entity {
        Entity::new_from_existing(self.world, self.term.src.id)
    }

    pub fn get_first(&self) -> Entity {
        Entity::new_from_existing(self.world, self.term.first.id)
    }

    pub fn get_second(&self) -> Entity {
        Entity::new_from_existing(self.world, self.term.second.id)
    }
}

/// Builder pattern functions
impl Term {
    fn assert_term_id(&self) {
        ecs_assert!(
            self.term_id.id != 0,
            FlecsErrorCode::InvalidParameter,
            "no active term (call .term() first"
        );
    }

    fn assert_term(&self) {
        ecs_assert!(
            self.term.id != 0,
            FlecsErrorCode::InvalidParameter,
            "no active term (call .term() first"
        );
    }

    /// The self flag indicates the term identifier itself is used
    pub fn self_term(mut self) -> Self {
        self.assert_term_id();
        self.term_id.flags |= ECS_SELF;
        self
    }

    /// The up flag indicates that the term identifier may be substituted by
    /// traversing a relationship upwards. For example: substitute the identifier
    /// with its parent by traversing the ChildOf relationship.
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The optional relationship to traverse.
    pub fn up_id(mut self, traverse_relationship: Option<EntityT>) -> Self {
        self.assert_term_id();
        self.term_id.flags |= ECS_UP;
        if let Some(trav_rel) = traverse_relationship {
            self.term_id.trav = trav_rel;
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
    pub fn up<TravRel: CachedComponentData>(mut self) -> Self {
        self.assert_term_id();
        self.term_id.flags |= ECS_UP;
        self.term_id.trav = TravRel::get_id(self.world);
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for flecs::query
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The optional relationship to traverse.
    pub fn cascade_id(mut self, traverse_relationship: Option<EntityT>) -> Self {
        self.assert_term_id();
        self.term_id.flags |= ECS_CASCADE;
        if let Some(trav_rel) = traverse_relationship {
            self.term_id.trav = trav_rel;
        }
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for flecs::query
    ///
    /// # Type Arguments
    ///
    /// * `TravRel` - The relationship to traverse.
    pub fn cascade<TravRel: CachedComponentData>(mut self) -> Self {
        self.assert_term_id();
        self.term_id.flags |= ECS_CASCADE;
        self.term_id.trav = TravRel::get_id(self.world);
        self
    }

    /// the parent flag is short for up (flecs::ChildOf)
    pub fn parent(mut self) -> Self {
        self.assert_term_id();
        self.term_id.flags |= ECS_PARENT;
        self
    }

    /// Specify relationship to traverse, and flags to indicate direction
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The relationship to traverse.
    /// * `flags` - The direction to traverse.
    pub fn trav(mut self, traverse_relationship: EntityT, flags: Flags32T) -> Self {
        self.assert_term_id();
        self.term_id.trav = traverse_relationship;
        self.term_id.flags |= flags;
        self
    }

    /// Specify value of identifier by id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    pub fn id(mut self, id: EntityT) -> Self {
        self.assert_term_id();
        self.term_id.id = id;
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
    pub fn entity(mut self, id: EntityT) -> Self {
        self.assert_term_id();
        self.term_id.flags |= ECS_IS_ENTITY;
        self.term_id.id = id;
        self
    }

    /// Specify value of identifier by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    pub fn name(mut self, name: &str) -> Self {
        self.assert_term_id();
        let c_name = CString::new(name).unwrap();
        let leak_name = CString::into_raw(c_name);
        self.term_id.name = leak_name;
        self.term_id.flags |= ECS_IS_NAME;
        self
    }

    /// Specify identifier is a variable (resolved at query evaluation time)
    ///
    /// # Arguments
    ///
    /// * `var_name` - The name of the variable.
    pub fn var(mut self, var_name: &str) -> Self {
        self.assert_term_id();
        let c_name = CString::new(var_name).unwrap();
        let leak_name = CString::into_raw(c_name);
        self.term_id.flags |= ECS_IS_VARIABLE;
        self.term_id.name = leak_name;
        self
    }

    /// Override term id flags
    ///
    /// # Arguments
    ///
    /// * `flags` - The flags to set.
    pub fn flags(mut self, flags: Flags32T) -> Self {
        self.assert_term_id();
        self.term_id.flags = flags;
        self
    }

    /// Call prior to setting values for src identifier
    pub fn setup_src(mut self) -> Self {
        self.assert_term();
        self.term_id = self.term.src;
        self
    }

    /// Call prior to setting values for first identifier. This is either the
    /// component identifier, or first element of a pair (in case second is
    /// populated as well).
    pub fn setup_first(mut self) -> Self {
        self.assert_term();
        self.term_id = self.term.first;
        self
    }

    /// Call prior to setting values for second identifier. This is the second
    /// element of a pair. Requires that first() is populated as well.
    pub fn setup_second(mut self) -> Self {
        self.assert_term();
        self.term_id = self.term.second;
        self
    }

    /// Select src identifier, initialize it with entity id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    pub fn src_id(self, id: EntityT) -> Self {
        self.setup_src().id(id)
    }

    /// Select src identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    pub fn src<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.src_id(T::get_id(world))
    }

    /// Select src identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    pub fn src_name(mut self, name: &str) -> Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self = self.setup_src();
        if let Some(stripped_name) = name.strip_prefix('$') {
            self.var(stripped_name)
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
        self.setup_first().id(id)
    }

    /// Select first identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    pub fn first<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.first_id(T::get_id(world))
    }

    /// Select first identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    pub fn first_name(mut self, name: &str) -> Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self = self.setup_first();
        if let Some(stripped_name) = name.strip_prefix('$') {
            self.var(stripped_name)
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
        self.setup_second().id(id)
    }

    /// Select second identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    pub fn second<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.second_id(T::get_id(world))
    }

    /// Select second identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    pub fn second_name(mut self, name: &str) -> Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self = self.setup_second();
        if let Some(stripped_name) = name.strip_prefix('$') {
            self.var(stripped_name)
        } else {
            self.name(name)
        }
    }

    /// Set role of term
    ///
    /// # Arguments
    ///
    /// * `role` - The role to set.
    pub fn role(mut self, role: IdT) -> Self {
        self.assert_term();
        self.term.id_flags = role;
        self
    }

    /// Set read=write access of term
    ///
    /// # Arguments
    ///
    /// * `inout` - The inout to set.
    pub fn set_inout(mut self, inout: InOutKind) -> Self {
        self.assert_term();
        self.term.inout = inout as i32;
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
    pub fn inout_stage(mut self, inout: InOutKind) -> Self {
        self.assert_term();
        self = self.set_inout(inout);
        if self.term.inout != OperKind::Not as i32 {
            self = self.setup_src().entity(0);
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
    pub fn oper(mut self, oper: OperKind) -> Self {
        self.assert_term_id();
        self.term.oper = oper as i32;
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
    #[allow(clippy::should_implement_trait)]
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
            self.term.id != 0 || self.term.first.id != 0,
            FlecsErrorCode::InvalidParameter,
            "no component specified for singleton"
        );

        let sid = if self.term.id != 0 {
            self.term.id
        } else {
            self.term.first.id
        };

        ecs_assert!(sid != 0, FlecsErrorCode::InvalidParameter, "invalid id");
        self.term.src.id = sid;
        self
    }

    /// Filter terms are not triggered on by observers
    pub fn filter(mut self) -> Self {
        self.term.src.flags |= ECS_FILTER;
        self
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        unsafe {
            if !self.term_id.name.is_null() {
                let _ = CString::from_raw(self.term_id.name);
            }
            ecs_term_fini(&mut self.term);
        }
    }
}
