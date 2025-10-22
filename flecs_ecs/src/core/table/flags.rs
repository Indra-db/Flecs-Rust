//! Table property flags.
//!
//! This module defines [`TableFlags`] which describe various properties and capabilities of
//! a table. These flags indicate what kinds of components, relationships, hooks, and
//! behaviors are present in a table.

use crate::sys;

bitflags::bitflags! {
    /// Bitflags describing the properties and capabilities of a table.
    ///
    /// These flags indicate what components, relationships, hooks, and behaviors
    /// are present in a table. They are used internally for optimization and to
    /// determine what operations need to be performed on a table.
    ///
    /// # Component Type Flags
    ///
    /// - [`HasBuiltins`](Self::TableFlags::HasBuiltins): Table contains builtin components
    /// - [`HasPairs`](Self::HasPairs): Table contains relationship pairs
    /// - [`HasToggle`](Self::HasToggle): Table has components with the [`CanToggle`](crate::core::flecs::CanToggle) trait
    /// - [`HasSparse`](Self::HasSparse): Table has sparse components
    /// - [`HasDontFragment`](Self::HasDontFragment): Table has components with the [`DontFragment`](crate::core::flecs::DontFragment) trait
    ///
    /// # Relationship Flags
    ///
    /// - [`HasIsA`](Self::HasIsA): Table has entities with `IsA` relationships
    /// - [`HasChildOf`](Self::HasChildOf): Table has entities with `ChildOf` relationships
    /// - [`HasTraversable`](Self::HasTraversable): Table has traversable relationships
    ///
    /// # Entity State Flags
    ///
    /// - [`IsPrefab`](Self::IsPrefab): Table contains prefab entities
    /// - [`IsDisabled`](Self::IsDisabled): Table contains disabled entities
    /// - [`NotQueryable`](Self::NotQueryable): Table cannot be queried
    ///
    /// # Component Lifecycle Flags
    ///
    /// - [`HasCtors`](Self::HasCtors): Table has components with constructors
    /// - [`HasDtors`](Self::HasDtors): Table has components with destructors
    /// - [`HasCopy`](Self::HasCopy): Table has components with copy operations
    /// - [`HasMove`](Self::HasMove): Table has components with move operations
    ///
    /// # Hook Flags
    ///
    /// - [`HasOnAdd`](Self::HasOnAdd): Table has components with `OnAdd` hooks
    /// - [`HasOnRemove`](Self::HasOnRemove): Table has components with `OnRemove` hooks
    /// - [`HasOnSet`](Self::HasOnSet): Table has components with `OnSet` hooks
    /// - [`HasOnTableCreate`](Self::HasOnTableCreate): Table has `OnTableCreate` hooks
    /// - [`HasOnTableDelete`](Self::HasOnTableDelete): Table has `OnTableDelete` hooks
    ///
    /// # Example
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// # use flecs_ecs::core::table::TableFlags;
    /// # #[derive(Component)]
    /// # struct Position { x: f32, y: f32 }
    /// let flags = TableFlags::HasPairs | TableFlags::HasBuiltins;
    ///
    /// if flags.contains(TableFlags::HasPairs) {
    ///     println!("Table contains relationship pairs");
    /// }
    /// ```
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct TableFlags: u32 {
        /// Table contains builtin Flecs components.
        const HasBuiltins = sys::EcsTableHasBuiltins;
        /// Table contains prefab entities (entities with the `Prefab` tag).
        const IsPrefab = sys::EcsTableIsPrefab;
        /// Table has entities with `IsA` relationships (inheritance).
        const HasIsA = sys::EcsTableHasIsA;
        /// Table has entities with `ChildOf` relationships (hierarchy).
        const HasChildOf = sys::EcsTableHasChildOf;
        /// Table has entities with names.
        const HasName = sys::EcsTableHasName;
        /// Table contains relationship pairs.
        const HasPairs = sys::EcsTableHasPairs;
        /// Table has entities marked as modules.
        const HasModule = sys::EcsTableHasModule;
        /// Table contains disabled entities.
        const IsDisabled = sys::EcsTableIsDisabled;
        /// Table cannot be queried (internal use).
        const NotQueryable = sys::EcsTableNotQueryable;
        /// Table has components with constructors.
        const HasCtors = sys::EcsTableHasCtors;
        /// Table has components with destructors.
        const HasDtors = sys::EcsTableHasDtors;
        /// Table has components with copy operations.
        const HasCopy = sys::EcsTableHasCopy;
        /// Table has components with move operations.
        const HasMove = sys::EcsTableHasMove;
        /// Table has components that can be toggled (enabled/disabled).
        const HasToggle = sys::EcsTableHasToggle;
        /// Table has components that override base components.
        const HasOverrides = sys::EcsTableHasOverrides;
        /// Table has components with `OnAdd` hooks.
        const HasOnAdd = sys::EcsTableHasOnAdd;
        /// Table has components with `OnRemove` hooks.
        const HasOnRemove = sys::EcsTableHasOnRemove;
        /// Table has components with `OnSet` hooks.
        const HasOnSet = sys::EcsTableHasOnSet;
        /// Table has `OnTableCreate` hooks.
        const HasOnTableCreate = sys::EcsTableHasOnTableCreate;
        /// Table has `OnTableDelete` hooks.
        const HasOnTableDelete = sys::EcsTableHasOnTableDelete;
        /// Table has sparse components (stored outside the table).
        const HasSparse = sys::EcsTableHasSparse;
        /// Table has components with the `DontFragment` trait.
        const HasDontFragment = sys::EcsTableHasDontFragment;
        /// Table overrides the `DontFragment` behavior.
        const OverrideDontFragment = sys::EcsTableOverrideDontFragment;
        /// Table has traversable relationships.
        const HasTraversable = sys::EcsTableHasTraversable;
        /// Table has entities with ordered children.
        const HasOrderedChildren = sys::EcsTableHasOrderedChildren;
        /// Edge requires reparenting when traversed.
        const EdgeReparent = sys::EcsTableEdgeReparent;
        /// Table is marked for deletion.
        const MarkedForDelete = sys::EcsTableMarkedForDelete;
        /// Table has any lifecycle hooks (ctors, dtors, copy, move).
        const HasLifecycle = sys::EcsTableHasLifecycle;
        /// Table has complex components requiring special handling.
        const IsComplex = sys::EcsTableIsComplex;
        /// Table has actions triggered on component add.
        const HasAddActions = sys::EcsTableHasAddActions;
        /// Table has actions triggered on component remove.
        const HasRemoveActions = sys::EcsTableHasRemoveActions;
        /// Combined edge flags.
        const EdgeFlags = sys::EcsTableEdgeFlags;
        /// Combined add edge flags.
        const AddEdgeFlags = sys::EcsTableAddEdgeFlags;
        /// Combined remove edge flags.
        const RemoveEdgeFlags = sys::EcsTableRemoveEdgeFlags;
    }
}
