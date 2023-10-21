use std::ffi::CString;

use crate::{core::utility::errors::FlecsErrorCode, ecs_assert};

use super::{
    c_binding::bindings::{
        ecs_term_copy, ecs_term_finalize, ecs_term_fini, ecs_term_id_t, ecs_term_is_initialized,
        ecs_term_move, ecs_term_t, ECS_ID_FLAGS_MASK,
    },
    c_types::{
        EntityT, Flags32T, IdT, InOutKind, OperKind, TermIdT, TermT, WorldT, ECS_CASCADE,
        ECS_FILTER, ECS_IS_ENTITY, ECS_IS_NAME, ECS_IS_VARIABLE, ECS_PARENT, ECS_SELF, ECS_UP,
    },
    component_registration::CachedComponentData,
    entity::Entity,
    id::Id,
    utility::functions::ecs_pair,
    world::World,
};

/// Struct that describes a term identifier.
///
/// A term is a single element of a query expression.
///
/// A term identifier describes a single identifier in a term. Identifier
/// descriptions can reference entities by id, name or by variable, which means
/// the entity will be resolved when the term is evaluated.
pub struct Term {
    pub term_id: *mut TermIdT,
    pub term_ptr: *mut TermT,
    pub term: TermT,
    world: *mut WorldT,
}

impl Default for Term {
    fn default() -> Self {
        Self {
            term_id: std::ptr::null_mut(),
            term_ptr: std::ptr::null_mut(),
            term: Default::default(),
            world: std::ptr::null_mut(),
        }
    }
}

/// this is for copying the term
impl Clone for Term {
    fn clone(&self) -> Self {
        let mut obj = Self {
            term_id: std::ptr::null_mut(),
            term_ptr: std::ptr::null_mut(),
            term: Default::default(),
            world: self.world,
        };
        obj.term = unsafe { ecs_term_copy(&self.term) };
        let obj_term = &mut obj.term as *mut TermT;
        obj.set_term(obj_term);
        obj
    }
}

pub enum With {
    Term(TermT),
    Id(IdT),
    Pair(EntityT, EntityT),
}

impl Term {
    pub fn new(world: Option<&World>, with: With) -> Self {
        if let Some(world) = world {
            match with {
                With::Term(term) => Self::new_term(world.raw_world, term),
                With::Id(id) => Self::new_id(world.raw_world, id),
                With::Pair(rel, target) => Self::new_rel_target(world.raw_world, rel, target),
            }
        } else {
            match with {
                With::Term(term) => {
                    ecs_assert!(false, FlecsErrorCode::InvalidParameter, "world is None");
                    Self::new_term(std::ptr::null_mut(), term)
                }
                With::Id(id) => Self::new_id_only(id),
                With::Pair(rel, target) => Self::new_rel_target_only(rel, target),
            }
        }
    }

