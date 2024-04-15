//! Cached query implementation. Fast to iterate, but slower to create than Filter

use std::{
    ffi::CStr,
    ops::Deref,
    os::raw::{c_int, c_void},
};

use crate::core::*;
use crate::sys;

/// Fast to iterate, but slower to create than Filter
pub struct QueryBuilder<'a, T>
where
    T: Iterable,
{
    pub(crate) desc: sys::ecs_query_desc_t,
    pub(crate) term: Term<'a>,
    world: WorldRef<'a>,
    pub(crate) expr_count: i32,
    pub(crate) next_term_index: i32,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> Deref for QueryBuilder<'a, T>
where
    T: Iterable,
{
    type Target = Term<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.term
    }
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
        let desc = Default::default();

        let mut obj = Self {
            desc,
            expr_count: 0,
            term: Term::new_world_only(world.world()),
            world: world.world(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };

        //let entity_desc = sys::ecs_entity_desc_t {
        //    name: std::ptr::null(),
        //    sep: SEPARATOR.as_ptr(),
        //    root_sep: SEPARATOR.as_ptr(),
        //    ..Default::default()
        //};

        //obj.desc.entity = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &entity_desc) };
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
    pub fn new_named(world: &'a World, name: &CStr) -> Self {
        let mut desc = Default::default();

        let mut obj = Self {
            desc,
            expr_count: 0,
            term: Term::new_world_only(world.world()),
            world: world.world(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        };

        let entity_desc = sys::ecs_entity_desc_t {
            name: name.as_ptr(),
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
    pub fn new_from_desc(world: impl IntoWorld<'a>, desc: &mut sys::ecs_query_desc_t) -> Self {
        Self {
            desc: *desc,
            expr_count: 0,
            term: Term::new_world_only(world.world()),
            world: world.world(),
            next_term_index: 0,
            _phantom: std::marker::PhantomData,
        }
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
    pub fn new_from_desc_term_index(
        world: &'a World,
        desc: &mut sys::ecs_query_desc_t,
        term_index: i32,
    ) -> Self {
        let mut obj = Self {
            desc: *desc,
            expr_count: 0,
            term: Term::new_world_only(world.world()),
            world: world.world(),
            next_term_index: term_index,
            _phantom: std::marker::PhantomData,
        };

        let entity_desc = sys::ecs_entity_desc_t {
            name: std::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &entity_desc) };
        T::populate(&mut obj);
        obj
    }
}

impl<'a, T> Filterable<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    fn current_term(&mut self) -> &mut TermT {
        unsafe { &mut *self.term.term_ptr }
    }

    fn next_term(&mut self) {
        self.next_term_index += 1;
    }
}

impl<'a, T> TermBuilder<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn term_mut(&mut self) -> &mut Term<'a> {
        &mut self.term
    }

    #[inline]
    fn term_ptr_mut(&mut self) -> *mut TermT {
        self.term.term_ptr
    }

    #[inline]
    fn term_id_ptr_mut(&mut self) -> *mut super::c_types::TermRefT {
        self.term.term_id_ptr
    }
}

impl<'a, T> Builder<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    type BuiltType = Query<'a, T>;

    /// Build the `observer_builder` into an query
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        let world = self.world;
        Query::<T>::new_from_desc(world, &mut self.desc)
    }
}

// This is a raw function pointer type, compatible with C to pass to the desc.
type OrderByFn<T> = extern "C" fn(EntityT, *const T, EntityT, *const T) -> c_int;
// Assuming some imports and definitions from your previous example, and adding the required ones for this example.
type GroupByFn = extern "C" fn(*mut WorldT, *mut TableT, IdT, *mut c_void) -> u64;

pub trait QueryBuilderImpl<'a>: TermBuilder<'a> {
    fn desc_mut(&mut self) -> &mut sys::ecs_query_desc_t;

    fn expr_count_mut(&mut self) -> &mut i32;

    fn term_index_mut(&mut self) -> &mut i32;

