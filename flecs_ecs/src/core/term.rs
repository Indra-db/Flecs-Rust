use std::ffi::CStr;

use crate::core::*;
use crate::sys;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TermRefMode {
    #[default]
    Src,
    First,
    Second,
}

/// Term builder interface.
/// A term is a single element of a query expression.
pub trait TermBuilder<'a>: Sized + IntoWorld<'a> {
    fn current_term_ref_mode(&self) -> TermRefMode;

    fn set_term_ref_mode(&mut self, mode: TermRefMode);

    fn get_term_mut_at(&mut self, index: i32) -> &mut TermT;

    fn get_current_term_mut(&mut self) -> &mut TermT;

    fn get_current_term(&self) -> &TermT;

    fn term_ref_mut(&mut self) -> &mut TermRefT;

    /// initializes a new term from a id of a component or pair
    ///
    /// # Arguments
    ///
    /// * `id` - The id to use of pair or component
    ///
    /// # See also
    ///
    /// * C++ API: `term::term`
    #[doc(alias = "term::term")]
    fn init_current_term<T>(&mut self, id: T)
    where
        T: IntoId,
    {
        let id = id.into();
        let term = self.get_current_term_mut();

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
    ///
    /// # See also
    ///
    /// * C++ API: `term::term`
    #[doc(alias = "term::term")]
    fn init_term_from<T: IntoComponentId>(&mut self) {
        if !T::IS_PAIR {
            let id: IdT = if T::First::is_registered() {
                unsafe { T::First::get_id_unchecked() }
            } else {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidOperation,
                    "component not registered"
                );
                0
            };
            self.init_current_term(id);
        } else {
            let id_rel = if T::First::is_registered() {
                unsafe { T::First::get_id_unchecked() }
            } else {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidOperation,
                    "component not registered"
                );
                0
            };

            let id_target = if T::Second::is_registered() {
                unsafe { T::Second::get_id_unchecked() }
            } else {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidOperation,
                    "component not registered"
                );
                0
            };