    pub fn new_world_only<'w>(world: &'w World) -> Self {
        let mut obj = Self {
            world: world.raw_world,
            term_id: std::ptr::null_mut(),
            term: Default::default(),
            term_ptr: std::ptr::null_mut(),
        };
        obj.term.move_ = true;
        obj
    }

    pub fn new_component<T: CachedComponentData>(world: Option<&World>) -> Self {
        if let Some(world) = world {
            Self::new_id(world.raw_world, T::get_id(world.raw_world))
        } else {
            let id: IdT = if T::is_registered() {
                unsafe { T::get_id_unchecked() }
            } else {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidOperation,
                    "component not registered"
                );
                0
            };
            Self::new_id_only(id)
        }
    }

    pub fn new_pair<T: CachedComponentData, U: CachedComponentData>(world: Option<&World>) -> Self {
        if let Some(world) = world {
            Self::new_rel_target(
                world.raw_world,
                T::get_id(world.raw_world),
                U::get_id(world.raw_world),
            )
        } else {
            let id_rel = if T::is_registered() {
                unsafe { T::get_id_unchecked() }
            } else {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidOperation,
                    "component not registered"
                );
                0
            };

            let id_target = if U::is_registered() {
                unsafe { U::get_id_unchecked() }
            } else {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidOperation,
                    "component not registered"
                );
                0
            };

            Self::new_rel_target_only(id_rel, id_target)
        }
    }

    fn new_term(world: *mut WorldT, term: TermT) -> Self {
        let mut obj = Self {
            world,
            term_id: std::ptr::null_mut(),
            term,
            term_ptr: std::ptr::null_mut(),
        };
        obj.term.move_ = false;
        let obj_term = &mut obj.term as *mut TermT;
        obj.set_term(obj_term);
        obj
    }

    fn new_id(world: *mut WorldT, id: IdT) -> Self {
        let mut obj = Self {
            world,
            term_id: std::ptr::null_mut(),
            term_ptr: std::ptr::null_mut(),
            term: Default::default(),
        };
        if id & ECS_ID_FLAGS_MASK as u64 != 0 {
            obj.term.id = id;
        } else {
            obj.term.first.id = id;
        }
        obj.term.move_ = false;
        let obj_term = &mut obj.term as *mut TermT;
        obj.set_term(obj_term);
        obj
    }

    fn new_id_only(id: IdT) -> Self {
        let mut obj = Self {
            world: std::ptr::null_mut(),
            term_id: std::ptr::null_mut(),
            term_ptr: std::ptr::null_mut(),
            term: Default::default(),
        };
        if id & ECS_ID_FLAGS_MASK as u64 != 0 {
            obj.term.id = id;
        } else {
            obj.term.first.id = id;
        }
        obj.term.move_ = true;
        obj
    }

    fn new_rel_target(world: *mut WorldT, rel: EntityT, target: EntityT) -> Self {
        let mut obj = Self {
            world,
            term_id: std::ptr::null_mut(),
            term_ptr: std::ptr::null_mut(),
            term: Default::default(),
        };
        obj.term.id = ecs_pair(rel, target);
        obj.term.move_ = false;
        let obj_term = &mut obj.term as *mut TermT;
        obj.set_term(obj_term);
        obj
    }

    fn new_rel_target_only(rel: EntityT, target: EntityT) -> Self {
        let mut obj = Self {
            world: std::ptr::null_mut(),
            term_id: std::ptr::null_mut(),
            term_ptr: std::ptr::null_mut(),
            term: Default::default(),
        };
        obj.term.id = ecs_pair(rel, target);
        obj.term.move_ = true;
        obj
    }

    /// This is how you should move a term, not the default rust way
    pub fn move_term(mut self) -> Self {
        let mut obj = Self {
            world: self.world,
            term_id: std::ptr::null_mut(),
            term_ptr: std::ptr::null_mut(),
            term: Default::default(),
        };
        obj.term = unsafe { ecs_term_move(&mut self.term) };
        self.reset();
        let obj_term = &mut obj.term as *mut TermT;
        obj.set_term(obj_term);
        obj
    }

    pub fn reset(&mut self) {
        // we don't for certain if this causes any side effects not using the nullptr and just using the default value.
        // if it does we can use Option.
        self.term = Default::default();
        self.set_term(std::ptr::null_mut());
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

    pub fn move_raw_term(&mut self) -> TermT {
        unsafe { ecs_term_move(&mut self.term) }
    }
}

/// Builder pattern functions
impl Term {}

impl Drop for Term {
    fn drop(&mut self) {
        unsafe {
            if !self.term_id.is_null() && !(*self.term_id).name.is_null() {
                let _ = CString::from_raw((*self.term_id).name);
            }
            ecs_term_fini(&mut self.term);
        }
    }
}

pub trait TermBuilder: Sized {
    fn get_world(&self) -> *mut WorldT;

    fn get_term(&mut self) -> &mut Term;

    fn get_raw_term(&mut self) -> *mut TermT;

    fn get_term_id(&mut self) -> *mut TermIdT;

