//! Cached query implementation. Fast to iterate, but slower to create than Filter

use std::os::raw::{c_int, c_void};

use crate::core::internals::*;
use crate::core::*;
use crate::sys;

/// Fast to iterate, but slower to create than Filter
pub struct QueryBuilder<'a, T>
where
    T: Iterable,
{
    pub(crate) desc: sys::ecs_query_desc_t,
    pub(crate) term_builder: TermBuilder,
    world: WorldRef<'a>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> QueryBuilder<'a, T>
where
    T: Iterable,
{
    /// Create a new query builder
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    ///
    /// See also
    ///
    /// * C++ API: `builder::builder`
    #[doc(alias = "builder::builder")]
    pub fn new(world: &'a World) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            world: world.world(),
            term_builder: Default::default(),
            _phantom: std::marker::PhantomData,
        };

        T::populate(&mut obj);

        obj
    }

    /// Create a new query builder with a name
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `name` - The name of the observer
    ///
    /// See also
    ///
    /// * C++ API: `query_builder::query_builder`
    #[doc(alias = "query_builder::query_builder")]
    pub fn new_named(world: &'a World, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let desc = Default::default();

        let mut obj = Self {
            desc,
            term_builder: Default::default(),
            world: world.world(),
            _phantom: std::marker::PhantomData,
        };

        let entity_desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &entity_desc) };

        T::populate(&mut obj);
        obj
    }

    /// Create a new query builder from an existing descriptor
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `desc` - The descriptor to create the observer from
    ///
    /// See also
    ///
    /// * C++ API: `query_builder_i::query_builder_i`
    #[doc(alias = "query_builder_i::query_builder_i")]
    pub(crate) fn new_from_desc(
        world: impl IntoWorld<'a>,
        desc: &mut sys::ecs_query_desc_t,
    ) -> Self {
        let obj = Self {
            desc: *desc,
            term_builder: Default::default(),
            world: world.world(),
            _phantom: std::marker::PhantomData,
        };

        obj
    }

    /// Create a new query builder from an existing descriptor with a term index
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the observer in
    /// * `desc` - The descriptor to create the observer from
    /// * `term_index` - The index of the term to create the observer from
    ///
    /// See also
    ///
    /// * C++ API: `query_builder_i::query_builder_i`
    #[doc(alias = "query_builder_i::query_builder_i")]
    pub(crate) fn new_from_desc_term_index(
        world: &'a World,
        desc: &mut sys::ecs_query_desc_t,
        term_index: i32,
    ) -> Self {
        let mut obj = Self {
            desc: *desc,
            term_builder: TermBuilder {
                current_term_index: term_index,
                next_term_index: term_index,
                expr_count: 0,
                term_ref_mode: TermRefMode::Src,
                str_ptrs_to_free: Vec::new(),
            },
            world: world.world(),
            _phantom: std::marker::PhantomData,
        };

        T::populate(&mut obj);
        obj
    }
}

#[doc(hidden)]
impl<'a, T: Iterable> internals::QueryConfig<'a> for QueryBuilder<'a, T> {
    #[inline(always)]
    fn term_builder(&self) -> &TermBuilder {
        &self.term_builder
    }

    #[inline(always)]
    fn term_builder_mut(&mut self) -> &mut TermBuilder {
        &mut self.term_builder
    }

    #[inline(always)]
    fn query_desc(&self) -> &sys::ecs_query_desc_t {
        &self.desc
    }

    #[inline(always)]
    fn query_desc_mut(&mut self) -> &mut sys::ecs_query_desc_t {
        &mut self.desc
    }
}

impl<'a, T: Iterable> TermBuilderImpl<'a> for QueryBuilder<'a, T> {}

impl<'a, T: Iterable> QueryBuilderImpl<'a> for QueryBuilder<'a, T> {}

impl<'a, T: Iterable> IntoWorld<'a> for QueryBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a, T> Builder<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    type BuiltType = Query<T>;

    /// Build the `observer_builder` into an query
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        let world = self.world;
        let query = Query::<T>::new_from_desc(world, &mut self.desc);
        for string_parts in self.term_builder.str_ptrs_to_free.iter() {
            unsafe {
                String::from_raw_parts(
                    string_parts.ptr as *mut u8,
                    string_parts.len,
                    string_parts.capacity,
                );
            }
        }
        query
    }
}

