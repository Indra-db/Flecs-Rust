//! Queries for iterating over entities that match conditions.
//!
//! Queries enable applications to quickly find entities that match a list of conditions,
//! and are at the core of many Flecs features like systems, observers, tooling,
//! and serialization. Queries can match anything from simple component lists to
//! complex patterns against entity relationship graphs.
//!
//! > 📚 **For comprehensive documentation**, see the [Flecs Query Manual](https://www.flecs.dev/flecs/md_docs_2Queries.html)
//!
//! # Quick Start
//!
//! ```
//! use flecs_ecs::prelude::*;
//!
//! #[derive(Component)]
//! struct Position {
//!     x: f32,
//!     y: f32,
//! }
//!
//! #[derive(Component)]
//! struct Velocity {
//!     x: f32,
//!     y: f32,
//! }
//!
//! let world = World::new();
//!
//! // Create some entities
//! world
//!     .entity()
//!     .set(Position { x: 0.0, y: 0.0 })
//!     .set(Velocity { x: 1.0, y: 2.0 });
//!
//! // Create a query for entities with Position and Velocity
//! let query = world.new_query::<(&mut Position, &Velocity)>();
//!
//! // Iterate and update matching entities
//! query.each(|(pos, vel)| {
//!     pos.x += vel.x;
//!     pos.y += vel.y;
//! });
//! ```
//!
//! # Query Types
//!
//! Flecs supports different query caching strategies optimized for different use cases:
//!
//! ## Cached Queries
//!
//! Cached queries maintain internal data structures that track matching archetypes.
//! They're faster to iterate but slower to create. Queries are cached when associated
//! with an entity (named queries, system queries):
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # #[derive(Component)] struct Velocity { x: f32, y: f32 }
//! # let world = World::new();
//! // Named query - automatically cached
//! let query = world.new_query_named::<(&Position, &Velocity)>("movement");
//!
//! // Or explicitly request caching
//! let query = world
//!     .query::<(&Position, &Velocity)>()
//!     .set_cache_kind(QueryCacheKind::Auto)
//!     .build();
//!
//! // Can iterate multiple times efficiently
//! query.each(|(pos, vel)| { /* ... */ });
//! query.each(|(pos, vel)| { /* ... */ }); // Still fast
//! ```
//!
//! ## Uncached Queries
//!
//! Uncached queries don't maintain cached state. They're faster to create but
//! slower to iterate, ideal for one-off queries or ad-hoc lookups:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # let world = World::new();
//! // Uncached query - fast creation, slower iteration
//! let query = world.new_query::<&Position>();
//!
//! // Or explicitly disable caching
//! let query = world
//!     .query::<&Position>()
//!     .set_cache_kind(QueryCacheKind::None)
//!     .build();
//!
//! // Good for one-time operations
//! query.each(|pos| { /* ... */ });
//! ```
//!
//! ## Default Behavior
//!
//! When using the builder without specifying cache kind:
//! - Queries **with** an entity (named) are cached
//! - Queries **without** an entity are uncached
//! - `new_query()` creates uncached queries
//! - `new_query_named()` creates cached queries
//!
//! # Iteration Methods
//!
//! Queries provide several ways to iterate over matching entities:
//!
//! ## Each Iterator
//!
//! The simplest way to iterate, receiving component data directly:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # #[derive(Component)] struct Velocity { x: f32, y: f32 }
//! # let world = World::new();
//! # world.entity().set(Position { x: 0.0, y: 0.0 }).set(Velocity { x: 1.0, y: 2.0 });
//! # let query = world.new_query::<(&mut Position, &Velocity)>();
//! query.each(|(pos, vel)| {
//!     pos.x += vel.x;
//!     pos.y += vel.y;
//! });
//! ```
//!
//! ## Each with Entity
//!
//! Get both the entity and its components:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # let world = World::new();
//! # world.entity().set(Position { x: 0.0, y: 0.0 });
//! # let query = world.new_query::<&Position>();
//! query.each_entity(|entity, pos| {
//!     println!("Entity {} at ({}, {})", entity.name(), pos.x, pos.y);
//! });
//! ```
//!
//! ## Table Iterator (Run)
//!
//! iterate over entities table-by-table:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # #[derive(Component)] struct Velocity { x: f32, y: f32 }
//! # let world = World::new();
//! # world.entity().set(Position { x: 0.0, y: 0.0 }).set(Velocity { x: 1.0, y: 2.0 });
//! # let query = world.new_query::<(&mut Position, &Velocity)>();
//! query.run(|mut it| {
//!     while it.next() {
//!         let mut pos = it.field_mut::<Position>(0);
//!         let vel = it.field::<Velocity>(1);
//!
//!         for i in it.iter() {
//!             pos[i].x += vel[i].x;
//!             pos[i].y += vel[i].y;
//!         }
//!     }
//! });
//! ```
//!
//! # Advanced Query Features
//!
//! ## Optional Components
//!
//! Match entities with or without certain components:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # #[derive(Component)] struct Health(i32);
//! # let world = World::new();
//! let query = world.new_query::<(&Position, Option<&Health>)>();
//!
//! query.each(|(pos, health)| {
//!     if let Some(health) = health {
//!         println!("Entity at ({}, {}) has {} health", pos.x, pos.y, health.0);
//!     }
//! });
//! ```
//!
//! ## Query Builder
//!
//! For complex queries with additional conditions, use the builder pattern:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # #[derive(Component)] struct Velocity { x: f32, y: f32 }
//! # #[derive(Component)] struct Enemy;
//! # let world = World::new();
//! let query = world
//!     .query::<(&Position, &Velocity)>()
//!     .with(Enemy)
//!     .without(flecs::Prefab)
//!     .build();
//! ```
//!
//! ## Relationship Queries
//!
//! Query entities based on their relationships with other entities:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! #[derive(Component)]
//! struct Eats;
//!
//! # let world = World::new();
//! let apples = world.entity_named("Apples");
//! let burgers = world.entity_named("Burgers");
//!
//! world
//!     .entity_named("Bob")
//!     .add((Eats, apples))
//!     .add((Eats, burgers));
//!
//! // Query for entities that eat apples
//! let query = world.query::<()>().with((Eats, apples)).build();
//!
//! query.each_entity(|entity, _| {
//!     println!("{} eats apples", entity.name());
//! });
//! ```
//!
//! ## Enum Queries (repr(C) Enums)
//!
//! Query entities based on enum values (which are stored as relationships):
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! #[repr(C)]
//! enum Direction {
//!     Up,
//!     Down,
//!     Left,
//!     Right,
//! }
//!
//! # let world = World::new();
//! world.entity().add_enum(Direction::Up);
//! world.entity().add_enum(Direction::Down);
//!
//! // Query for entities with Direction::Up
//! let query = world.query::<()>().with_enum(Direction::Up).build();
//!
//! query.each_entity(|entity, _| {
//!     println!("{} is facing up", entity.name());
//! });
//! ```
//!
//! # Query Creation Methods
//!
//! There are two main ways to create queries:
//!
//! 1. **Direct creation** with `new_query()` - creates an uncached query immediately
//! 2. **Direct creation** with `new_query_named()` - creates a cached query with a name
//! 3. **Builder pattern** with `query()` - returns a builder for complex queries
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # let world = World::new();
//! // Uncached: simple query, good for one-time use
//! let q1 = world.new_query::<&Position>();
//!
//! // Cached: named query, good for repeated use
//! let q2 = world.new_query_named::<&Position>("positions");
//!
//! // Builder: complex query with additional terms
//! let q3 = world.query::<&Position>().with(flecs::Prefab).build();
//! ```
//!
//! # Query Lifetime and Ownership
//!
//! Queries are reference-counted and can be cloned cheaply. They remain valid
//! as long as the world exists:
//!
//! ```
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)] struct Position { x: f32, y: f32 }
//! # let world = World::new();
//! let query = world.new_query::<&Position>();
//! let query_clone = query.clone(); // Cheap, reference counted
//!
//! // Both can be used independently
//! query.each(|pos| { /* ... */ });
//! query_clone.each(|pos| { /* ... */ });
//! ```
//!
//! # Performance Tips
//!
//! - Use **cached queries** (named or explicit) for frequently executed queries (e.g., systems)
//! - Use **uncached queries** (`new_query`) for one-off operations or ad-hoc lookups
//! - Use table iteration (`run()`) when processing sparse components or for better performance for optional terms or you need better control between tables.
//! - Avoid creating queries in hot loops - create once, reuse many times
//! - Order query terms from most to least restrictive for better performance
//! - Monitor query rematching costs when using traversal (see manual)
//!
//! # See Also
//!
//! - [`QueryBuilder`] for building complex queries
//! - [Flecs Query Manual](https://www.flecs.dev/flecs/md_docs_2Queries.html) for comprehensive documentation
//! - [`World::new_query()`] for creating simple queries
//! - [`World::query()`] for creating query builders
//! - [`World::each()`] for quick one-off iterations
//! - [`TableIter`] for low-level table iteration