    /// Set term to a specific term
    ///
    /// # Arguments
    ///
    /// * `term` - The term to set.
    ///
    /// # C++ API Equivalent
    ///
    /// term_builder_i::set_term`
    fn set_term(&mut self, term: *mut TermT) {
        let self_term: &mut Term = self.get_term();
        self_term.term_ptr = term;

        self_term.term_id = if !term.is_null() {
            unsafe { &mut (*term).src }
        } else {
            std::ptr::null_mut()
        };
    }

    fn assert_term_id(&mut self) {
        ecs_assert!(
            self.get_term_id() != std::ptr::null_mut(),
            FlecsErrorCode::InvalidParameter,
            "no active term (call .term() first"
        );
    }

    fn assert_term(&mut self) {
        ecs_assert!(
            self.get_raw_term() != std::ptr::null_mut(),
            FlecsErrorCode::InvalidParameter,
            "no active term (call .term() first"
        );
    }

    /// The self flag indicates the term identifier itself is used
    fn self_term(&mut self) -> &mut Self {
        self.assert_term_id();
        unsafe { (*self.get_term_id()).flags |= ECS_SELF };
        self
    }

    /// The up flag indicates that the term identifier may be substituted by
    /// traversing a relationship upwards. For example: substitute the identifier
    /// with its parent by traversing the ChildOf relationship.
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The optional relationship to traverse.
    fn up_id(&mut self, traverse_relationship: Option<EntityT>) -> &mut Self {
        self.assert_term_id();
        unsafe { (*self.get_term_id()).flags |= ECS_UP };
        if let Some(trav_rel) = traverse_relationship {
            unsafe { (*self.get_term_id()).trav = trav_rel };
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
    fn up<TravRel: CachedComponentData>(&mut self) -> &mut Self {
        self.assert_term_id();
        unsafe {
            (*self.get_term_id()).flags |= ECS_UP;
            (*self.get_term_id()).trav = TravRel::get_id(self.get_world())
        };
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for flecs::query
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The optional relationship to traverse.
    fn cascade_id(&mut self, traverse_relationship: Option<EntityT>) -> &mut Self {
        self.assert_term_id();
        unsafe { (*self.get_term_id()).flags |= ECS_CASCADE };
        if let Some(trav_rel) = traverse_relationship {
            unsafe { (*self.get_term_id()).trav = trav_rel };
        }
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for flecs::query
    ///
    /// # Type Arguments
    ///
    /// * `TravRel` - The relationship to traverse.
    fn cascade<TravRel: CachedComponentData>(&mut self) -> &mut Self {
        self.assert_term_id();
        unsafe {
            (*self.get_term_id()).flags |= ECS_CASCADE;
            (*self.get_term_id()).trav = TravRel::get_id(self.get_world())
        };
        self
    }

    /// the parent flag is short for up (flecs::ChildOf)
    fn parent(&mut self) -> &mut Self {
        self.assert_term_id();
        unsafe { (*self.get_term_id()).flags |= ECS_PARENT };
        self
    }

    /// Specify relationship to traverse, and flags to indicate direction
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The relationship to traverse.
    /// * `flags` - The direction to traverse.
    fn trav(&mut self, traverse_relationship: EntityT, flags: Flags32T) -> &mut Self {
        self.assert_term_id();
        unsafe {
            (*self.get_term_id()).trav = traverse_relationship;
            (*self.get_term_id()).flags |= flags
        };
        self
    }

    /// Specify value of identifier by id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    fn id(&mut self, id: EntityT) -> &mut Self {
        self.assert_term_id();
        unsafe { (*self.get_term_id()).id = id };
        self
    }

    /// Specify value of identifier by id, same as id()
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    fn set_term_id(&mut self, id: IdT) -> &mut Self {
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
    fn entity(&mut self, id: EntityT) -> &mut Self {
        self.assert_term_id();
        unsafe {
            (*self.get_term_id()).flags |= ECS_IS_ENTITY;
            (*self.get_term_id()).id = id
        };
        self
    }

    /// Specify value of identifier by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    fn name(&mut self, name: &str) -> &mut Self {
        self.assert_term_id();
        let c_name = CString::new(name).unwrap();
        let leak_name = CString::into_raw(c_name);
        unsafe {
            (*self.get_term_id()).name = leak_name;
            (*self.get_term_id()).flags |= ECS_IS_NAME
        };
        self
    }

    /// Specify identifier is a variable (resolved at query evaluation time)
    ///
    /// # Arguments
    ///
    /// * `var_name` - The name of the variable.
    fn var(&mut self, var_name: &str) -> &mut Self {
        self.assert_term_id();
        let c_name = CString::new(var_name).unwrap();
        let leak_name = CString::into_raw(c_name);
        unsafe {
            (*self.get_term_id()).flags |= ECS_IS_VARIABLE;
            (*self.get_term_id()).name = leak_name
        };
        self
    }

    /// Override term id flags
    ///
    /// # Arguments
    ///
    /// * `flags` - The flags to set.
    fn flags(&mut self, flags: Flags32T) -> &mut Self {
        self.assert_term_id();
        unsafe { (*self.get_term_id()).flags = flags };
        self
    }

    /// Call prior to setting values for src identifier
    fn setup_src(&mut self) -> &mut Self {
        self.assert_term();
        unsafe { *self.get_term_id() = (*self.get_raw_term()).src };
        self
    }

    /// Call prior to setting values for first identifier. This is either the
    /// component identifier, or first element of a pair (in case second is
    /// populated as well).
    fn setup_first(&mut self) -> &mut Self {
        self.assert_term();
        unsafe { *self.get_term_id() = (*self.get_raw_term()).first };
        self
    }

    /// Call prior to setting values for second identifier. This is the second
    /// element of a pair. Requires that first() is populated as well.
    fn setup_second(&mut self) -> &mut Self {
        self.assert_term();
        unsafe { *self.get_term_id() = (*self.get_raw_term()).second };
        self
    }

    /// Select src identifier, initialize it with entity id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    fn src_id(&mut self, id: EntityT) -> &mut Self {
        self.setup_src().id(id)
    }

    /// Select src identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    fn src<T: CachedComponentData>(&mut self) -> &mut Self {
        let world = self.get_world();
        self.src_id(T::get_id(world))
    }

    /// Select src identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    fn src_name(&mut self, name: &str) -> &mut Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_src();
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
    fn first_id(&mut self, id: EntityT) -> &mut Self {
        self.setup_first().id(id)
    }

    /// Select first identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    fn first<T: CachedComponentData>(&mut self) -> &mut Self {
        let world = self.get_world();
        self.first_id(T::get_id(world))
    }

