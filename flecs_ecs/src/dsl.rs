//! # Flecs Rust DSL
//!
//! ## Introduction
//! The Flecs Rust DSL is a compile-time translation of the Flecs Query Language into Rust procedural macros. The DSL makes it easy to create queries, systems, and observers using a declarative syntax that closely mirrors the Flecs Query Language.
//!
//! This manual is primarily focused on describing the DSL syntax and usage. For more details on specific query features, see the [Flecs Query Language](https://www.flecs.dev/flecs/md_docs_2FlecsQueryLanguage.html).
//!
//! ## Example
//! ```rust
//! # use flecs_ecs::prelude::*;
//!
//! # #[derive(Component)]
//! # struct SpaceShip;
//!
//! # #[derive(Component)]
//! # struct Planet;
//!
//! # #[derive(Component)]
//! # struct DockedTo;
//!
//! # let world = World::new();
//! // Match spaceship entities that are docked to a planet
//! query!(world, SpaceShip, (DockedTo, $"planet"), Planet($"planet"));
//! ```
//!
//! ## The Basics
//! An expression in the Flecs Rust DSL consists of comma-separated "terms", where each term is a condition that entities matching the query must satisfy.
//!
//! ### Components
//! The most basic kind of condition is "the entity must have this component". Components can be accessed with different modes:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! // Match entities that have Position and Velocity (read-only)
//! query!(world, &Position, &Velocity);
//! ```
//!
//! The following table shows the different access modes:
//!
//! | Syntax | Description |
//! |--------|-------------|
//! | `&Component` | Read-only access (immutable borrow) |
//! | `&mut Component` | Read-write access (mutable borrow) |
//! | `Component` | Filter-only (no data access) |
//!
//! Example:
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! // Read-only access
//! let q = query!(world, &Position);
//!
//! // Mutable access
//! let q = query!(world, &mut Position);
//!
//! // Filter-only (match but don't fetch data)
//! let q = query!(world, Position);
//! ```
//!
//! Note that a query matches all entities that _at least_ have the specified components; entities with components in addition to those specified will also match the query.
//!
//! **Multiple components:**
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! // Both Position and Velocity (read-only)
//! query!(world, &Position, &Velocity);
//!
//! // Mixed mutability
//! query!(world, &mut Position, &Velocity);
//!
//! // All mutable
//! query!(world, &mut Position, &mut Velocity);
//! ```
//!
//! ### Pairs
//! Pairs represent relationships between entities. The following expression is an example of a query that matches two pairs:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! # #[derive(Component)]
//! # struct Bob;
//! # #[derive(Component)]
//! # struct Apples;
//! // Match entities that have (Likes, Bob) and (Eats, Apples)
//! query!(world, (Likes, Bob), (Eats, Apples));
//! ```
//!
//! Pairs can be combined with components:
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Parent;
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! // Match entities with Position and a ChildOf relationship
//! query!(world, &Position, (flecs::ChildOf, Parent));
//! ```
//!
//!
//! ## Operators
//! Query operators change how a term is matched against entities. Only a single operator can be applied to a term at a time. The following sections go over the different operators in the DSL.
//!
//! ### Not
//! The `not` operator, specified with the `!` character, inverts the result of a term and makes it possible to match entities that _do not_ satisfy a condition. The following expression is an example of a query with a `not` operator:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! // Match entities that have Position but not Velocity
//! query!(world, &Position, !Velocity);
//! ```
//!
//! The not operator can only be applied to filter terms (without `&` or `&mut`):
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Dead;
//! // Multiple exclusions
//! query!(world, &Position, !Velocity, !Dead);
//!
//! // Not with pairs
//! query!(world, &Position, !(flecs::ChildOf, *));
//! ```
//!
//! ### Or
//! The `or` operator, specified with the `||` character, makes it possible to chain together a list of terms of which at least one term must be true. The following expression is an example of a query with an `or` operator:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Mass { value: f32 }
//! // Match entities that have Position and Velocity or Mass
//! query!(world, &Position, Velocity || Mass);
//! ```
//!
//! ### Optional
//! The `optional` operator, specified with the `?` character, optionally matches a component. Optional terms do not change which entities are matched by a query, but can be useful for various reasons:
//!
//! - Fetching a component in a query is more efficient than `get`
//! - It allows for a single query to do what would otherwise have to be split up across several queries
//!
//! The following expression is an example of a query with an `optional` operator:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! // Match entities that have Position and optionally Velocity
//! query!(world, &Position, ?&Velocity);
//! ```
//!
//! Optional components return `Option<&T>` or `Option<&mut T>` during iteration:
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component,Debug)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component,Debug)]
//! # struct Velocity { x: f32, y: f32 }
//! query!(world, &Position, ?&Velocity).build().each(|(pos, vel_opt)| {
//!     if let Some(vel) = vel_opt {
//!         // Entity has velocity
//!         println!("Moving entity at {pos:?} with velocity {vel:?}");
//!     } else {
//!         // Entity has no velocity
//!         println!("Static entity at {pos:?}");
//!     }
//! });
//! ```
//!
//! Multiple optional components:
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Acceleration { x: f32, y: f32 }
//! // Multiple optional
//! query!(world, &Position, ?&Velocity, ?&Acceleration);
//!
//! // All optional
//! query!(world, ?&Position, ?&Velocity);
//!
//! // Optional with mutable access
//! query!(world, &Position, ?&mut Velocity);
//! ```
//!
//! ### AndFrom
//! The `andfrom` operator allows a query to match a list of components that another entity has. The entities used for the component list are typically prefabs, as they are not matched with queries themselves.
//!
//! The following expression is an example of a query with an `andfrom` operator:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct MyType { value: u8 }
//! // Match entities with Position and all components that MyType has
//! query!(world, Position, and | MyType);
//! ```
//!
//! ### NotFrom
//! The `notfrom` operator allows a query to not match a list of components that another entity has. The entities used for the component list are typically prefabs, as they are not matched with queries themselves.
//!
//! The following expression is an example of a query with a `notfrom` operator:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct MyType { value: u8 }
//! // Match entities with Position and not any of the components that MyType has
//! query!(world, Position, not | MyType);
//! ```
//!
//! ### OrFrom
//! The `orfrom` operator allows a query to match at least one of a list of components that another entity has. The entities used for the component list are typically prefabs, as they are not matched with queries themselves.
//!
//! The following expression is an example of a query with an `orfrom` operator:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct MyType { value: u8 }
//! // Match entities with Position and at least one of the components that MyType has
//! query!(world, Position, or | MyType);
//! ```
//!
//! ## Access Modifiers
//! The following access modifiers can be added to terms to specify how term data is accessed:
//!
//! | Modifier | Description |
//! | -------- | ----------- |
//! | `[in]`   | Matched component is read-only |
//! | `[out]`  | Matched component is read-write |
//! | `[inout]` | Matched component is read-write |
//! | `[none]`  | Matched component is not accessed |
//! | `[filter]` | Term does not produce events (for use with observers mostly) |
//!
//! An example:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! query!(world, [inout] Position, [in] Velocity);
//! ```
//!
//! When no access modifier is specified, the term defaults to `InOutDefault`. This defaults to `[inout]` for terms that match owned components, and `[in]` for terms that match shared components (like singleton terms or terms matched through `up` traversal).
//!
//! ## Wildcards
//! Query expressions can use wildcards to match any or all instances of a matching component or pair. Wildcards may appear in all parts of a term. The following examples are all valid wildcard queries:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Dogs;
//! query!(world, &Position, (Likes, *));
//! query!(world, &Position, (*, Dogs));
//! query!(world, &Position, (*, *));
//! ```
//!
//! Query results contain information about the exact component or pair that was matched with the [`crate::core::TableIter::id()`] function. This allows an application to inspect what was actually matched by a wildcard.
//!
//! ### Wildcard wildcard (*)
//! The following expression is an example of a query that uses a wildcard:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! // Match entities with a (Likes, *) pair
//! // Return all matching pairs
//! query!(world, (Likes, *));
//! ```
//!
//! The `*` wildcard returns all matching instances of the wildcard. If an entity has both `(Likes, Dogs)` and `(Likes, Cats)`, it will be returned twice by the query, once for each pair.
//!
//! If a query has multiple wildcards, each permutation of the matched results will be returned. The following expression is an example of a query that has multiple wildcards:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! // Match entities with (Likes, *) and (Eats, *) pairs
//! // Return all pair permutations
//! query!(world, (Likes, *), (Eats, *));
//! ```
//!
//! If a single entity has `(Likes, Dogs)` and `(Likes, Cats)`, and has `(Eats, Pizza)` and `(Eats, Salad)`, that entity will yield four results:
//!
//! - `(Likes, Dogs)`, `(Eats, Pizza)`
//! - `(Likes, Dogs)`, `(Eats, Salad)`
//! - `(Likes, Cats)`, `(Eats, Pizza)`
//! - `(Likes, Cats)`, `(Eats, Salad)`
//!
//! ### Any wildcard (_)
//! The `any` (`_`) wildcard returns at most one result per wildcard. The following expression is an example of a query that uses an `any` wildcard:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! // Match entities with a (Likes, *) pair
//! // Return at most one result per entity
//! query!(world, (Likes, _));
//! ```
//!
//! If an entity has both `(Likes, Dogs)` and `(Likes, Cats)`, the query will return only one result. The location of the `any` wildcard in the matched id will be replaced with `*`, indicating that no specific pair was matched. The above query would return the following id:
//!
//! - `(Likes, *)`
//!
//! If a query has multiple `any` wildcards, only a single result is returned. The following expression is an example of a query that has multiple wildcards:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! // Match entities with (Likes, *) and (Eats, *) pairs
//! // Return at most one result per entity
//! query!(world, (Likes, _), (Eats, _));
//! ```
//!
//! If a single entity has `(Likes, Dogs)` and `(Likes, Cats)`, and has `(Eats, Pizza)` and `(Eats, Salad)`, that entity will yield one result:
//!
//! - `(Likes, *)`, `(Eats, *)`
//!
//! ## Variables
//! Query variables constrain which values a wildcard can assume by ensuring that the value that was matched by a wildcard in one term is used in all other terms. The following expression is an example of a query that uses variables:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! // Match all entities that eat what they like
//! query!(world, (Likes, $"food"), (Eats, $"food"));
//! ```
//!
//! If a single entity has `(Likes, Dogs)` and `(Likes, Pizza)`, and has `(Eats, Pizza)` and `(Eats, Salad)`, that entity will yield only one result:
//!
//! - `(Likes, Pizza)`, `(Eats, Pizza)`
//!
//! Note how this is a strict subset of the results that would be returned by the following query:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! query!(world, (Likes, *), (Eats, *));
//! ```
//!
//! Which would return:
//!
//! - `(Likes, Dogs)`, `(Eats, Pizza)`
//! - `(Likes, Dogs)`, `(Eats, Salad)`
//! - `(Likes, Pizza)`, `(Eats, Pizza)`
//! - `(Likes, Pizza)`, `(Eats, Salad)`
//!
//! Variables with names that that start with a `_` are treated as anonymous, and are not accessible when a query is iterated.
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! // Anonymous variables
//! query!(world, (Likes, $"_"), (Eats, $"_"));
//! ```
//!
//! **Practical examples:**
//!
//! Spaceship docked to a planet:
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! # #[derive(Component)]
//! # struct DockedTo;
//! # #[derive(Component)]
//! # struct Planet;
//! query!(world, SpaceShip, (DockedTo, $"planet"), Planet($"planet"));
//! ```
//!
//! Eating healthy food you don't like:
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Eats;
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Healthy;
//! query!(world, (Eats, $"food"), !(Likes, $"food"), Healthy($"food"));
//! ```
//!
//! ## Source
//! All query terms have a "source", which is the entity on which the term is matched. If no term source is specified, it defaults to the `$this` variable. The following expressions show the same query without and with explicit source:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! // Implicit source
//! query!(world, &Position, &Velocity);
//! ```
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! // Explicit source
//! query!(world, Position($"this"), Velocity($"this"));
//! ```
//!
//! Note how both terms have the same `$this` source. Using the same variable ensures that both components are matched on the same entity.
//!
//! The following expressions show how to use pair queries without and with explicit source:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! # #[derive(Component)]
//! # struct Dogs;
//! # #[derive(Component)]
//! # struct Salad;
//! // Implicit source
//! query!(world, (Likes, Dogs), (Eats, Salad));
//! ```
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! # #[derive(Component)]
//! # struct Dogs;
//! # #[derive(Component)]
//! # struct Salad;
//! // Explicit source
//! query!(world, Likes($"this", Dogs), Eats($"this", Salad));
//! ```
//!
//! A single query can have terms that are matched on more than one source. The following sections describe the supported source kinds.
//!
//! ### Static source
//! A static source is a term that is always matched on an entity that is known at query creation time. A static source is specified by just using the name of the entity on which the component should be matched:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct TimeOfDay;
//! # world.entity_named("Game");
//! // Match TimeOfDay component on 'Game' entity
//! query!(world, TimeOfDay("Game"));
//! ```
//!
//! ### Variable source
//! A variable source is a variable that is used as term source. As mentioned already, when no source is specified, a term implicitly uses the builtin `$this` variable as source:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Velocity { x: f32, y: f32 }
//! // Match entities with both Position and Velocity
//! query!(world, Position($"this"), Velocity($"this"));
//! ```
//!
//! A variable used as source may appear in a different location in other terms. For example, the following expression uses a variable to match all entities that have components with the `Serializable` component:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Serializable;
//! query!(world, Serializable($"component"), $"component"($"this"));
//! ```
//!
//! The following example matches all spaceship entities that are docked to a planet:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! # #[derive(Component)]
//! # struct DockedTo;
//! # #[derive(Component)]
//! # struct Planet;
//! query!(world, SpaceShip($"this"), DockedTo($"this", $"planet"), Planet($"planet"));
//! ```
//!
//! The following example matches all entities that are eating healthy, but do not like what they are eating:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Eats;
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Healthy;
//! query!(world, Eats($"this", $"food"), !Likes($"this", $"food"), Healthy($"food"));
//! ```
//!
//! ## Traversal
//! Query traversal makes it possible to match a component by traversing a relationship until an entity with the component has been found. A common use case for this is a transform system, where a `Transform` component is matched both on an entity and its parent.
//!
//! The following expression shows an example of a query that matches a `Transform` component both on an entity and its parent:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Transform { x: f32, y: f32 }
//! query!(world, &Transform($"this"), Transform($"this" up flecs::ChildOf));
//! ```
//!
//! The same query can be specified without the `$this` variable:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Transform { x: f32, y: f32 }
//! query!(world, &Transform, &Transform(up flecs::ChildOf));
//! ```
//!
//! As `ChildOf` is the default traversal relationship, this query can be further shortened to:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Transform { x: f32, y: f32 }
//! query!(world, &Transform, &Transform(up));
//! ```
//!
//! The `cascade` modifier is similar to `up` but returns results in breadth-first order. This is typically used in transform systems to ensure parents are transformed before children. The following expression shows an example with `cascade`:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Transform { x: f32, y: f32 }
//! query!(world, &Transform, &Transform(cascade));
//! ```
//!
//! The `desc` modifier can be used in combination with `cascade` to return results in reverse order:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Transform { x: f32, y: f32 }
//! query!(world, &Transform, &Transform(cascade | desc));
//! ```
//!
//! The `self` traversal modifier can be used in combination with `up` to first test if the entity itself has the component before traversing the hierarchy:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Style { theme: String }
//! // First try matching Style on self, find on parent if not found
//! query!(world, &Position, &Style(self | up));
//! ```
//!
//! When a component has the `(OnInstantiate, Inherit)` trait, queries will automatically insert `self|up` traversal for the `IsA` relationship. The following two queries are equivalent, if `Style` has the `(OnInstantiate, Inherit)` trait:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Style { theme: String }
//! query!(world, &Position, &Style);
//! ```
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Style { theme: String }
//! query!(world, &Position, &Style(self | up flecs::IsA));
//! ```
//!
//! Traversal modifiers can be used with any relationship that has the `Traversable` trait:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct MyRelationship;
//! world.component::<MyRelationship>().add_trait::<flecs::Traversable>();
//!
//! // Then use in queries
//! query!(world, &Position(up MyRelationship));
//! ```
//!
//! When a query matches a component that is inherited from, a query will automatically traverse the `IsA` relationship downwards to find all subclasses. For example, if `MeleeUnit` has an `IsA` relationship to `Unit`, the following query matches entities with `Unit` and `MeleeUnit`:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Unit;
//! query!(world, Unit);
//! ```
//!
//! To prevent queries from evaluating component inheritance, the `self` modifier can be added to the component:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Unit;
//! query!(world, Unit(self));
//! ```
//!
//! For terms with an explicit source, the `self` modifier comes after the source variable:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Unit;
//! query!(world, Unit($"this" self));
//! ```
//!
//! When a query matches a relationship that has the `Transitive` trait, it will traverse the relationship up or down depending on which parts of the query are variable. To prevent a query from matching results transitively, add the `self` modifier to the second element of a pair:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct LocatedIn;
//! world.component::<LocatedIn>().add_trait::<flecs::Transitive>();
//! # #[derive(Component)]
//! # struct SanFrancisco;
//! query!(world, (LocatedIn, SanFrancisco | self));
//! ```
//!
//! This will only match entities that have `(LocatedIn, SanFrancisco)` and not, for example, entities with `(LocatedIn, GoldenGateBridge)`.
//!
//! ## Advanced
//!
//! ### Equality operators
//! Equality operators allow queries to match variables with specific values or names. The following example shows a query that matches a variable against with a specific entity:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! # #[derive(Component)]
//! # struct UssEnterprise;
//! # #[derive(Component)]
//! # struct Voyager;
//! query!(world, SpaceShip($"this"), ($"this" == UssEnterprise || $"this" != Voyager));
//! ```
//!
//! The `!=` operator can be used to negate a result:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! # #[derive(Component)]
//! # struct UssEnterprise;
//! query!(world, SpaceShip($"this"), ($"this" != UssEnterprise));
//! ```
//!
//! Queries may also compare two variables:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct PoweredBy;
//! query!(world, (PoweredBy, $"source"), ($"this" != $"source"));
//! ```
//!
//! When a string is used as operand, the operation will test if the name of the entity matches:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! query!(world, SpaceShip($"this"), ($"this" == "UssEnterprise"));
//! ```
//!
//! The `~=` operator can be used to do a fuzzy comparison, equivalent to the behavior of the `substr` function:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! query!(world, SpaceShip($"this"), ($"this" ~= "Uss"));
//! ```
//!
//! The result of `~=` can be negated by prefixing the expression with a `!`:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! query!(world, SpaceShip($"this"), ($"this" ~= "!Uss"));
//! ```
//!
//! When an equality operator is the first term that populates a variable, it will assign the variable:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! query!(world, ($"this" == "UssEnterprise"),SpaceShip($"this"));
//! ```
//!
//! ### Lookup variables
//! Variables can be used as the starting point of a by-name lookup. This can be useful when matching hierarchies that have a well-defined structure. The following expression is an example of a query with a lookup variable:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! # #[derive(Component)]
//! # struct PoweredBy;
//! // Match all spaceship entities where the cockpit has no power
//! query!(world, SpaceShip($"this"), !PoweredBy($"this.cockpit"));
//! ```
//!
//! This query will look for an child entity named `cockpit` in the scope of the matched entity for `$this`, and use that entity to match with `Powered`. If no entity with the name `cockpit` is found, the term will evaluate to false.
//! ### Member matching
//! Queries can match against the values of component members if they are of the `ecs_entity_t` type. The following expression shows an example of how to match against a `direction` member in a `Movement` component:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! #[derive(Component)]
//! #[flecs(meta, name = "Movement")]
//! struct Movement {
//!     direction: Entity,
//! }
//!
//! let left = world.entity();
//! world.entity().set(Movement {
//!     direction: left.id(),
//! });
//! query!(world, "Movement.direction"($"this",$left));
//! ```
//!
//! The same query with an implicit source:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! #[derive(Component)]
//! #[flecs(meta, name = "Movement")]
//! struct Movement {
//!     direction: Entity,
//! }
//!
//! let left = world.entity();
//! world.entity().set(Movement {
//!     direction: left.id(),
//! });
//! query!(world, ("Movement.direction",$left));
//! ```
//!
//! A member expression can be used in combination with variables:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! #[derive(Component)]
//! #[flecs(meta, name = "Thrusters")]
//! struct Thrusters {
//!     left: Entity,
//!     right: Entity,
//! }
//! #[derive(Component)]
//! struct Active;
//!
//! let thruster_left = world.entity().add(Active);
//! let thruster_right = world.entity();
//! world.entity_named("ASpaceship").set(Thrusters {
//!     left: thruster_left.id(),
//!     right: thruster_right.id(),
//! });
//! query!(world, ("Thrusters.left",$"thruster"), Active($"thruster"))
//!     .build()
//!     .each_entity(|e, _| {
//!         //prints ASpaceship
//!         println!("Entity with active left thruster: {}", e.name());
//!     });
//! ```
//!
//! ### Dependent variables
//! When a variable is used first in a term that is conditionally evaluated, any subsequent terms that use the variable will only be evaluated if the variable was set. This allows for the creation of simple branches within queries. The following expression shows an example of dependent variables:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct Likes;
//! # #[derive(Component)]
//! # struct Eats;
//! # #[derive(Component)]
//! # struct Friendly;
//! # #[derive(Component)]
//! # struct Healthy;
//! query!(
//!         world,
//!         /* $animal and $"food" are set conditionally */
//!         (Likes, $"animal") || (Eats, $"food"),
//!         Friendly($"animal")/* Evaluated if (Likes, $animal) matched */,
//!         Healthy($"food") /* Evaluated if (Eats, $"food") matched*/);
//! ```
//!
//! Dependent variables can also be created from optional terms:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! # #[derive(Component)]
//! # struct DockedTo;
//! # #[derive(Component)]
//! # struct Planet;
//! // Planet($"object") is only evaluated if (DockedTo, $"object")
//! // returned a result.
//! query!(world, SpaceShip, ?(DockedTo, $"object"), Planet($"object"));
//! ```
//!
//! ### Query scopes
//! Query scopes can be used to apply an operator to the result of more than one term. Currently query scopes are only supported in combination with `not` operators. The following expressions show examples of query scopes:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! # #[derive(Component)]
//! # struct Engine;
//! # #[derive(Component)]
//! # struct Healthy;
//! // Match spaceships where none of the engines are healthy
//! query!(world, SpaceShip, !{ (Engine, $"engine"), Healthy($"engine") });
//! ```
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # let world = World::new();
//! # #[derive(Component)]
//! # struct SpaceShip;
//! # #[derive(Component)]
//! # struct Engine;
//! # #[derive(Component)]
//! # struct Healthy;
//! // Match spaceships where all of the engines are healthy
//! query!(world, SpaceShip, !{ (Engine, $"engine"), !Healthy($"engine") });
//! ```
//!