use core::panic;
use core::{ffi::c_void, marker::PhantomData, ptr::NonNull};

use flecs_ecs_sys::ecs_get_binding_ctx;
use sys::ecs_get_alive;

use crate::core::*;
use crate::sys;

/// Query for iterating over entities that match a set of conditions.
///
/// Queries are the primary way to find and iterate over entities in Flecs. They efficiently
/// match entities based on components, relationships, and other criteria. Queries can be
/// cached for fast repeated iteration or uncached for one-time use.
///
/// > 📚 **For comprehensive documentation**, see the [Flecs Query Manual](https://www.flecs.dev/flecs/md_docs_2Queries.html)
///
/// # Creating Queries
///
/// Queries are created using [`World::new_query()`] for simple cases or [`World::query()`]
/// when you need a builder for complex conditions:
///
/// ```
/// # use flecs_ecs::prelude::*;
/// # #[derive(Component)] struct Position { x: f32, y: f32 }
/// # #[derive(Component)] struct Velocity { x: f32, y: f32 }
/// # let world = World::new();
/// // Simple query - creates cached query immediately
/// let q1 = world.new_query::<(&Position, &Velocity)>();
///
/// // Complex query - use builder for additional conditions
/// let q2 = world
///     .query::<(&Position, &Velocity)>()
///     .without(flecs::Prefab)
///     .build();
/// ```
///
/// # Iteration
///
/// Queries provide multiple iteration methods:
///
/// ```
/// # use flecs_ecs::prelude::*;
/// # #[derive(Component)] struct Position { x: f32, y: f32 }
/// # #[derive(Component)] struct Velocity { x: f32, y: f32 }
/// # let world = World::new();
/// # world.entity().set(Position { x: 0.0, y: 0.0 }).set(Velocity { x: 1.0, y: 2.0 });
/// # let query = world.new_query::<(&mut Position, &Velocity)>();
/// // Simple iteration over components
/// query.each(|(pos, vel)| {
///     pos.x += vel.x;
/// });
///
/// // Iteration with entity access
/// query.each_entity(|entity, (pos, vel)| {
///     println!("{} at ({}, {})", entity.name(), pos.x, pos.y);
/// });
///
/// // Low-level table iteration for maximum performance
/// query.run(|mut it| {
///     while it.next() {
///         let mut pos = it.field_mut::<Position>(0);
///         let vel = it.field::<Velocity>(1);
///         for i in it.iter() {
///             pos[i].x += vel[i].x;
///         }
///     }
/// });
/// ```
///
/// # Cached vs Uncached
///
/// Queries can be cached or uncached:
/// - **Uncached** (created with `new_query()`): Fast creation, slower iteration
/// - **Cached** (created with `new_query_named()` or explicit cache kind): Slower creation, fast iteration
///
/// Use the builder's `set_cache_kind()` to control caching behavior:
///
/// ```
/// # use flecs_ecs::prelude::*;
/// # #[derive(Component)] struct Position { x: f32, y: f32 }
/// # let world = World::new();
/// // Uncached - good for one-off queries
/// let uncached = world.new_query::<&Position>();
///
/// // Cached - good for repeated queries
/// let cached = world.new_query_named::<&Position>("positions");
///
/// // Explicit control
/// let custom = world
///     .query::<&Position>()
///     .set_cache_kind(QueryCacheKind::Auto)
///     .build();
/// ```
///
/// # Lifetime and Ownership
///
/// Queries are reference-counted and can be cloned cheaply. They remain valid as long
/// as the world exists. Attempting to use a query after its world is destroyed will panic.
///
/// ```
/// # use flecs_ecs::prelude::*;
/// # #[derive(Component)] struct Position { x: f32, y: f32 }
/// # let world = World::new();
/// let query = world.new_query::<&Position>();
/// let query_clone = query.clone(); // Cheap - just increments refcount
/// ```
///
/// # Examples
///
/// ## Returning Queries from Functions
///
/// ```
/// # use flecs_ecs::prelude::*;
/// # #[derive(Component)]
/// # struct Foo(u8);
/// // The 'static lifetime ensures components live long enough
/// fn create_foo_query(world: &World) -> Query<&'static Foo> {
///     world.new_query::<&Foo>()
/// }
///
/// # let world = World::new();
/// let query = create_foo_query(&world);
/// query.each(|foo| { /* ... */ });
/// ```
///
/// ## Complex Query Conditions
///
/// ```
/// # use flecs_ecs::prelude::*;
/// # #[derive(Component)] struct Position { x: f32, y: f32 }
/// # #[derive(Component)] struct Velocity { x: f32, y: f32 }
/// # #[derive(Component)] struct Enemy;
/// # let world = World::new();
/// let query = world
///     .query::<(&Position, &Velocity)>()
///     .with(Enemy)
///     .without(flecs::Prefab)
///     .build();
/// ```
///
/// # See Also
///
/// - [`QueryBuilder`] for building complex queries
/// - [`World::new_query()`] for creating simple cached queries
/// - [`World::query()`] for creating query builders
/// - [`World::each()`] for one-off iterations without creating a query
/// - [Module documentation](self) for comprehensive query examples
/// - [Flecs Query Manual](https://www.flecs.dev/flecs/md_docs_2Queries.html)
///
/// [systems]: crate::addons::system
/// [observers]: Observer
/// [tooling]: flecs::rest
pub struct Query<T>
where
    T: QueryTuple,
{
    pub query: NonNull<sys::ecs_query_t>,
    // this is a leaked box, which is valid during the lifecycle of the query object.
    world_ctx: NonNull<WorldCtx>,
    _phantom: PhantomData<T>,
}

