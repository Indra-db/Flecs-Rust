//! API for building [`Query`], [`Observer`] and [`System`][crate::addons::system::System] with their respected builders.

use crate::core::*;
use crate::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::format;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TermRefMode {
    #[default]
    Src,
    First,
    Second,
}

/// A reference to a term in a query.
/// This type is used to get information about a term in a query.
/// It is not possible to modify the term using this type.
/// To modify a term, use the `TermBuilder` interface.
/// Useful for debugging purposes.
pub struct TermRef<'a> {
    term: &'a sys::ecs_term_t,
}

impl<'a> TermRef<'a> {
    pub fn new(term: &'a sys::ecs_term_t) -> Self {
        Self { term }
    }

    pub fn is_set(&self) -> bool {
        unsafe { sys::ecs_term_is_initialized(self.term) }
    }

    pub fn id(&self) -> Id {
        Id(self.term.id)
    }

    pub fn inout(&self) -> InOutKind {
        self.term.inout.into()
    }

    pub fn oper(&self) -> OperKind {
        self.term.oper.into()
    }

    pub fn src_id(&self) -> Entity {
        let id = self.term.src.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }

    pub fn first_id(&self) -> Entity {
        let id = self.term.first.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }

    pub fn second_id(&self) -> Entity {
        let id = self.term.second.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }
}

#[doc(hidden)]
pub mod internals {
    use crate::core::*;
    use crate::sys;

    #[cfg(feature = "std")]
    extern crate std;

    extern crate alloc;
    use alloc::{string::String, vec::Vec};

    #[derive(Default)]
    pub struct TermBuilder {
        pub(crate) expr_count: i32,
        pub(crate) current_term_index: i32,
        pub(crate) next_term_index: i32,
        pub(crate) term_ref_mode: TermRefMode,
        pub(crate) str_ptrs_to_free: Vec<core::mem::ManuallyDrop<String>>,
    }

    #[doc(hidden)]
    pub trait QueryConfig<'a> {
        fn term_builder(&self) -> &TermBuilder;
        fn term_builder_mut(&mut self) -> &mut TermBuilder;

        fn query_desc(&self) -> &sys::ecs_query_desc_t;
        fn query_desc_mut(&mut self) -> &mut sys::ecs_query_desc_t;

        fn count_generic_terms(&self) -> i32;

        #[inline(always)]
        fn current_term_ref_mode(&self) -> TermRefMode {
            self.term_builder().term_ref_mode
        }

        #[inline(always)]
        fn set_term_ref_mode(&mut self, mode: TermRefMode) {
            self.term_builder_mut().term_ref_mode = mode;
        }

        #[inline(always)]
        fn term_mut_at(&mut self, index: i32) -> &mut sys::ecs_term_t {
            &mut self.query_desc_mut().terms[index as usize]
        }

        #[inline(always)]
        fn current_term_mut(&mut self) -> &mut sys::ecs_term_t {
            let index = self.current_term_index();
            self.term_mut_at(index)
        }

        #[inline(always)]
        fn current_term(&self) -> &sys::ecs_term_t {
            &self.query_desc().terms[self.term_builder().current_term_index as usize]
        }

        #[inline(always)]
        fn term_ref_mut(&mut self) -> &mut sys::ecs_term_ref_t {
            let term_mode = self.current_term_ref_mode();
            let term = self.current_term_mut();

            match term_mode {
                TermRefMode::Src => &mut term.src,
                TermRefMode::First => &mut term.first,
                TermRefMode::Second => &mut term.second,
            }
        }

        #[inline(always)]
        fn expr_count_mut(&mut self) -> &mut i32 {
            &mut self.term_builder_mut().expr_count
        }

        #[inline(always)]
        fn current_term_index(&self) -> i32 {
            self.term_builder().current_term_index
        }

        #[inline(always)]
        fn current_term_index_mut(&mut self) -> &mut i32 {
            &mut self.term_builder_mut().current_term_index
        }

        #[inline(always)]
        fn next_term_index(&self) -> i32 {
            self.term_builder().next_term_index
        }

        #[inline(always)]
        fn next_term_index_mut(&mut self) -> &mut i32 {
            &mut self.term_builder_mut().next_term_index
        }

        #[inline(always)]
        fn increment_current_term(&mut self) {
            *self.current_term_index_mut() += 1;
        }
    }
}