    /// Select first identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    fn first_name(&mut self, name: &str) -> &mut Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_first();
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
    fn second_id(&mut self, id: EntityT) -> &mut Self {
        self.setup_second().id(id)
    }

    /// Select second identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    fn second<T: CachedComponentData>(&mut self) -> &mut Self {
        let world = self.get_world();
        self.second_id(T::get_id(world))
    }

    /// Select second identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    fn second_name(&mut self, name: &str) -> &mut Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_second();
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
    fn role(&mut self, role: IdT) -> &mut Self {
        self.assert_term();
        unsafe { (*self.get_raw_term()).id_flags = role };
        self
    }

    /// Set read=write access of term
    ///
    /// # Arguments
    ///
    /// * `inout` - The inout to set.
    fn set_inout(&mut self, inout: InOutKind) -> &mut Self {
        self.assert_term();
        unsafe { (*self.get_raw_term()).inout = inout as ::std::os::raw::c_int };
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
    fn inout_stage(&mut self, inout: InOutKind) -> &mut Self {
        self.assert_term();
        self.set_inout(inout);
        unsafe {
            if (*self.get_raw_term()).inout != OperKind::Not as ::std::os::raw::c_int {
                self.setup_src().entity(0);
            }
        }
        self
    }

    /// Short for inout_stage(flecs::Out).
    ///  Use when system uses add, remove or set.
    ///
    fn write(&mut self) -> &mut Self {
        self.inout_stage(InOutKind::Out)
    }

    /// Short for inout_stage(flecs::In).
    /// Use when system uses get
    fn read(&mut self) -> &mut Self {
        self.inout_stage(InOutKind::In)
    }

    /// Short for inout_stage(flecs::InOut).
    /// Use when system uses get_mut
    fn read_write(&mut self) -> &mut Self {
        self.inout_stage(InOutKind::InOut)
    }

    /// short for inout(flecs::In)
    fn in_(&mut self) -> &mut Self {
        self.set_inout(InOutKind::In)
    }

    /// short for inout(flecs::Out)
    fn out(&mut self) -> &mut Self {
        self.set_inout(InOutKind::Out)
    }

    /// short for inout(flecs::InOut)
    fn inout(&mut self) -> &mut Self {
        self.set_inout(InOutKind::InOut)
    }

    /// short for inout(flecs::InOutNone)
    fn inout_none(&mut self) -> &mut Self {
        self.set_inout(InOutKind::InOutNone)
    }

    /// set operator of term
    ///
    /// # Arguments
    ///
    /// * `oper` - The operator to set.
    fn oper(&mut self, oper: OperKind) -> &mut Self {
        self.assert_term_id();
        unsafe { (*self.get_raw_term()).oper = oper as ::std::os::raw::c_int };
        self
    }

    /// short for oper(flecs::And)
    fn and(&mut self) -> &mut Self {
        self.oper(OperKind::And)
    }

    /// short for oper(flecs::Or)
    fn or(&mut self) -> &mut Self {
        self.oper(OperKind::Or)
    }

    /// short for oper(flecs::Not)
    #[allow(clippy::should_implement_trait)]
    fn not(&mut self) -> &mut Self {
        self.oper(OperKind::Not)
    }

    /// short for oper(flecs::Optional)
    fn optional(&mut self) -> &mut Self {
        self.oper(OperKind::Optional)
    }

    /// short for oper(flecs::AndFrom)
    fn and_from(&mut self) -> &mut Self {
        self.oper(OperKind::AndFrom)
    }

    /// short for oper(flecs::OrFrom)
    fn or_from(&mut self) -> &mut Self {
        self.oper(OperKind::OrFrom)
    }

    /// short for oper(flecs::NotFrom)
    fn not_from(&mut self) -> &mut Self {
        self.oper(OperKind::NotFrom)
    }

    /// Match singleton
    fn singleton(&mut self) -> &mut Self {
        self.assert_term();

        ecs_assert!(
            unsafe { (*self.get_raw_term()).id != 0 || (*self.get_raw_term()).first.id != 0 },
            FlecsErrorCode::InvalidParameter,
            "no component specified for singleton"
        );

        unsafe {
            let sid = if (*self.get_raw_term()).id != 0 {
                (*self.get_raw_term()).id
            } else {
                (*self.get_raw_term()).first.id
            };

            ecs_assert!(sid != 0, FlecsErrorCode::InvalidParameter, "invalid id");
            (*self.get_raw_term()).src.id = sid;
        }
        self
    }

    /// Filter terms are not triggered on by observers
    fn filter(&mut self) -> &mut Self {
        unsafe { (*self.get_raw_term()).src.flags |= ECS_FILTER };
        self
    }
}

impl TermBuilder for Term {
    fn get_world(&self) -> *mut WorldT {
        self.world
    }

    fn get_term(&mut self) -> &mut Term {
        self
    }

    fn get_raw_term(&mut self) -> *mut TermT {
        self.term_ptr
    }

    fn get_term_id(&mut self) -> *mut TermIdT {
        self.term_id
    }
}