            self.init_current_term((id_rel, id_target))
        }
    }

    /// Reset the term
    ///
    /// # See also
    ///
    /// * C++ API: `term::reset`
    #[doc(alias = "term::reset")]
    fn reset(&mut self) {
        // we don't for certain if this causes any side effects not using the nullptr and just using the default value.
        // if it does we can use Option.
        let term = self.get_current_term_mut();
        *term = Default::default();
    }

    /// Check if term is initialized
    ///
    /// Test whether a term is set. This operation can be used to test whether a term has been initialized with values or whether it is empty.
    ///
    /// An application generally does not need to invoke this operation.
    /// It is useful when initializing a 0-initialized array of terms (like in `sys::ecs_term_desc_t`)
    /// as this operation can be used to find the last initialized element.
    ///
    /// # See also
    ///
    /// * C++ API: `term::is_set`
    #[doc(alias = "term::is_set")]
    // todo v4, probably remove this, since we don't store unitialized terms anymore like CPP API.
    fn is_set(&mut self) -> bool {
        unsafe { sys::ecs_term_is_initialized(self.get_current_term()) }
    }

    /// Get the term id of the current term set
    ///
    /// # Returns
    ///
    /// The term id as `Id`.
    ///
    /// # See also
    ///
    /// * C++ API: `term::id`
    #[doc(alias = "term::id")]
    fn id(&self) -> Id {
        Id(self.get_current_term().id)
    }

    /// Get the inout type of term of the current term set
    ///
    /// # See also
    ///
    /// * C++ API: `term::inout`
    #[doc(alias = "term::inout")]
    fn inout(&self) -> InOutKind {
        self.get_current_term().inout.into()
    }

    /// Get the operator of term of the current term set
    ///
    /// # See also
    ///
    /// * C++ API: `term::oper`
    #[doc(alias = "term::oper")]
    fn oper(&self) -> OperKind {
        self.get_current_term().oper.into()
    }

    /// Get the src id of term of the current term set
    ///
    /// # See also
    ///
    /// * C++ API: `term::get_src`
    #[doc(alias = "term::get_src")]
    fn src(&self) -> Entity {
        //id & ~EcsTermRefFlags
        let id = self.get_current_term().src.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }

    /// Get the first of term of the current term set
    ///
    /// # See also
    ///
    /// * C++ API: `term::first`
    #[doc(alias = "term::get_first")]
    fn first(&self) -> Entity {
        let id = self.get_current_term().first.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }

    /// Get the second of term of the current term set
    ///
    /// # See also
    ///
    /// * C++ API: `term::second`
    #[doc(alias = "term::get_second")]
    fn second(&self) -> Entity {
        let id = self.get_current_term().second.id & !flecs::TermRefFlags::ID;
        Entity(id)
    }

    /// The self flag indicates the term identifier itself is used
    /// # See also
    ///
    /// * C++ API: `term_builder_i::self`
    #[doc(alias = "term_builder_i::self")]
    fn self_term(&mut self) -> &mut Self {
        self.term_ref_mut().id |= ECS_SELF;
        self
    }

    /// Specify value of identifier by id, same as `id()` of the current term set
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::id`
    #[doc(alias = "term_builder_i::id")]
    fn set_term_ref_id(&mut self, id: impl Into<Entity>) -> &mut Self {
        let term_ref = self.term_ref_mut();
        term_ref.id = *id.into();
        self
    }

    /// Specify value of identifier by id. Almost the same as id(entity), but this
    /// operation explicitly sets the `flecs::IsEntity` flag. This forces the id to
    /// be interpreted as entity, whereas not setting the flag would implicitly
    /// convert ids for builtin variables such as `flecs::This` to a variable.
    ///
    /// This function can also be used to disambiguate id(0), which would match
    /// both id(Entity) and id(&str).
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::entity`
    #[doc(alias = "term_builder_i::entity")]
    fn entity(&mut self, entity: impl Into<Entity>) -> &mut Self {
        self.term_ref_mut().id = *entity.into() | ECS_IS_ENTITY;
        self
    }

    /// Specify value of identifier by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::name`
    #[doc(alias = "term_builder_i::name")]
    fn name(&mut self, name: &CStr) -> &mut Self {
        let term_ref = self.term_ref_mut();
        term_ref.name = name.as_ptr() as *mut i8;
        term_ref.id |= flecs::IsEntity::ID;
        self
    }

    /// Specify identifier is a variable (resolved at query evaluation time)
    ///
    /// # Arguments
    ///
    /// * `var_name` - The name of the variable.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::var`
    #[doc(alias = "term_builder_i::var")]
    fn var(&mut self, var_name: &CStr) -> &mut Self {
        let term_ref = self.term_ref_mut();
        term_ref.id |= flecs::IsVariable::ID;
        term_ref.name = var_name.as_ptr() as *mut i8;
        self
    }

    /// Override term id flags
    ///
    /// # Arguments
    ///
    /// * `flags` - The flags to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::flags`
    #[doc(alias = "term_builder_i::flags")]
    fn flags(&mut self, flags: u64) -> &mut Self {
        self.term_ref_mut().id = flags;
        self
    }

    /// Call prior to setting values for src identifier
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::src`
    #[doc(alias = "term_builder_i::src")]
    fn setup_src(&mut self) -> &mut Self {
        self.set_term_ref_mode(TermRefMode::Src);
        self
    }

    /// Call prior to setting values for first identifier. This is either the
    /// component identifier, or first element of a pair (in case second is
    /// populated as well).
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::first`
    #[doc(alias = "term_builder_i::first")]
    fn setup_first(&mut self) -> &mut Self {
        self.set_term_ref_mode(TermRefMode::First);
        self
    }

    /// Call prior to setting values for second identifier. This is the second
    /// element of a pair. Requires that `first()` is populated as well.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::second`
    #[doc(alias = "term_builder_i::second")]
    fn setup_second(&mut self) -> &mut Self {
        self.set_term_ref_mode(TermRefMode::Second);
        self
    }

    /// Select src identifier, initialize it with entity id
    ///
    /// # Arguments
    ///
    /// * `id` - The id to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::src`
    #[doc(alias = "term_builder_i::src")]
    fn select_src_id(&mut self, id: impl Into<Entity>) -> &mut Self {
        self.setup_src().set_term_ref_id(id)
    }

    /// Select src identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::src`
    #[doc(alias = "term_builder_i::src")]
    fn select_src<T: ComponentId>(&mut self) -> &mut Self {
        self.select_src_id(T::get_id(self.world()))
    }

    /// Select src identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::src`
    #[doc(alias = "term_builder_i::src")]
    fn select_src_name(&mut self, name: &CStr) -> &mut Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_src();
        if let Some(stripped_name) =
            strip_prefix_cstr_raw(name, CStr::from_bytes_with_nul(b"$\0").unwrap())
        //todo v4 fix this
        {
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
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::first`
    #[doc(alias = "term_builder_i::first")]
    fn select_first_id(&mut self, id: impl Into<Entity>) -> &mut Self {
        self.setup_first().set_term_ref_id(id)
    }

    /// Select first identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::first`
    #[doc(alias = "term_builder_i::first")]
    fn select_first<T: ComponentId>(&mut self) -> &mut Self {
        self.select_first_id(T::get_id(self.world()))
    }

    /// Select first identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::first`
    #[doc(alias = "term_builder_i::first")]
    fn select_first_name(&mut self, name: &'static CStr) -> &mut Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_first();
        if let Some(stripped_name) =
            strip_prefix_cstr_raw(name, CStr::from_bytes_with_nul(b"$\0").unwrap())
        {
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
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::second`
    #[doc(alias = "term_builder_i::second")]
    fn select_second_id(&mut self, id: impl Into<Entity>) -> &mut Self {
        self.setup_second().set_term_ref_id(id)
    }

    /// Select second identifier, initialize it with id associated with type
    ///
    /// # Type Arguments
    ///
    /// * `T` - The type to use.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::second`
    #[doc(alias = "term_builder_i::second")]
    fn select_second<T: ComponentId>(&mut self) -> &mut Self {
        self.select_second_id(T::get_id(self.world()))
    }

    /// Select second identifier, initialize it with name. If name starts with a $
    /// the name is interpreted as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::second`
    #[doc(alias = "term_builder_i::second")]
    fn select_second_name(&mut self, name: &CStr) -> &mut Self {
        ecs_assert!(
            !name.is_empty(),
            FlecsErrorCode::InvalidParameter,
            "name is empty"
        );

        self.setup_second();
        if let Some(stripped_name) =
            strip_prefix_cstr_raw(name, CStr::from_bytes_with_nul(b"$\0").unwrap())
        {
            self.var(stripped_name)
        } else {
            self.name(name)
        }
    }

    /// default up where trav is set to 0.
    /// The up flag indicates that the term identifier may be substituted by
    /// traversing a relationship upwards. For example: substitute the identifier
    /// with its parent by traversing the `ChildOf` relationship.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::up`
    #[doc(alias = "term_builder_i::up")]
    #[inline]
    fn up(&mut self) -> &mut Self {
        self.term_ref_mut().id |= ECS_UP;
        self
    }

    /// same as [`up`](crate::core::term)
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::parent`
    #[doc(alias = "term_builder_i::parent")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::up`
    #[doc(alias = "term_builder_i::up")]
    fn up_id(&mut self, traverse_relationship: impl Into<Entity>) -> &mut Self {
        let term_ref = self.term_ref_mut();
        term_ref.id |= ECS_UP;
        self.get_current_term_mut().trav = *traverse_relationship.into();
        self
    }

    /// The up flag indicates that the term identifier may be substituted by
    /// traversing a relationship upwards. For example: substitute the identifier
    /// with its parent by traversing the `ChildOf` relationship.
    ///
    /// # Type Arguments
    ///
    /// * `TravRel` - The relationship to traverse.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::up`
    #[doc(alias = "term_builder_i::up")]
    fn up_type<TravRel: ComponentId>(&mut self) -> &mut Self {
        self.term_ref_mut().id |= ECS_UP;
        self.get_current_term_mut().trav = TravRel::get_id(self.world());
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for `flecs::query`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::cascade`
    #[doc(alias = "term_builder_i::cascade")]
    fn cascade(&mut self) -> &mut Self {
        self.term_ref_mut().id |= ECS_CASCADE;
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for `flecs::query`
    ///
    /// # Arguments
    ///
    /// * `traverse_relationship` - The optional relationship to traverse.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::cascade`
    #[doc(alias = "term_builder_i::cascade")]
    fn cascade_id(&mut self, traverse_relationship: impl Into<Entity>) -> &mut Self {
        //ecs_assert!(
        //    traverse_relationship != 0,
        //    FlecsErrorCode::InvalidOperation,
        //    "Opt the usage of `cascade` if you are passing 0"
        //);
        self.term_ref_mut().id |= ECS_CASCADE;
        self.get_current_term_mut().trav = *traverse_relationship.into();
        self
    }

    /// The cascade flag is like up, but returns results in breadth-first order.
    /// Only supported for `flecs::query`
    ///
    /// # Type Arguments
    ///
    /// * `TravRel` - The relationship to traverse.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::cascade`
    #[doc(alias = "term_builder_i::cascade")]
    fn cascade_type<TravRel: ComponentId>(&mut self) -> &mut Self {
        self.term_ref_mut().id |= ECS_CASCADE;
        self.get_current_term_mut().trav = TravRel::get_id(self.world());
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
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::trav`
    #[doc(alias = "term_builder_i::trav")]
    fn trav(&mut self, traverse_relationship: impl Into<Entity>, flags: u64) -> &mut Self {
        self.get_current_term_mut().trav = *traverse_relationship.into();
        self.term_ref_mut().id |= flags;
        self
    }

    /// Set id flags for term.
    ///
    /// # Arguments
    ///
    /// * `flags` - The direction to traverse.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::id_flags`
    #[doc(alias = "term_builder_i::id_flags")]
    fn id_flags(&mut self, flags: impl IntoId) -> &mut Self {
        self.term_ref_mut().id |= *flags.into();
        self
    }

    /// Set read=write access of term
    ///
    /// # Arguments
    ///
    /// * `inout` - The inout to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::inout`
    #[doc(alias = "term_builder_i::inout")]
    fn set_inout(&mut self, inout: InOutKind) -> &mut Self {
        self.get_current_term_mut().inout = inout.into();
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
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::inout_stage`
    #[doc(alias = "term_builder_i::inout_stage")]
    fn inout_stage(&mut self, inout: InOutKind) -> &mut Self {
        self.set_inout(inout);
        if self.get_current_term_mut().oper != OperKind::Not as i16 {
            self.setup_src().entity(0);
        }

        self
    }

    /// Short for `inout_stage(flecs::Out`.
    ///  Use when system uses add, remove or set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::write`
    #[doc(alias = "term_builder_i::write")]
    #[inline(always)]
    fn write(&mut self) -> &mut Self {
        self.inout_stage(InOutKind::Out)
    }

    /// Short for `inout_stage(flecs::In`.
    /// Use when system uses get
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::read`
    #[doc(alias = "term_builder_i::read")]
    #[inline(always)]
    fn read(&mut self) -> &mut Self {
        self.inout_stage(InOutKind::In)
    }

    /// Short for `inout_stage(flecs::InOut`.
    /// Use when system uses `ensure`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::read_write`
    #[doc(alias = "term_builder_i::read_write")]
    #[inline(always)]
    fn read_write(&mut self) -> &mut Self {
        self.inout_stage(InOutKind::InOut)
    }

    /// short for `inout(flecs::In`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::in`
    #[doc(alias = "term_builder_i::in")]
    #[inline(always)]
    fn set_as_in(&mut self) -> &mut Self {
        self.set_inout(InOutKind::In)
    }

    /// short for `inout(flecs::Out`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::out`
    #[doc(alias = "term_builder_i::out")]
    #[inline(always)]
    fn set_as_out(&mut self) -> &mut Self {
        self.set_inout(InOutKind::Out)
    }

    /// short for `inout(flecs::InOut`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::inout`
    #[doc(alias = "term_builder_i::inout")]
    #[inline(always)]
    fn set_as_inout(&mut self) -> &mut Self {
        self.set_inout(InOutKind::InOut)
    }

    /// short for `inout(flecs::InOutNone`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::inout_none`
    #[doc(alias = "term_builder_i::inout_none")]
    #[inline(always)]
    fn set_as_inout_none(&mut self) -> &mut Self {
        self.set_inout(InOutKind::InOutNone)
    }

    /// set operator of term
    ///
    /// # Arguments
    ///
    /// * `oper` - The operator to set.
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::oper`
    #[doc(alias = "term_builder_i::oper")]
    #[inline(always)]
    fn set_oper(&mut self, oper: OperKind) -> &mut Self {
        self.get_current_term_mut().oper = oper as i16;
        self
    }

    /// short for `oper(flecs::And`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::and`
    #[doc(alias = "term_builder_i::and")]
    #[inline(always)]
    fn and(&mut self) -> &mut Self {
        self.set_oper(OperKind::And)
    }

    /// short for `oper(flecs::Or`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::or`
    #[doc(alias = "term_builder_i::or")]
    #[inline(always)]
    fn or(&mut self) -> &mut Self {
        self.set_oper(OperKind::Or)
    }

    /// short for `oper(flecs::Not`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::not`
    #[doc(alias = "term_builder_i::not")]
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    fn not(&mut self) -> &mut Self {
        self.set_oper(OperKind::Not)
    }

    /// short for `oper(flecs::Optional`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::optional`
    #[doc(alias = "term_builder_i::optional")]
    #[inline(always)]
    fn optional(&mut self) -> &mut Self {
        self.set_oper(OperKind::Optional)
    }

    /// short for `oper(flecs::AndFrom`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::and_from`
    #[doc(alias = "term_builder_i::and_from")]
    #[inline(always)]
    fn and_from(&mut self) -> &mut Self {
        self.set_oper(OperKind::AndFrom)
    }

    /// short for `oper(flecs::OrFrom`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::or_from`
    #[doc(alias = "term_builder_i::or_from")]
    #[inline(always)]
    fn or_from(&mut self) -> &mut Self {
        self.set_oper(OperKind::OrFrom)
    }

    /// short for `oper(flecs::NotFrom`
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::not_from`
    #[doc(alias = "term_builder_i::not_from")]
    #[inline(always)]
    fn not_from(&mut self) -> &mut Self {
        self.set_oper(OperKind::NotFrom)
    }

    /// Match singleton
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::singleton`
    #[doc(alias = "term_builder_i::singleton")]
    fn singleton(&mut self) -> &mut Self {
        ecs_assert!(
            self.get_current_term_mut().id != 0 || self.get_current_term_mut().first.id != 0,
            FlecsErrorCode::InvalidParameter,
            "no component specified for singleton"
        );

        unsafe {
            let sid = if self.get_current_term_mut().id != 0 {
                self.get_current_term_mut().id
            } else {
                self.get_current_term_mut().first.id
            };

            ecs_assert!(sid != 0, FlecsErrorCode::InvalidParameter, "invalid id");

            if !ecs_is_pair(sid) {
                self.get_current_term_mut().src.id = sid;
            } else {
                self.get_current_term_mut().src.id =
                    sys::ecs_get_alive(self.world_ptr_mut(), *ecs_pair_first(sid));
            }
        }
        self
    }

    /// Query terms are not triggered on by observers
    ///
    /// # See also
    ///
    /// * C++ API: `term_builder_i::filter`
    #[doc(alias = "term_builder_i::filter")]
    #[inline(always)]
    fn filter(&mut self) -> &mut Self {
        self.get_current_term_mut().src.id |= InOutKind::InOutFilter as u64;
        self
    }
}