unsafe impl<T> Send for Query<T> where T: QueryTuple {}

unsafe impl<T> Sync for Query<T> where T: QueryTuple {}

impl<T> Clone for Query<T>
where
    T: QueryTuple,
{
    fn clone(&self) -> Self {
        unsafe { Query::<T>::new_from(self.query) }
    }
}

impl<T> Drop for Query<T>
where
    T: QueryTuple,
{
    fn drop(&mut self) {
        unsafe {
            // If the world didn't end through normal reasons (user dropping it manually or resetting it)
            // and it's holding remaining references to queries in Rust, the world will panic, in that case, don't invoke
            // the query destruction since the memory will already be invalidated.
            if self.world_ctx.as_ref().is_panicking() {
                return;
            }

            // fn [`destruct`](crate::core::query::destruct) does not decrease the ref count, because it still calls drop.
            self.world().world_ctx_mut().dec_query_ref_count();

            // Only free if query is not associated with entity. Queries are associated with entities
            // when they are either named or cached, such as system, cached queries and named queries. These queries have to be either explicitly
            // deleted with the .destruct() method, or will be deleted when the
            // world is deleted.
            if self.query.as_ref().entity == 0 {
                if sys::flecs_poly_release_(self.query.as_ptr() as *mut c_void) == 0 {
                    sys::ecs_query_fini(self.query.as_ptr());
                }
            }
            // we need to free a poly if the refcount is bigger than 1, this happens when the query is cloned
            else {
                let header = self.query_ptr() as *const sys::ecs_header_t;
                let ref_count_bigger_than_1 = (*header).refcount > 1;
                if ref_count_bigger_than_1 {
                    sys::flecs_poly_release_(self.query.as_ptr() as *mut c_void);
                }
            }
        }
    }
}