    /// set itself to be instanced
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::instanced`
    #[doc(alias = "query_builder_i::instanced")]
    fn instanced(&mut self) -> &mut Self {
        self.desc_mut().flags |= flecs::query_flags::IsInstanced::ID as u32;
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
        self.desc_mut().flags |= flags;
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
        self.desc_mut().cache_kind = kind as u32;
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
    fn expr(&mut self, expr: &CStr) -> &mut Self {
        ecs_assert!(
            *self.expr_count_mut() == 0,
            FlecsErrorCode::InvalidOperation,
            "query_builder::expr() called more than once"
        );

        self.desc_mut().expr = expr.as_ptr();
        *self.expr_count_mut() += 1;
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
        let term_ptr = self.term_ptr_mut();
        unsafe {
            *term_ptr = *Term::new_id(self.world(), id).term_ptr;
            if (*term_ptr).inout == InOutKind::InOutDefault as i16 {
                self.inout_none();
            }
        }

        self
    }

    /// set term with type
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with<T: InOutType>(&mut self) -> &mut Self {
        if <T::Type as IntoComponentId>::IS_PAIR {
            self.with_id(<T::Type as IntoComponentId>::get_id(self.world()));
        } else {
            self.term();
            let world = self.world();
            let term_ptr = self.term_ptr_mut();
            unsafe {
                *term_ptr = *Term::new_id(world, T::Type::get_id(world)).term_ptr;
                (*term_ptr).inout = type_to_inout::<T>() as i16;
                if (*term_ptr).inout == InOutKind::InOutDefault as i16 {
                    self.inout_none();
                }
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
    fn with_enum_wildcard<T: ComponentType<Enum> + CachedEnumData + InOutType>(
        &mut self,
    ) -> &mut Self {
        self.with_first::<T>(flecs::Wildcard::ID)
    }

    /// set term with pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_first<First: InOutType>(&mut self, second: impl Into<Entity>) -> &mut Self {
        self.with_id((First::Type::get_id(self.world()), second))
    }

    /// set term with pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_first_name<First: InOutType>(&mut self, second: &'static CStr) -> &mut Self {
        self.with_first_id(First::Type::get_id(self.world()), second)
    }

    /// set term with pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_second<Second: InOutType>(&mut self, first: impl Into<Entity>) -> &mut Self {
        self.with_id((first, Second::Type::get_id(self.world())))
    }

    /// set term with pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    #[doc(alias = "query_builder_i::with")]
    fn with_second_name<Second: InOutType>(&mut self, first: &'static CStr) -> &mut Self {
        self.with_second_id(first, Second::Type::get_id(self.world()))
    }

    /// set term with Name
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::with")]
    fn with_name(&mut self, name: &'static CStr) -> &mut Self {
        self.term();
        let term_ptr = self.term_ptr_mut();
        unsafe {
            *term_ptr = *Term::new_world_only(self.world())
                .select_first_name(name)
                .term_ptr;
            if (*term_ptr).inout == InOutKind::InOutDefault as i16 {
                self.inout_none();
            }
        }
        self
    }

    /// set term with pair names
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::with")]
    fn with_names(&mut self, first: &'static CStr, second: &'static CStr) -> &mut Self {
        self.term();
        let term_ptr = self.term_ptr_mut();
        unsafe {
            *term_ptr = *Term::new_world_only(self.world())
                .select_first_name(first)
                .select_second_name(second)
                .term_ptr;
            if (*term_ptr).inout == InOutKind::InOutDefault as i16 {
                self.inout_none();
            }
        }
        self
    }

    /// set term with first id and second name
    fn with_first_id(&mut self, first: impl Into<Entity>, second: &'static CStr) -> &mut Self {
        self.term();
        let term_ptr = self.term_ptr_mut();
        unsafe {
            *term_ptr = *Term::new_id(self.world(), first.into())
                .select_second_name(second)
                .term_ptr;
            if (*term_ptr).inout == InOutKind::InOutDefault as i16 {
                self.inout_none();
            }
        }
        self
    }

    fn with_second_id(&mut self, first: &'static CStr, second: impl Into<Entity>) -> &mut Self {
        self.term();
        let term_ptr = self.term_ptr_mut();
        unsafe {
            *term_ptr = *Term::new_world_only(self.world())
                .select_first_name(first)
                .select_second_id(second.into())
                .term_ptr;
            if (*term_ptr).inout == InOutKind::InOutDefault as i16 {
                self.inout_none();
            }
        }
        self
    }

    fn with_term(&mut self, term: Term) -> &mut Self {
        self.term();
        let term_ptr = self.term_ptr_mut();
        unsafe {
            *term_ptr = *term.term_ptr;
        }
        self
    }

    /// set term
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::with`
    // #[doc(alias = "query_builder_i::with")]
    // fn with(&mut self, with: With) -> &mut Self {
    //     // match with {
    //     //     With::Id(id) => {
    //     //         let term = self.term_with_id(id);
    //     //         if term.term.inout
    //     //     },
    //     //     With::Name(name) => self.term_with_name(name).inout_none(),
    //     //     With::PairIds(first, second) => self.term_with_id((first, second)).inout_none(),
    //     //     With::PairNames(first, second) => self.term_with_pair_names(first, second).inout_none(),
    //     //     With::PairIdName(first, second) => {
    //     //         self.term_with_pair_id_name(first, second).inout_none()
    //     //     }
    //     //     With::Term(term) => self.term_with_term(term).inout_none(),
    //     // }
    // }

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
    fn without<T: InOutType>(&mut self) -> &mut Self {
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
    fn without_enum_wildcard<T: InOutType + ComponentType<Enum> + CachedEnumData>(
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
    fn without_first<First: InOutType>(&mut self, second: impl Into<Entity>) -> &mut Self {
        self.with_first::<First>(second).not()
    }

    /// set term without pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_first_name<First: InOutType>(&mut self, second: &'static CStr) -> &mut Self {
        self.with_first_name::<First>(second).not()
    }

    /// set term without pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_second<Second: InOutType>(&mut self, first: impl Into<Entity>) -> &mut Self {
        self.with_second::<Second>(first).not()
    }