// This is a raw function pointer type, compatible with C to pass to the desc.
type OrderByFn<T> = extern "C" fn(EntityT, *const T, EntityT, *const T) -> c_int;
// Assuming some imports and definitions from your previous example, and adding the required ones for this example.
type GroupByFn = extern "C" fn(*mut WorldT, *mut TableT, IdT, *mut c_void) -> u64;

/// Functions to build a query using terms.
pub trait QueryBuilderImpl<'a>: TermBuilderImpl<'a> {
    /// set the name of the query-like object
    fn named(&mut self, name: &str) -> &mut Self {
        let name = compact_str::format_compact!("{}\0", name);
        let world_ptr = self.world_ptr_mut();

        let entity_desc: sys::ecs_entity_desc_t = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..std::default::Default::default()
        };
        let entity_field_ref = &mut self.query_desc_mut().entity;
        if *entity_field_ref != 0 {
            unsafe { sys::ecs_delete(world_ptr, *entity_field_ref) };
        }

        *entity_field_ref = unsafe { sys::ecs_entity_init(world_ptr, &entity_desc) };
        self
    }

    /// set itself to be instanced
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::instanced`
    #[doc(alias = "query_builder_i::instanced")]
    fn instanced(&mut self) -> &mut Self {
        self.query_desc_mut().flags |= flecs::query_flags::IsInstanced::ID as u32;
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
    /// * C++ API: `query_builder_i::filter_flags`
    #[doc(alias = "query_builder_i::filter_flags")]
    fn flags(&mut self, flags: sys::ecs_flags32_t) -> &mut Self {
        self.query_desc_mut().flags |= flags;
        self
    }

    /// Set what cache method to use for the query
    ///
    /// # Arguments
    ///
    /// * `kind` - the cache kind to set
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::cache_kind`
    #[doc(alias = "query_builder_i::cache_kind")]
    fn set_cache_kind(&mut self, kind: QueryCacheKind) -> &mut Self {
        self.query_desc_mut().cache_kind = kind as sys::ecs_query_cache_kind_t;
        self
    }

    /// Set the cache method to cached
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::cached`
    #[doc(alias = "query_builder_i::cached")]
    fn set_cached(&mut self) -> &mut Self {
        self.set_cache_kind(QueryCacheKind::Auto)
    }

    /// set expression
    ///
    /// # Arguments
    ///
    /// * `expr` - the expression to set
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::expr`
    #[doc(alias = "query_builder_i::expr")]
    fn expr(&mut self, expr: &'a str) -> &mut Self {
        let expr = format!("{}\0", expr);

        ecs_assert!(
            *self.expr_count_mut() == 0,
            FlecsErrorCode::InvalidOperation,
            "query_builder::expr() called more than once"
        );

        self.query_desc_mut().expr = expr.as_ptr() as *const _;
        *self.expr_count_mut() += 1;
        self.term_builder_mut().str_ptrs_to_free.push(StringToFree {
            ptr: expr.as_ptr() as *mut _,
            len: expr.len(),
            capacity: expr.capacity(),
        });
        std::mem::forget(expr);
        self
    }

    /// set term with Id
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_id(&mut self, id: impl IntoId) -> &mut Self {
        self.term();
        self.init_current_term(id);
        let current_term = self.current_term_mut();
        if current_term.inout == InOutKind::Default as i16 {
            self.set_inout_none();
        }
        self
    }

    /// set term with type
    ///
    /// if T is passed along, inout is set to `inout_none` which indicates
    /// that you are not planning on fetching the component data
    /// for reading or writing purposes use &T or &mut T instead.
    /// you can alternatively use `.set_in()` and `.set_inout()` to set the
    /// inout mode explicitly.
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Position;
    ///
    /// #[derive(Component)]
    /// struct Velocity;
    ///
    /// #[derive(Component)]
    /// struct Mass;
    ///
    /// let world = World::new();
    ///
    /// world.query::<()>()
    ///     //this can be retrieved from it.field if desired
    ///     .with::<Position>().set_inout() //equivalent to .with::<&mut Position>()
    ///     .with::<&Velocity>() //equivalent to .with::<Velocity>().set_in()
    ///     .with::<&mut Mass>() //equivalent to .with::<Mass>().set_inout()
    ///     .build();
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with<T: IntoComponentId>(&mut self) -> &mut Self {
        if <T as IntoComponentId>::IS_PAIR {
            self.with_id(<T as IntoComponentId>::get_id(self.world()));
        } else {
            self.term();
            let world = self.world();
            let id = T::get_id(world);
            self.init_current_term(id);
            if T::First::IS_REF {
                self.set_in();
            } else if T::First::IS_MUT {
                self.set_inout();
            } else {
                self.set_inout_none();
            }
        }
        self
    }

    /// set term with enum
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        let enum_id = T::get_id(self.world());
        // SAFETY: we know that the enum_value is a valid because of the T::get_id call
        let enum_field_id = value.get_id_variant(self.world());
        self.with_id((enum_id, enum_field_id))
    }

    /// set term with enum wildcard
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_enum_wildcard<T: ComponentType<Enum> + ComponentId>(&mut self) -> &mut Self {
        self.with_first::<T>(flecs::Wildcard::ID)
    }

    /// set term with pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_first<First: ComponentId>(&mut self, second: impl Into<Entity> + Copy) -> &mut Self {
        self.with_id((First::get_id(self.world()), second))
    }

    /// set term with pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_first_name<First: ComponentId>(&mut self, second: &'a str) -> &mut Self {
        self.with_first_id(First::get_id(self.world()), second)
    }

    /// set term with pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_second<Second: ComponentId>(&mut self, first: impl Into<Entity> + Copy) -> &mut Self {
        self.with_id((first, Second::get_id(self.world())))
    }

    /// set term with pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_second_name<Second: ComponentId>(&mut self, first: &'a str) -> &mut Self {
        self.with_second_id(first, Second::get_id(self.world()))
    }

    /// set term with Name
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::with")]
    fn with_name(&mut self, name: &'a str) -> &mut Self {
        self.term();
        self.set_first_name(name);
        let term = self.current_term();
        if term.inout == InOutKind::Default as i16 {
            self.set_inout_none();
        }
        self
    }

    /// set term with pair names
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::with")]
    fn with_names(&mut self, first: &'a str, second: &'a str) -> &mut Self {
        self.term();
        self.set_first_name(first).set_second_name(second);
        let term = self.current_term();
        if term.inout == InOutKind::Default as i16 {
            self.set_inout_none();
        }
        self
    }

    /// set term with first id and second name
    fn with_first_id(&mut self, first: impl Into<Entity>, second: &'a str) -> &mut Self {
        self.term();
        self.init_current_term(first.into());
        self.set_second_name(second);
        let term = self.current_term();
        if term.inout == InOutKind::Default as i16 {
            self.set_inout_none();
        }
        self
    }

    /// set term with second id and first name
    fn with_second_id(&mut self, first: &'a str, second: impl Into<Entity>) -> &mut Self {
        self.term();
        self.set_first_name(first).set_second_id(second.into());
        let term = self.current_term();
        if term.inout == InOutKind::Default as i16 {
            self.set_inout_none();
        }
        self
    }

    /* Without methods, shorthand for .with(...).not() */

    /// set term without Id
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_id(&mut self, id: impl IntoId) -> &mut Self {
        self.with_id(id).not()
    }

    /// set term without type
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without<T: IntoComponentId>(&mut self) -> &mut Self {
        self.with::<T>().not()
    }

    /// set term without enum
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.with_enum(value).not()
    }

    /// set term without enum wildcard
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_enum_wildcard<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
    ) -> &mut Self {
        self.with_enum_wildcard::<T>().not()
    }

    /// set term without pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_first<First: ComponentId>(&mut self, second: impl Into<Entity> + Copy) -> &mut Self {
        self.with_first::<First>(second).not()
    }

    /// set term without pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_first_name<First: ComponentId>(&mut self, second: &'a str) -> &mut Self {
        self.with_first_name::<First>(second).not()
    }

    /// set term without pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_second<Second: ComponentId>(
        &mut self,
        first: impl Into<Entity> + Copy,
    ) -> &mut Self {
        self.with_second::<Second>(first).not()
    }

    /// set term without pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_second_name<Second: ComponentId>(&mut self, first: &'a str) -> &mut Self {
        self.with_second_name::<Second>(first).not()
    }

    /// set term without Name
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::without")]
    fn without_name(&mut self, name: &'a str) -> &mut Self {
        self.with_name(name).not()
    }

    /// set term without pair names
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::without")]
    fn without_names(&mut self, first: &'a str, second: &'a str) -> &mut Self {
        self.with_names(first, second).not()
    }

    /// set term without first id and second name
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_first_id(&mut self, first: impl Into<Entity>, second: &'a str) -> &mut Self {
        self.with_first_id(first, second).not()
    }

    /// set term without second id and first name
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_second_id(&mut self, first: &'a str, second: impl Into<Entity>) -> &mut Self {
        self.with_second_id(first, second).not()
    }

    /// set term without term
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    // fn without_term(&mut self, term: Term) -> &mut Self {
    //     self.with_term(term).not()
    // }

    /// Term notation for more complex query features
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::term")]
    fn term(&mut self) -> &mut Self {
        //let index = *self.current_term_index();

        let current_index = self.current_term_index();
        let next_index = self.next_term_index();

        if current_index != next_index {
            *self.current_term_index_mut() = next_index;
        }
        *self.next_term_index_mut() = next_index + 1;

        ecs_assert!(
            current_index < sys::FLECS_TERM_COUNT_MAX as i32,
            FlecsErrorCode::InvalidParameter,
            "Maximum number of terms reached in query builder",
        );

        // let term = &mut self.query_desc_mut().terms[index as usize] as *mut sys::ecs_term_t;

        // self.set_term(term);

        // *self.current_term_index_mut() += 1;

        self
    }

    /// Term notation for more complex query features
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term_at`
    #[doc(alias = "query_builder_i::term_at")]
    fn term_at(&mut self, index: i32) -> &mut Self {
        ecs_assert!(
            index >= 0,
            FlecsErrorCode::InvalidParameter,
            "term_at() called with invalid index"
        );

        self.set_term_ref_mode(TermRefMode::Src);

        *self.current_term_index_mut() = index;

        self
    }

    /// Set the type as current term and in mode inout
    fn write<T: IntoComponentId>(&mut self) -> &mut Self {
        self.with::<T>();
        TermBuilderImpl::write_curr(self)
    }

    /// Set the id as current term and in mode inout
    fn write_id(&mut self, id: impl IntoId) -> &mut Self {
        self.with_id(id);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the name as current term and in mode inout
    fn write_name(&mut self, name: &'a str) -> &mut Self {
        self.with_name(name);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the names as current term and in mode inout
    fn write_names(&mut self, first: &'a str, second: &'a str) -> &mut Self {
        self.with_names(first, second);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the type as current term and in mode inout
    fn write_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.with_enum(value);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the relationship as current term and in mode inout
    fn write_first<T: ComponentId>(&mut self, second: impl Into<Entity> + Copy) -> &mut Self {
        self.with_first::<T>(second);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the relationship as current term and in mode inout
    fn write_first_name<T: ComponentId>(&mut self, second: &'a str) -> &mut Self {
        self.with_first_name::<T>(second);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the relationship as current term and in mode inout
    fn write_second<T: ComponentId>(&mut self, first: impl Into<Entity> + Copy) -> &mut Self {
        self.with_second::<T>(first);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the relationship as current term and in mode inout
    fn write_second_name<T: ComponentId>(&mut self, first: &'a str) -> &mut Self {
        self.with_second_name::<T>(first);
        TermBuilderImpl::write_curr(self)
    }

    /// Set the type as current term and in mode in
    fn read<T: IntoComponentId>(&mut self) -> &mut Self {
        self.with::<T>();
        TermBuilderImpl::read_curr(self)
    }

    /// Set the id as current term and in mode in
    fn read_id(&mut self, id: impl IntoId) -> &mut Self {
        self.with_id(id);
        TermBuilderImpl::read_curr(self)
    }

    /// Set the name as current term and in mode in
    fn read_name(&mut self, name: &'a str) -> &mut Self {
        self.with_name(name);
        TermBuilderImpl::read_curr(self)
    }

    /// Set the names as current term and in mode in
    fn read_names(&mut self, first: &'a str, second: &'a str) -> &mut Self {
        self.with_names(first, second);
        TermBuilderImpl::read_curr(self)
    }

    /// Set the type as current term and in mode in
    fn read_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &mut self,
        value: T,
    ) -> &mut Self {
        self.with_enum(value);
        TermBuilderImpl::read_curr(self)
    }

    /// Set the relationship as current term and in mode in
    fn read_first<T: ComponentId>(&mut self, second: impl Into<Entity> + Copy) -> &mut Self {
        self.with_first::<T>(second);
        TermBuilderImpl::read_curr(self)
    }

    /// Set the relationship as current term and in mode in
    fn read_first_name<T: ComponentId>(&mut self, second: &'a str) -> &mut Self {
        self.with_first_name::<T>(second);
        TermBuilderImpl::read_curr(self)
    }

    /// Set the relationship as current term and in mode in
    fn read_second<T: ComponentId>(&mut self, first: impl Into<Entity> + Copy) -> &mut Self {
        self.with_second::<T>(first);
        TermBuilderImpl::read_curr(self)
    }

    /// Set the relationship as current term and in mode in
    fn read_second_name<T: ComponentId>(&mut self, first: &'a str) -> &mut Self {
        self.with_second_name::<T>(first);
        TermBuilderImpl::read_curr(self)
    }

    /* scope_open/scope_close shorthand notation. */

    /// Open a scope for the query
    fn scope_open(&mut self) -> &mut Self {
        self.with_id(flecs::ScopeOpen::ID).entity(0)
    }

    /// Close a scope for the query
    fn scope_close(&mut self) -> &mut Self {
        self.with_id(flecs::ScopeClose::ID).entity(0)
    }

    /// Sorts the output of a query.
    ///
    /// This enables sorting of entities across matched tables. As a result of this
    /// operation, the order of entities in the matched tables may change.
    /// Resorting occurs when a query iterator is obtained, and only if the table
    /// data has changed.
    ///
    /// If multiple queries that match the same (down)set of tables specify different
    /// sorting functions, resorting is likely to occur every time an iterator is
    /// obtained, potentially slowing down iterations significantly.
    ///
    /// The sorting function will be applied to the specified component. Resorting
    /// only occurs if that component has changed, or when the entity order in the
    /// table changes. If no component is provided, resorting only occurs when
    /// the entity order changes.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component used to sort.
    ///
    /// # Arguments
    ///
    /// * `compare`: The compare function used to sort the components.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::order_by`
    #[doc(alias = "query_builder_i::order_by")]
    fn order_by<T>(&mut self, compare: OrderByFn<T>) -> &mut Self
    where
        T: ComponentId,
    {
        let cmp: sys::ecs_order_by_action_t = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(u64, *const T, u64, *const T) -> i32,
                unsafe extern "C" fn(
                    u64,
                    *const std::ffi::c_void,
                    u64,
                    *const std::ffi::c_void,
                ) -> i32,
            >(compare)
        });
        self.order_by_id(T::get_id(self.world()), cmp);
        self
    }

    /// Sorts the output of a query.
    ///
    /// This is similar to `order_by<T>`, but uses a component identifier instead.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to sort.
    /// * `compare`: The compare function used to sort the components.
    /// # See also
    ///
    /// * C++ API: `query_builder_i::order_by`
    #[doc(alias = "query_builder_i::order_by")]
    fn order_by_id(
        &mut self,
        component: impl Into<Entity>,
        compare: sys::ecs_order_by_action_t,
    ) -> &mut Self {
        let desc = self.query_desc_mut();
        desc.order_by_callback = compare;
        desc.order_by = *component.into();
        self
    }

    /// Group and sort matched tables.
    ///
    /// This function is similar to `group_by<T>`, but uses a default `group_by` action.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component used to determine the group rank.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by`
    #[doc(alias = "query_builder_i::group_by")]
    fn group_by<T>(&mut self) -> &mut Self
    where
        T: ComponentId,
    {
        self.group_by_id_fn(T::get_id(self.world()), None)
    }

    /// Group and sort matched tables.
    ///
    /// This function is similar to `order_by`, but instead of sorting individual entities,
    /// it only sorts matched tables. This can be useful if a query needs to enforce a
    /// certain iteration order upon the tables it is iterating, for example by giving
    /// a certain component or tag a higher priority.
    ///
    /// The sorting function assigns a "rank" to each type, which is then used to sort
    /// the tables. Tables with higher ranks will appear later in the iteration.
    ///
    /// Resorting happens when a query iterator is obtained, and only if the set of
    /// matched tables for a query has changed. If table sorting is enabled together
    /// with entity sorting, table sorting takes precedence, and entities will be sorted
    /// within each set of tables that are assigned the same rank.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component used to determine the group rank.
    ///
    /// # Arguments
    ///
    /// * `group_by_action`: Callback that determines group id for table.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by`
    #[doc(alias = "query_builder_i::group_by")]
    fn group_by_fn<T>(&mut self, group_by_action: sys::ecs_group_by_action_t) -> &mut Self
    where
        T: ComponentId,
    {
        self.group_by_id_fn(T::get_id(self.world()), group_by_action);
        self
    }

    /// Group and sort matched tables.
    ///
    /// This is similar to `group_by<T>`, but uses a component identifier instead.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to determine the group rank.
    /// * `group_by_action`: Callback that determines group id for table.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by`
    #[doc(alias = "query_builder_i::group_by")]
    fn group_by_id_fn(
        &mut self,
        component: impl Into<Entity>,
        group_by_action: sys::ecs_group_by_action_t,
    ) -> &mut Self {
        let desc = self.query_desc_mut();
        desc.group_by_callback = group_by_action;
        desc.group_by = *component.into();
        self
    }

    /// Group and sort matched tables.
    ///
    /// This is similar to `group_by_default<T>`, but uses a component identifier instead.
    ///
    /// # Arguments
    ///
    /// * `component`: The component used to determine the group rank.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by`
    #[doc(alias = "query_builder_i::group_by")]
    fn group_by_id(&mut self, component: impl Into<Entity>) -> &mut Self {
        self.group_by_id_fn(component, None)
    }

    /// Specify context to be passed to the `group_by` function.
    ///
    /// # Arguments
    ///
    /// * `ctx`: Context to pass to the `group_by` function.
    /// * `ctx_free`: Function to clean up the context (called when the query is deleted).
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::group_by_ctx`
    #[doc(alias = "query_builder_i::group_by_ctx")]
    fn group_by_ctx(&mut self, ctx: *mut c_void, ctx_free: sys::ecs_ctx_free_t) -> &mut Self {
        let desc = self.query_desc_mut();
        desc.group_by_ctx = ctx;
        desc.group_by_ctx_free = ctx_free;
        self
    }

    /// Specify the `on_group_create` action.
    ///
    /// # Arguments
    ///
    /// * `action`: The action to execute when a group is created.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::on_group_create`
    #[doc(alias = "query_builder_i::on_group_create")]
    fn on_group_create(&mut self, action: sys::ecs_group_create_action_t) -> &mut Self {
        let desc = self.query_desc_mut();
        desc.on_group_create = action;
        self
    }

    /// Specify the `on_group_delete` action.
    ///
    /// # Arguments
    ///
    /// * `action`: The action to execute when a group is deleted.
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::on_group_delete`
    #[doc(alias = "query_builder_i::on_group_delete")]
    fn on_group_delete(&mut self, action: sys::ecs_group_delete_action_t) -> &mut Self {
        let desc = self.query_desc_mut();
        desc.on_group_delete = action;
        self
    }
}