impl<T> IterOperations for Query<T>
where
    T: QueryTuple,
{
    #[inline(always)]
    fn retrieve_iter(&self) -> sys::ecs_iter_t {
        unsafe { sys::ecs_query_iter(self.world_ptr(), self.query.as_ptr()) }
    }

    #[inline(always)]
    fn retrieve_iter_stage<'a>(&self, stage: impl WorldProvider<'a>) -> sys::ecs_iter_t {
        unsafe { sys::ecs_query_iter(stage.world_ptr(), self.query.as_ptr()) }
    }

    #[inline(always)]
    fn iter_next(&self, iter: &mut sys::ecs_iter_t) -> bool {
        unsafe { sys::ecs_query_next(iter) }
    }

    #[inline(always)]
    fn query_ptr(&self) -> *const sys::ecs_query_t {
        self.query.as_ptr()
    }

    #[inline(always)]
    fn iter_next_func(&self) -> ExternIterNextFn {
        sys::ecs_query_next
    }
}

impl<T> QueryAPI<'_, (), T> for Query<T>
where
    T: QueryTuple,
{
    #[inline(always)]
    fn entity(&self) -> EntityView<'_> {
        EntityView::new_from(self.world(), unsafe { (*self.query.as_ptr()).entity })
    }
}

impl<'a, T> WorldProvider<'a> for Query<T>
where
    T: QueryTuple,
{
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        unsafe { WorldRef::from_ptr(self.query.as_ref().world) }
    }
}