    /// set term without pairs
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_second_name<Second: InOutType>(&mut self, first: &'static CStr) -> &mut Self {
        self.with_second_name::<Second>(first).not()
    }

    /// set term without Name
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::without")]
    fn without_name(&mut self, name: &'static CStr) -> &mut Self {
        self.with_name(name).not()
    }

    /// set term without pair names
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::without")]
    fn without_names(&mut self, first: &'static CStr, second: &'static CStr) -> &mut Self {
        self.with_names(first, second).not()
    }

    /// set term without first id and second name
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_first_id(&mut self, first: impl Into<Entity>, second: &'static CStr) -> &mut Self {
        self.with_first_id(first, second).not()
    }

    /// set term without second id and first name
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    #[doc(alias = "query_builder_i::without")]
    fn without_second_id(&mut self, first: &'static CStr, second: impl Into<Entity>) -> &mut Self {
        self.with_second_id(first, second).not()
    }

    /// set term without term
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::without`
    fn without_term(&mut self, term: Term) -> &mut Self {
        self.with_term(term).not()
    }

    /// Term notation for more complex query features
    ///
    /// # See also
    ///
    /// * C++ API: `query_builder_i::term`
    #[doc(alias = "query_builder_i::term")]
    fn term(&mut self) -> &mut Self {
        let index = *self.term_index_mut();
        // ecs_assert!(
        //     if !self.term_ptr_mut().is_null() {
        //         unsafe { sys::ecs_term_is_initialized(self.term_ptr_mut()) }
        //     } else {
        //         true
        //     },
        //     FlecsErrorCode::InvalidOperation,
        //     "QueryBuilder::term() called without initializing term"
        // );

        ecs_assert!(
            index < sys::FLECS_TERM_COUNT_MAX as i32,
            FlecsErrorCode::InvalidParameter,
            "Maximum number of terms reached in query builder",
        );

        let term = &mut self.desc_mut().terms[index as usize] as *mut sys::ecs_term_t;

        self.set_term(term);

        *self.term_index_mut() += 1;

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

        let term_index = *self.term_index_mut();
        let prev_index = term_index;

        *self.term_index_mut() = index;
        self.term();

        *self.term_index_mut() = prev_index;

        ecs_assert!(
            unsafe { sys::ecs_term_is_initialized(self.term_ptr_mut()) },
            FlecsErrorCode::InvalidOperation,
            "term_at() called without initializing term"
        );

        self
    }

    fn write(&mut self) -> &mut Self {
        self.term_mut().write_();
        self
    }

    fn write_type<T: InOutType>(&mut self) -> &mut Self {
        self.with::<T>();
        QueryBuilderImpl::write(self)
    }

    fn write_id(&mut self, id: impl IntoId) -> &mut Self {
        self.with_id(id);
        QueryBuilderImpl::write(self)
    }

    fn read(&mut self) -> &mut Self {
        self.term_mut().read();
        self
    }

    fn read_type<T: InOutType>(&mut self) -> &mut Self {
        self.with::<T>();
        QueryBuilderImpl::read(self)
    }

    fn read_id(&mut self, id: impl IntoId) -> &mut Self {
        self.with_id(id);
        QueryBuilderImpl::read(self)
    }

    /* scope_open/scope_close shorthand notation. */

    fn scope_open(&mut self) -> &mut Self {
        self.with_id(flecs::ScopeOpen::ID).entity(0)
    }

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
        let cmp: sys::ecs_order_by_action_t = Some(unsafe { std::mem::transmute(compare) });
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
        let desc = self.desc_mut();
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
        let desc = self.desc_mut();
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
        let desc = self.desc_mut();
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
        let desc = self.desc_mut();
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
        let desc = self.desc_mut();
        desc.on_group_delete = action;
        self
    }
}
impl<'a, T> QueryBuilderImpl<'a> for QueryBuilder<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn desc_mut(&mut self) -> &mut sys::ecs_query_desc_t {
        &mut self.desc
    }

    fn expr_count_mut(&mut self) -> &mut i32 {
        &mut self.expr_count
    }

    fn term_index_mut(&mut self) -> &mut i32 {
        &mut self.next_term_index
    }
}

impl<'a, T: Iterable> IntoWorld<'a> for QueryBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.term.world()
    }
}