fn check_term_access_validity<'a>(term: &impl TermBuilderImpl<'a>) {
    if term.current_term_index() < term.count_generic_terms()
        && term.current_term_ref_mode() != TermRefMode::Src
    {
        panic!(
            "This function should only be used on terms that are not part of the generic type signature. "
        )
    }
}
/// Term builder interface for building queries, observers and systems.
/// A term is a single element of a query expression.
pub trait TermBuilderImpl<'a>: Sized + WorldProvider<'a> + internals::QueryConfig<'a> {
    /// initializes a new term from a id of a component or pair
    ///
    /// # Arguments
    ///
    /// * `id` - The id to use of pair or component
    fn init_current_term<T>(&mut self, id: T)
    where
        T: IntoId,
    {
        let id = id.into_id(self.world());
        let term = self.current_term_mut();

        #[allow(clippy::collapsible_else_if)]
        if T::IS_PAIR {
            term.id = *id;
        } else {
            if id & RUST_ecs_id_FLAGS_MASK != 0 {
                term.id = *id;
            } else {
                term.first.id = *id;
            }
        }
    }

    /// initialize a new term from a component or pair
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type of component to use.
    fn init_term_from<T: ComponentOrPairId>(&mut self) {
        if !T::IS_PAIR {
            let id: sys::ecs_id_t = T::First::id(self.world());
            self.init_current_term(id);
        } else {
            let world = self.world();
            let id_rel = T::First::id(world);
            let id_target = T::Second::id(world);
            self.init_current_term((id_rel, id_target));
        }
    }

    /// Reset the term
    fn reset(&mut self) {
        check_term_access_validity(self);

        // we don't for certain if this causes any side effects not using the nullptr and just using the default value.
        // if it does we can use Option.
        let term = self.current_term_mut();
        *term = Default::default();
    }

    /// Check if term is initialized
    ///
    /// Test whether a term is set. This operation can be used to test whether a term has been initialized with values or whether it is empty.
    ///
    /// An application generally does not need to invoke this operation.
    /// It is useful when initializing a 0-initialized array of terms (like in `sys::ecs_term_desc_t`)
    /// as this operation can be used to find the last initialized element.
    fn is_set(&mut self) -> bool {
        unsafe { sys::ecs_term_is_initialized(self.current_term()) }
    }

    /// Get the term id of the current term set
    ///
    /// # Returns
    ///
    /// The term id as `Id`.
    fn id(&self) -> Id {
        Id(self.current_term().id)
    }

    /// Get the inout type of term of the current term set
    fn inout(&self) -> InOutKind {
        self.current_term().inout.into()
    }

    /// Get the operator of term of the current term set
    fn oper(&self) -> OperKind {
        self.current_term().oper.into()
    }

    /// Get the src id of term of the current term set
    fn src_id(&self) -> Entity {
        let id = self.current_term().src.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }

    /// Get the first of term of the current term set
    fn first_id(&self) -> Entity {
        let id = self.current_term().first.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }

    /// Get the second of term of the current term set
    fn second_id(&self) -> Entity {
        let id = self.current_term().second.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }

    /// The self flag indicates the term identifier itself is used
    /// # See also
    ///
    fn self_(&mut self) -> &mut Self {
        self.term_ref_mut().id |= ECS_SELF;
        self
    }

    /// Specify value of identifier by id, same as `id()` of the current term set
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    fn set_id(&mut self, id: impl IntoEntity) -> &mut Self {
        let world = self.world();
        if self.current_term_ref_mode() != TermRefMode::Src {
            check_term_access_validity(self);
        }

        let term_ref = self.term_ref_mut();
        term_ref.id = *id.into_entity(world);
        self
    }

    /// Specify value of identifier by id. Almost the same as id(entity), but this
    /// operation explicitly sets the `flecs::IsEntity` flag. This forces the id to
    /// be interpreted as entity, whereas not setting the flag would implicitly
    /// convert ids for builtin variables such as `flecs::This` to a variable.
    ///
    /// This function can also be used to disambiguate `id(0)`, which would match
    /// both `id(Entity)` and `id(&str)`.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    fn entity(&mut self, entity: impl Into<Entity>) -> &mut Self {
        check_term_access_validity(self);

        self.term_ref_mut().id = *entity.into() | ECS_IS_ENTITY;
        self
    }

    /// Specify value of identifier by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    fn name(&mut self, name: &'a str) -> &mut Self {
        let name = core::mem::ManuallyDrop::new(format!("{}\0", name));
        let term_ref = self.term_ref_mut();
        term_ref.name = name.as_ptr() as *mut _;
        term_ref.id |= flecs::IsEntity::ID;
        self.term_builder_mut().str_ptrs_to_free.push(name);
        self
    }

    /// Specify identifier is a variable (resolved at query evaluation time)
    ///
    /// # Arguments
    ///
    /// * `var_name` - The name of the variable.
    fn set_var(&mut self, var_name: &'a str) -> &mut Self {
        check_term_access_validity(self);

        let var_name = core::mem::ManuallyDrop::new(format!("{}\0", var_name));
        let term_ref = self.term_ref_mut();
        term_ref.id |= flecs::IsVariable::ID;
        term_ref.name = var_name.as_ptr() as *mut _;
        self.term_builder_mut().str_ptrs_to_free.push(var_name);
        self
    }

    /// Override term id flags
    ///
    /// # Arguments
    ///
    /// * `flags` - The flags to set.
    fn flags(&mut self, flags: u64) -> &mut Self {
        check_term_access_validity(self);

        self.term_ref_mut().id = flags;
        self
    }

    /// Call prior to setting values for src identifier
    fn src(&mut self) -> &mut Self {
        self.set_term_ref_mode(TermRefMode::Src);
        self
    }

    /// Call prior to setting values for first identifier. This is either the
    /// component identifier, or first element of a pair (in case second is
    /// populated as well).
    fn first(&mut self) -> &mut Self {
        check_term_access_validity(self);

        self.set_term_ref_mode(TermRefMode::First);
        self
    }

    /// Call prior to setting values for second identifier. This is the second
    /// element of a pair. Requires that `first()` is populated as well.
    fn second(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.set_term_ref_mode(TermRefMode::Second);
        self
    }

    /// Select src identifier
    ///
    ///
    /// * initialize it with entity id or
    /// * initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    fn set_src<T: SingleAccessArg>(&mut self, id: T) -> &mut Self
    where
        Access: FromAccessArg<T>,
    {
        let access = Access::from_access_arg(id, self.world());

        match access.target {
            AccessTarget::Entity(entity) => self.src().set_id(entity),
            AccessTarget::Name(name) => {
                ecs_assert!(
                    !name.is_empty(),
                    FlecsErrorCode::InvalidParameter,
                    "name is empty"
                );
                self.src();
                if let Some(stripped_name) = strip_prefix_str_raw(name, "$") {
                    self.set_var(stripped_name)
                } else {
                    self.name(name)
                }
            }
            _ => panic!("Invalid access target, only single targets allowed"),
        }
    }

    /// Select first identifier
    ///
    /// * initialize with id or
    /// * initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    fn set_first<T: SingleAccessArg>(&mut self, id: T) -> &mut Self
    where
        Access: FromAccessArg<T>,
    {
        check_term_access_validity(self);
        let access = Access::from_access_arg(id, self.world());
        match access.target {
            AccessTarget::Entity(entity) => {
                self.first().set_id(entity);
            }
            AccessTarget::Name(name) => {
                ecs_assert!(
                    !name.is_empty(),
                    FlecsErrorCode::InvalidParameter,
                    "name is empty"
                );

                self.first();
                if let Some(stripped_name) = strip_prefix_str_raw(name, "$") {
                    self.set_var(stripped_name);
                } else {
                    self.name(name);
                }
            }
            _ => panic!("Invalid access target, only single targets allowed"),
        }
        // reset term ref mode to src, otherwise it stays on second and makes other actions potentially invalid
        self.set_term_ref_mode(TermRefMode::Src);
        self
    }

    /// Select second identifier
    ///
    /// * initialize with id or
    /// * initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    fn set_second<T: SingleAccessArg>(&mut self, id: T) -> &mut Self
    where
        Access: FromAccessArg<T>,
    {
        check_term_access_validity(self);
        let access = Access::from_access_arg(id, self.world());
        match access.target {
            AccessTarget::Entity(entity) => {
                self.second().set_id(entity);
            }
            AccessTarget::Name(name) => {
                ecs_assert!(
                    !name.is_empty(),
                    FlecsErrorCode::InvalidParameter,
                    "name is empty"
                );

                self.second();
                if let Some(stripped_name) = strip_prefix_str_raw(name, "$") {
                    self.set_var(stripped_name);
                } else {
                    self.name(name);
                }
            }
            _ => panic!("Invalid access target, only single targets allowed"),
        }
        // reset term ref mode to src, otherwise it stays on second and makes other actions potentially invalid
        self.set_term_ref_mode(TermRefMode::Src);
        self
    }

    /// default up where trav is set to 0.
    /// The up flag indicates that the term identifier may be substituted by
    /// traversing a relationship upwards. For example: substitute the identifier
    /// with its parent by traversing the `ChildOf` relationship.
    #[inline]
    fn up(&mut self) -> &mut Self {
        ecs_assert!(
            self.current_term_ref_mode() == TermRefMode::Src,
            FlecsErrorCode::InvalidParameter,
            "up traversal can only be applied to term source"
        );
        self.term_ref_mut().id |= ECS_UP;
        self
    }

    /// same as [`up`](crate::core::term)
    #[inline]
    fn parent(&mut self) -> &mut Self {
        self.up()
    }

    /// The up flag indicates that the term identifier may be substituted by
    /// traversing a relationship upwards. For example: substitute the identifier
    /// with its parent by traversing the `ChildOf` relationship.
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The relationship to traverse.
    fn up_id(&mut self, traverse_relationship: impl IntoEntity) -> &mut Self {
        ecs_assert!(
            self.current_term_ref_mode() == TermRefMode::Src,
            FlecsErrorCode::InvalidParameter,
            "up traversal can only be applied to term source"
        );
        let term_ref = self.term_ref_mut();
        term_ref.id |= ECS_UP;
        self.current_term_mut().trav = *traverse_relationship.into_entity(self.world());
        self
    }

    /// Cascade iterates a hierarchy in top to bottom order (breadth first search)
    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for `flecs::query`.
    fn cascade(&mut self) -> &mut Self {
        self.up();
        self.term_ref_mut().id |= ECS_CASCADE;
        self
    }

    /// Cascade iterates a hierarchy in top to bottom order (breadth first search)
    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for `flecs::query`
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The optional relationship to traverse.
    fn cascade_id(&mut self, traverse_relationship: impl IntoEntity) -> &mut Self {
        self.up_id(traverse_relationship);
        self.term_ref_mut().id |= ECS_CASCADE;
        self
    }

    /// Use with cascade to iterate results in descending (bottom + top) order.
    fn desc(&mut self) -> &mut Self {
        self.term_ref_mut().id |= ECS_DESC;
        self
    }

    /// Specify relationship to traverse, and flags to indicate direction
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The relationship to traverse.
    /// * `flags` - The direction to traverse.
    fn trav(&mut self, traverse_relationship: impl IntoEntity, flags: u64) -> &mut Self {
        self.current_term_mut().trav = *traverse_relationship.into_entity(self.world());
        self.term_ref_mut().id |= flags;
        self
    }

    /// Set id flags for term.
    ///
    /// # Arguments
    ///
    /// * `flags` - The direction to traverse.
    fn id_flags(&mut self, flags: impl IntoId) -> &mut Self {
        self.term_ref_mut().id |= *flags.into_id(self.world());
        self
    }

    /// Set read/write access of term
    ///
    /// # Arguments
    ///
    /// * `inout` - The inout to set.
    fn set_inout_kind(&mut self, inout: InOutKind) -> &mut Self {
        check_term_access_validity(self);
        self.current_term_mut().inout = inout.into();
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
        check_term_access_validity(self);
        self.set_inout_kind(inout);
        if self.current_term_mut().oper != OperKind::Not as i16 {
            self.src().entity(0);
        }

        self
    }

    /// Set write mode on current term.
    /// Short for `inout_stage(InOutKind::Out)`.
    /// Use when system uses add, remove or set.
    ///
    /// # See also
    ///
    /// * [`Self::inout_stage`]
    /// * [`InOutKind`]
    #[inline(always)]
    fn write_curr(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.inout_stage(InOutKind::Out)
    }

    /// Set read mode on current term.
    /// Short for `inout_stage(InOutKind::In)`.
    /// Use when system uses get
    ///
    /// # See also
    ///
    /// * [`Self::inout_stage`]
    /// * [`InOutKind`]
    #[inline(always)]
    fn read_curr(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.inout_stage(InOutKind::In)
    }

    /// Short for `inout_stage(InOutKind::InOut)`.
    /// Use when system uses `ensure`
    ///
    /// # See also
    ///
    /// * [`Self::inout_stage`]
    /// * [`InOutKind`]
    #[inline(always)]
    fn read_write_curr(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.inout_stage(InOutKind::InOut)
    }

    /// short for `set_inout(InOutKind::In)`
    ///
    /// # See also
    ///
    /// * [`Self::inout_stage`]
    /// * [`InOutKind`]
    #[inline(always)]
    fn set_in(&mut self) -> &mut Self {
        if self.current_term_index() < self.count_generic_terms() {
            panic!(
                "This function should only be used on terms that are not part of the generic type signature. use &T instead"
            )
        }
        self.set_inout_kind(InOutKind::In)
    }

    /// short for `set_inout(InOutKind::Out)`
    ///
    /// # See also
    ///
    /// * [`Self::inout_stage`]
    /// * [`InOutKind`]
    #[inline(always)]
    fn set_out(&mut self) -> &mut Self {
        if self.current_term_index() < self.count_generic_terms() {
            panic!(
                "This function should only be used on terms that are not part of the generic type signature. Use &mut T instead."
            )
        }
        self.set_inout_kind(InOutKind::Out)
    }

    /// short for `set_inout(InOutKind::InOut)`
    ///
    /// # See also
    ///
    /// * [`Self::inout_stage`]
    /// * [`InOutKind`]
    #[inline(always)]
    fn set_inout(&mut self) -> &mut Self {
        if self.current_term_index() < self.count_generic_terms() {
            panic!(
                "This function should only be used on terms that are not part of the generic type signature. Use &mut T instead."
            )
        }
        self.set_inout_kind(InOutKind::InOut)
    }

    /// short for `set_inout(InOutKind::None)`
    ///
    /// # See also
    ///
    /// * [`Self::inout_stage`]
    /// * [`InOutKind`]
    #[inline(always)]
    fn set_inout_none(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.current_term_mut().inout = InOutKind::None as i16;
        self
    }

    /// set operator of term
    ///
    /// # Arguments
    ///
    /// * `oper` - The operator to set.
    #[inline(always)]
    fn set_oper(&mut self, oper: OperKind) -> &mut Self {
        check_term_access_validity(self);
        self.current_term_mut().oper = oper as i16;
        self
    }

    /// short for `set_oper(OperKind::And)`
    ///
    /// # See also
    ///
    /// * [`Self::set_oper`]
    /// * [`OperKind`]
    #[inline(always)]
    fn and(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.set_oper(OperKind::And)
    }

    /// short for `set_oper(OperKind::Or)`
    ///
    /// # See also
    ///
    /// * [`Self::set_oper`]
    /// * [`OperKind`]
    #[inline(always)]
    fn or(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.set_oper(OperKind::Or)
    }

    /// short for `set_oper(OperKind::Not)`
    ///
    /// # See also
    ///
    /// * [`Self::set_oper`]
    /// * [`OperKind`]
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    fn not(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.set_oper(OperKind::Not)
    }

    /// short for `set_oper(OperKind::Optional)`
    ///
    /// # See also
    ///
    /// * [`Self::set_oper`]
    /// * [`OperKind`]
    #[inline(always)]
    fn optional(&mut self) -> &mut Self {
        if self.current_term_index() < self.count_generic_terms() {
            panic!(
                "This function should only be used on terms that are not part of the generic type signature. Use Option<> instead."
            )
        }
        self.set_oper(OperKind::Optional)
    }

    /// short for `set_oper(OperKind::AndFrom)`
    ///
    /// # See also
    ///
    /// * [`Self::set_oper`]
    /// * [`OperKind`]
    #[inline(always)]
    fn and_from(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.set_oper(OperKind::AndFrom)
    }

    /// short for `set_oper(OperKind::OrFrom)`
    ///
    /// # See also
    ///
    /// * [`Self::set_oper`]
    /// * [`OperKind`]
    #[inline(always)]
    fn or_from(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.set_oper(OperKind::OrFrom)
    }

    /// short for `set_oper(OperKind::NotFrom)`
    ///
    /// # See also
    ///
    /// * [`Self::set_oper`]
    /// * [`OperKind`]
    #[inline(always)]
    fn not_from(&mut self) -> &mut Self {
        check_term_access_validity(self);
        self.set_oper(OperKind::NotFrom)
    }

    /// Match singleton
    fn singleton(&mut self) -> &mut Self {
        ecs_assert!(
            self.current_term_mut().id != 0 || self.current_term_mut().first.id != 0,
            FlecsErrorCode::InvalidParameter,
            "no component specified for singleton"
        );

        unsafe {
            let sid = if self.current_term_mut().id != 0 {
                self.current_term_mut().id
            } else {
                self.current_term_mut().first.id
            };

            ecs_assert!(sid != 0, FlecsErrorCode::InvalidParameter, "invalid id");

            if !ecs_is_pair(sid) {
                self.current_term_mut().src.id = sid;
            } else {
                self.current_term_mut().src.id =
                    sys::ecs_get_alive(self.world_ptr_mut(), *ecs_first(sid, self.world()));
            }
        }
        self
    }

    /// Query terms are not triggered on by observers
    #[inline(always)]
    fn filter(&mut self) -> &mut Self {
        self.current_term_mut().inout = InOutKind::Filter as i16;
        self
    }
}