impl<T> Query<T>
where
    T: QueryTuple,
{
    /// wraps the query pointer in a new query
    ///
    /// # Safety
    ///
    /// this is unsafe due to the fact that the type of the query is not checked.
    /// the user is responsible for ensuring that the query is of the correct type.
    /// if not possible, only use `.iter` functions that do not pass in the components in the callback
    ///
    /// # Arguments
    ///
    /// * `query` - The query pointer to wrap
    #[inline]
    pub unsafe fn new_from(query: NonNull<sys::ecs_query_t>) -> Self {
        unsafe {
            sys::flecs_poly_claim_(query.as_ptr() as *mut c_void);

            let world_ctx = ecs_get_binding_ctx((*query.as_ptr()).world) as *mut WorldCtx;
            (*world_ctx).inc_query_ref_count();
            let world_ctx = NonNull::new_unchecked(world_ctx);

            Self {
                query,
                world_ctx,
                _phantom: core::marker::PhantomData,
            }
        }
    }

    /// Create a new query from a query descriptor
    ///
    /// # Panics
    ///
    /// Panics if the query desc is faulty, such as a wrong name of a non-existent components being passed with `with_name`.
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the query in
    /// * `desc` - The query descriptor to create the query from
    pub(crate) fn new_from_desc<'a>(
        world: impl WorldProvider<'a>,
        desc: &mut sys::ecs_query_desc_t,
    ) -> Self {
        if desc.entity != 0 && desc.terms[0].id == 0 {
            let world_ptr = world.world_ptr();
            let query_poly = unsafe {
                sys::ecs_get_id(
                    world_ptr,
                    desc.entity,
                    ecs_pair(flecs::Poly::ID, flecs::Query::ID),
                )
            } as *const flecs::Poly;

            if !query_poly.is_null() {
                unsafe {
                    let query = NonNull::new_unchecked((*query_poly).poly as *mut sys::ecs_query_t);
                    sys::flecs_poly_claim_(query.as_ptr() as *mut c_void);
                    let world_ctx = ecs_get_binding_ctx((*query.as_ptr()).world) as *mut WorldCtx;
                    (*world_ctx).inc_query_ref_count();
                    let world_ctx = NonNull::new_unchecked(world_ctx);

                    return Self {
                        query,
                        world_ctx,
                        _phantom: PhantomData,
                    };
                }
            }
        }
        let world_ptr = world.world_ptr_mut();

        let query_ptr = unsafe { sys::ecs_query_init(world_ptr, desc) };

        if query_ptr.is_null() {
            panic!(
                "Failed to create query, this is due to the user creating an invalid query. Most likely by using `expr` with a wrong expression."
            );
        }

        unsafe {
            let world_ctx = ecs_get_binding_ctx(world_ptr) as *mut WorldCtx;
            (*world_ctx).inc_query_ref_count();
            let world_ctx = NonNull::new_unchecked(world_ctx);

            let query = NonNull::new_unchecked(query_ptr);

            Self {
                query,
                world_ctx,
                _phantom: PhantomData,
            }
        }
    }

    pub(crate) fn new_from_entity<'a>(
        world: impl WorldProvider<'a>,
        entity: impl Into<Entity>,
    ) -> Option<Query<()>> {
        let world_ptr = world.world_ptr();
        let entity = *entity.into();

        unsafe {
            if ecs_get_alive(world_ptr, entity) != 0 {
                let query_poly = sys::ecs_get_id(
                    world_ptr,
                    entity,
                    ecs_pair(flecs::Poly::ID, flecs::Query::ID),
                );

                if !query_poly.is_null() {
                    let mut desc = sys::ecs_query_desc_t {
                        entity,
                        ..Default::default()
                    };

                    let new_query = Query::<()>::new_from_desc(world, &mut desc);
                    return Some(new_query);
                }
            }
            None
        }
    }

    /// Destroy a query and its resources.
    ///
    /// If the query is used as the parent of subqueries, those subqueries will be
    /// orphaned and must be deinitialized as well.
    pub fn destruct(self) {
        ecs_assert!(
            unsafe { (*self.query.as_ptr()).entity } != 0,
            "destruct() should only be called on queries associated with entities"
        );

        if unsafe { (*self.query.as_ptr()).entity } != 0 {
            let world = self.world();
            let world_ctx = world.world_ctx_mut();
            if unsafe { sys::flecs_poly_release_(self.query.as_ptr() as *mut c_void) } > 0 {
                world_ctx.set_is_panicking_true();
                unsafe { sys::ecs_query_fini(self.query.as_ptr()) };
                panic!("The code base still has lingering references to `Query` objects. This is a bug in the user code. 
                Please ensure that all `Query` objects are out of scope that are a clone/copy of the current one.");
            } else {
                unsafe { sys::ecs_query_fini(self.query.as_ptr()) };
            }
        }
    }

    pub fn reference_count(&self) -> i32 {
        unsafe { sys::flecs_poly_refcount(self.query.as_ptr() as *mut c_void) }
    }

    /// Get the iterator for the query
    ///
    /// # Arguments
    ///
    /// * `world` - The world to get the iterator for
    pub fn get_iter_raw(&mut self) -> sys::ecs_iter_t {
        unsafe { sys::ecs_query_iter(self.world_ptr(), self.query.as_ptr()) }
    }

    /// Returns whether the query data changed since the last iteration.
    ///
    /// This operation must be invoked before obtaining the iterator, as this will
    /// reset the changed state.
    ///
    /// # Returns
    ///
    /// The operation will return `true` after:
    /// - new entities have been matched with
    /// - matched entities were deleted
    /// - matched components were changed
    ///
    /// Otherwise, it will return `false`.
    ///
    /// # See also
    ///
    /// * [`TableIter::is_changed()`]
    pub fn is_changed(&self) -> bool {
        unsafe { sys::ecs_query_changed(self.query.as_ptr()) }
    }

    /// Get info for group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The group id to get info for
    ///
    /// # Returns
    ///
    /// Returns a pointer to the group info
    pub fn group_info(&self, group_id: impl Into<Entity>) -> *const sys::ecs_query_group_info_t {
        unsafe { sys::ecs_query_get_group_info(self.query.as_ptr(), *group_id.into()) }
    }

    /// Get context for group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The group id to get context for
    ///
    /// # Returns
    ///
    /// Returns a (void) pointer to the group context
    pub fn group_context(&self, group_id: impl Into<Entity>) -> *mut c_void {
        let group_info = self.group_info(group_id);

        if !group_info.is_null() {
            unsafe { (*group_info).ctx }
        } else {
            core::ptr::null_mut()
        }
    }
}

impl<T: QueryTuple> From<&Query<T>> for NonNull<sys::ecs_query_t> {
    #[inline]
    fn from(q: &Query<T>) -> Self {
        q.query
    }
}
