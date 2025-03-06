use core::{
    ffi::{CStr, c_void},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use crate::sys;
use flecs_ecs::core::*;
use flecs_ecs_sys::ecs_rust_table_id;
use sys::ecs_get_with;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
#[allow(unused_imports)] //meant for no_std, not ready yet
use alloc::{borrow::ToOwned, boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// A view into an entity in the world that provides both read and write access to components and relationships.
///
/// `EntityView` is a wrapper around an entity ID that provides a safe interface for:
/// - Getting and setting component data
/// - Adding and removing components
/// - Managing relationships between entities
/// - Working with hierarchies (parent/child relationships)
/// - Querying entity metadata like names and paths
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use flecs_ecs::prelude::*;
///
/// // Define some components
/// #[derive(Component)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// #[derive(Component)]
/// struct Velocity {
///     x: f32,
///     y: f32,
/// }
///
/// #[derive(Component)]
/// struct Walking;
///
/// let world = World::new();
///
/// // Create an entity and add components
/// let player = world
///     .entity_named("Player")
///     .set(Position { x: 10.0, y: 20.0 })
///     .set(Velocity { x: 1.0, y: 0.0 })
///     .add::<Walking>();
///
/// // Get component data
/// player.get::<&Position>(|pos| {
///     println!("Position: ({}, {})", pos.x, pos.y);
/// });
///
/// // Check for components
/// assert!(player.has::<Walking>());
///
/// // Remove components
/// player.remove::<Walking>();
/// ```
///
/// Working with hierarchies:
/// ```rust
/// use flecs_ecs::prelude::*;
///
/// let world = World::new();
///
/// let parent = world.entity_named("parent");
/// let child = world.entity_named("child").child_of_id(parent);
///
/// assert!(parent.has_children());
/// assert_eq!(child.path().unwrap(), "::parent::child");
/// ```
///
/// # See also
///
/// - [`World::entity()`] - Create a new entity
/// - [`World::entity_named()`] - Create a new named entity
/// - [`Entity`] - The underlying entity identifier type
#[derive(Clone, Copy)]
pub struct EntityView<'a> {
    pub(crate) world: WorldRef<'a>,
    pub(crate) id: Entity,
}

impl Deref for EntityView<'_> {
    type Target = Entity;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl DerefMut for EntityView<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}

impl core::fmt::Display for EntityView<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(name) = self.get_name() {
            write!(f, "#{} | {}", self.id, name)
        } else {
            write!(f, "#{}", self.id)
        }
    }
}

impl core::fmt::Debug for EntityView<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let entity_display = match self.get_name() {
            Some(name_str) => alloc::format!("Entity: #{} | \"{}\"", self.id, name_str),
            None => format!("Entity: #{}", self.id),
        };
        let archetype_types_str = debug_separate_archetype_types_into_strings(&self.archetype());

        let mut children = alloc::vec![];
        self.each_child(|child| {
            children.push({
                match child.get_name() {
                    Some(name) => format!("#{} | \"{}\"", child.id, name),
                    None => format!("#{}", child.id),
                }
            });
        });

        if children.is_empty() {
            return write!(
                f,
                "\n  {}\n  Archetype:\n    - {}\n",
                entity_display,
                archetype_types_str.join("\n    - "),
            );
        }

        write!(
            f,
            "\n  {}\n  Archetype:\n    - {}\n  Children:\n    - {}\n",
            entity_display,
            archetype_types_str.join("\n    - "),
            children.join("\n    - ")
        )
    }
}

impl<'a> EntityView<'a> {
    /// Create a new entity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let entity = world.entity(); // Creates new unnamed entity
    /// assert!(entity.is_alive());
    /// assert!(entity.id() != 0);
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::new_named()`] - Create a named entity
    /// * [`EntityView::new_from()`] - Create from existing ID
    /// * [`World::entity()`] - Preferred way to create entities
    #[doc(alias = "entity::entity")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub(crate) fn new(world: impl WorldProvider<'a>) -> Self {
        let world_ptr = world.world_ptr_mut();
        let id = if unsafe { sys::ecs_get_scope(world_ptr) == 0 && ecs_get_with(world_ptr) == 0 } {
            unsafe { sys::ecs_new(world_ptr) }
        } else {
            let desc = sys::ecs_entity_desc_t::default();
            unsafe { sys::ecs_entity_init(world_ptr, &desc) }
        };
        Self {
            world: world.world(),
            id: id.into(),
        }
    }

    /// Creates a wrapper around an existing entity / id.
    ///
    /// This is useful when you have an entity ID and need to perform operations on it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let id = world.entity().id();
    /// let entity_view = EntityView::new_from(&world, id);
    ///
    /// assert_eq!(entity_view.id(), id);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Entity::entity_view()`] - Convert Entity to EntityView
    /// * [`World::entity_from_id()`] - Get entity view from ID
    #[doc(alias = "entity::entity")]
    #[doc(hidden)] //public due to macro newtype_of and world.entity_from_id has lifetime issues.
    pub fn new_from(world: impl WorldProvider<'a>, id: impl Into<Entity>) -> Self {
        Self {
            world: world.world(),
            id: id.into(),
        }
    }

    /// Create a named entity.
    ///
    /// Creates a new entity with the specified name. Named entities can be looked up using
    /// lookup functions. Entity names may be scoped with "::" separator.
    /// Parts of the hierarchy that don't exist will be automatically created.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// // Create simple named entity
    /// let bob = world.entity_named("Bob");
    /// assert_eq!(bob.name(), "Bob");
    ///
    /// // Create entity with hierarchical name
    /// let weapon = world.entity_named("Characters::Bob::Weapon");
    /// assert_eq!(weapon.path(), Some("::Characters::Bob::Weapon".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::entity_named()`] - Preferred way to create named entities
    /// * [`EntityView::name()`] - Get entity name
    /// * [`EntityView::path()`] - Get full hierarchical path
    /// * [`World::lookup()`] - Look up named entities
    #[doc(alias = "entity::entity")]
    pub(crate) fn new_named(world: impl WorldProvider<'a>, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            parent: 0,
            symbol: core::ptr::null(),
            use_low_id: false,
            add: core::ptr::null(),
            add_expr: core::ptr::null(),
            set: core::ptr::null(),
        };
        let id = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &desc) };
        Self {
            world: world.world(),
            id: id.into(),
        }
    }

    pub(crate) fn new_named_cstr(world: impl WorldProvider<'a>, name: &CStr) -> Self {
        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            parent: 0,
            symbol: core::ptr::null(),
            use_low_id: false,
            add: core::ptr::null(),
            add_expr: core::ptr::null(),
            set: core::ptr::null(),
        };
        let id = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &desc) };
        Self {
            world: world.world(),
            id: id.into(),
        }
    }

    /// Creates a null entity (ID 0) associated with the given world.
    ///
    /// This is useful when you need an entity reference that belongs to a world
    /// but represents "no entity" (ID 0).
    ///
    /// # See also
    ///
    /// * [`Entity::null()`] - Create null entity ID
    /// * [`EntityView::is_valid()`] - Check if entity is valid
    #[doc(alias = "entity::null")]
    pub(crate) fn new_null(world: &'a World) -> EntityView<'a> {
        Self::new_from(world, 0)
    }

    /// Get the [`IdView`] representation of the entity.
    ///
    /// Converts the entity view to an ID view, which provides operations
    /// for working with the entity as a component identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    /// let entity = world.entity();
    ///
    /// let id_view = entity.id_view();
    /// assert_eq!(id_view.id(), entity.id());
    /// ```
    ///
    /// # See also
    ///
    /// * [`Entity::id_view()`] - Convert Entity to [`IdView`]
    pub fn id_view(&self) -> IdView {
        IdView::new_from_id(self.world, *self.id)
    }

    /// Check if entity is valid.
    ///
    /// Entities are valid if they are not 0 and if they are alive. This function
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let entity = world.entity();
    /// let entity_copy = entity; // Creates a copy with same ID
    /// let entity_clone = entity.duplicate(false); // Creates new entity with different ID
    ///
    /// assert!(entity.is_valid());
    /// assert!(entity_copy.is_valid());
    /// assert!(entity_clone.is_valid());
    ///
    /// entity.destruct();
    ///
    /// assert!(!entity_copy.is_valid()); // Copy refers to same (now deleted) entity
    /// assert!(entity_clone.is_valid()); // Clone is a different entity
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::is_alive()`] - Check if entity exists in world
    /// * [`EntityView::duplicate()`] - Create copy of entity
    #[doc(alias = "entity_view::is_valid")]
    pub fn is_valid(self) -> bool {
        unsafe { sys::ecs_is_valid(self.world.world_ptr(), *self.id) }
    }

    /// Check if entity is alive.
    ///
    /// An entity is alive if it exists in the world. This is different from
    /// valid, as an entity that is not alive can still be valid if it has been
    /// deferred for deletion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let entity = world.entity();
    /// assert!(entity.is_alive());
    ///
    /// world.defer(|| {
    ///     entity.destruct(); // Deferred deletion
    ///     assert!(entity.is_alive()); // still alive since deferred
    /// });
    ///
    /// assert!(!entity.is_alive());
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::is_valid()`] - Check if entity is valid
    /// * [`World::defer()`] - Defer operations
    #[doc(alias = "entity_view::is_alive")]
    pub fn is_alive(self) -> bool {
        unsafe { sys::ecs_is_alive(self.world.world_ptr(), *self.id) }
    }

    /// Returns the entity name.
    ///
    /// Returns the name of the entity if one was assigned, or an empty string if
    /// the entity has no name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let bob = world.entity_named("Bob");
    /// assert_eq!(bob.name(), "Bob");
    ///
    /// let unnamed = world.entity();
    /// assert_eq!(unnamed.name(), "");
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::get_name()`] - Get name as Option
    /// * [`EntityView::path()`] - Get full hierarchical path
    /// * [`World::entity_named()`] - Create named entity
    #[doc(alias = "entity_view::name")]
    pub fn name(self) -> String {
        self.get_name().unwrap_or("".to_string())
    }

    /// Returns the entity name as an Option.
    ///
    /// Similar to [`name()`][EntityView::name] but returns None if the entity has no name instead of an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let bob = world.entity_named("Bob");
    /// assert_eq!(bob.get_name(), Some("Bob".to_string()));
    ///
    /// let unnamed = world.entity();
    /// assert_eq!(unnamed.get_name(), None);
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::name()`] - Get name, returns empty string if unnamed
    /// * [`EntityView::path()`] - Get full hierarchical path
    #[doc(alias = "entity_view::name")]
    pub fn get_name(self) -> Option<String> {
        // self.get_name_cstr().and_then(|s| s.to_str().ok())
        let cstr =
            NonNull::new(unsafe { sys::ecs_get_name(self.world.world_ptr(), *self.id) } as *mut _)
                .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) });
        cstr.and_then(|s| s.to_str().ok().map(ToString::to_string))
    }

    // /// Returns the entity name as a `CStr`.
    // ///
    // /// if the entity has no name, this will return an empty string
    // ///
    // /// # See also
    // ///
    // /// * C++ API: `entity_view::name`
    // pub fn name_cstr(self) -> &'a CStr {
    //     self.get_name_cstr().unwrap_or(c"")
    // }

    /// Returns the entity name as a `CStr`.
    ///
    /// If the entity has no name, this will return `None`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it returns a raw pointer to the C string. You have to manually
    /// ensure that the C string is valid for the lifetime of the pointer.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::name`
    pub(crate) unsafe fn get_name_cstr(&self) -> Option<&CStr> {
        NonNull::new(unsafe { sys::ecs_get_name(self.world.world_ptr(), *self.id) } as *mut _)
            .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) })
    }

    /// Returns the entity symbol.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::symbol`
    #[doc(alias = "entity_view::symbol")]
    pub fn symbol(self) -> String {
        //self.symbol_cstr().to_str().unwrap()
        let cstr = unsafe { CStr::from_ptr(sys::ecs_get_symbol(self.world.world_ptr(), *self.id)) };
        cstr.to_str()
            .ok()
            .map(ToString::to_string)
            .unwrap_or_default()
    }

    /// Return the hierarchical entity path.
    ///
    /// Returns the full hierarchical path of the entity, with path elements separated
    /// by the specified separators.
    ///
    /// # Note
    ///
    /// if you're using the default separator "::" you can use the non-allocating no `w_sep` version
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// // Create parent entity
    /// let parent = world.entity_named("Parent");
    ///
    /// // Create child entity
    /// let child = world.entity_named("Child").child_of_id(parent);
    ///
    /// assert_eq!(
    ///     child.path_w_sep("::", "::"),
    ///     Some("::Parent::Child".to_string())
    /// );
    /// assert_eq!(
    ///     child.path_w_sep("/", "/"),
    ///     Some("/Parent/Child".to_string())
    /// );
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::path()`] - Get path with default separator
    /// * [`EntityView::name()`] - Get entity name only
    #[doc(alias = "entity_view::path")]
    pub fn path_w_sep(self, sep: &str, init_sep: &str) -> Option<String> {
        self.path_from_id_w_sep(0, sep, init_sep)
    }

    /// Return the hierarchical entity path using the default separator "::".
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let parent = world.entity_named("Parent");
    /// let child = world.entity_named("Child").child_of_id(parent);
    ///
    /// assert_eq!(child.path(), Some("::Parent::Child".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::path_w_sep()`] - Get path with custom separator
    /// * [`EntityView::path_from()`] - Get path relative to parent
    #[doc(alias = "entity_view::path")]
    pub fn path(self) -> Option<String> {
        self.path_from_id(0)
    }

    /// Return the hierarchical entity path relative to a parent.
    ///
    /// Returns the path of the entity relative to the specified parent entity.
    /// Supports custom separators for path elements.
    ///
    /// # Note
    ///
    /// if you're using the default separator "::" you can use the non-allocating no `w_sep` version
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let root = world.entity_named("Root");
    /// let parent = world.entity_named("Parent").child_of_id(root);
    /// let child = world.entity_named("Child").child_of_id(parent);
    ///
    /// // Get path relative to root
    /// assert_eq!(
    ///     child.path_from_id_w_sep(root, "::", "::"),
    ///     Some("Parent::Child".to_string())
    /// );
    ///
    /// // Get path relative to parent
    /// assert_eq!(
    ///     child.path_from_id_w_sep(parent, "::", "::"),
    ///     Some("Child".to_string())
    /// );
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::path_from()`] - Get path relative to parent type
    /// * [`EntityView::path()`] - Get full path
    #[doc(alias = "entity_view::path_from")]
    pub fn path_from_id_w_sep(
        &self,
        parent: impl Into<Entity>,
        sep: &str,
        init_sep: &str,
    ) -> Option<String> {
        let sep = compact_str::format_compact!("{}\0", sep);
        let init_sep = compact_str::format_compact!("{}\0", init_sep);

        NonNull::new(unsafe {
            sys::ecs_get_path_w_sep(
                self.world.world_ptr(),
                *parent.into(),
                *self.id,
                sep.as_ptr() as *const _,
                init_sep.as_ptr() as *const _,
            )
        })
        .map(|s| unsafe {
            let len = CStr::from_ptr(s.as_ptr()).to_bytes().len();
            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            String::from_utf8_unchecked(Vec::from_raw_parts(s.as_ptr() as *mut u8, len, len))
        })
    }

    fn path_from_id_default_sep(&self, parent: impl Into<Entity>) -> Option<String> {
        NonNull::new(unsafe {
            sys::ecs_get_path_w_sep(
                self.world.world_ptr(),
                *parent.into(),
                *self.id,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
            )
        })
        .map(|s| unsafe {
            let len = CStr::from_ptr(s.as_ptr()).to_bytes().len();
            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            String::from_utf8_unchecked(Vec::from_raw_parts(s.as_ptr() as *mut u8, len, len))
        })
    }

    /// Return the hierarchical entity path relative to a parent id using the default separator "::".
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// let root = world.entity_named("Root");
    /// let parent = world.entity_named("Parent").child_of_id(root);
    /// let child = world.entity_named("Child").child_of_id(parent);
    ///
    /// assert_eq!(child.path_from_id(root), Some("Parent::Child".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::path_from_id_w_sep()`] - Get path with custom separator
    /// * [`EntityView::path_from()`] - Get path relative to parent type
    #[doc(alias = "entity_view::path_from")]
    pub fn path_from_id(self, parent: impl Into<Entity>) -> Option<String> {
        NonNull::new(unsafe {
            sys::ecs_get_path_w_sep(
                self.world.world_ptr(),
                *parent.into(),
                *self.id,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
            )
        })
        .map(|s| unsafe {
            let len = CStr::from_ptr(s.as_ptr()).to_bytes().len();

            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            String::from_utf8_unchecked(Vec::from_raw_parts(s.as_ptr() as *mut u8, len, len))
        })
    }

    /// Return the hierarchical entity path relative to a parent type.
    ///
    /// Gets the path relative to an entity of the specified component type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// #[derive(Component)]
    /// struct Earth;
    ///
    /// let world = World::new();
    /// let sun = world.entity_named("Sun");
    /// let earth = world.component::<Earth>().child_of_id(sun);
    /// let moon = world.entity_named("Moon").child_of_id(earth);
    ///
    /// // Get moon's path relative to nearest Planet
    /// assert_eq!(moon.path_from::<Earth>(), Some("Moon".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::path_from_id()`] - Get path relative to specific entity
    /// * [`EntityView::path()`] - Get full path
    #[doc(alias = "entity_view::path_from")]
    pub fn path_from<T: ComponentId>(self) -> Option<String> {
        self.path_from_id_default_sep(T::id(self.world))
    }
    /// Return the hierarchical entity path relative to a parent type with custom separators.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// #[derive(Component)]
    /// struct Sun;
    ///
    /// let world = World::new();
    /// let sun = world.component::<Sun>();
    /// let earth = world.entity_named("Earth").child_of_id(sun);
    /// let moon = world.entity_named("Moon").child_of_id(earth);
    ///
    /// assert_eq!(
    ///     moon.path_from_w_sep::<Sun>("/", "/"),
    ///     Some("Earth/Moon".to_string())
    /// );
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::path_from()`] - Get path with default separator
    /// * [`EntityView::path_from_id_w_sep()`] - Get path relative to ID with custom separator
    pub fn path_from_w_sep<T: ComponentId>(&self, sep: &str, init_sep: &str) -> Option<String> {
        self.path_from_id_w_sep(T::id(self.world), sep, init_sep)
    }

    /// Return the hierarchical entity path relative to a parent type using the default separator "::".
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::path_from`
    #[doc(alias = "entity_view::path_from")]
    pub fn hierarchy_path_from_parent_type<T: ComponentId>(self) -> Option<String> {
        self.path_from_id(T::id(self.world))
    }

    /// Checks if the entity is enabled.
    ///
    /// # Returns
    ///
    /// True if the entity is enabled, false if disabled.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_self(self) -> bool {
        unsafe { !sys::ecs_has_id(self.world.world_ptr(), *self.id, flecs::Disabled::ID) }
    }

    /// Get the entity's archetype.
    ///
    /// An archetype represents the structural type of an entity - the exact set of
    /// components and relationships it has.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flecs_ecs::prelude::*;
    /// #[derive(Component)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    /// #[derive(Component)]
    /// struct Velocity {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// let world = World::new();
    /// let entity = world
    ///     .entity()
    ///     .set(Position { x: 0.0, y: 0.0 })
    ///     .set(Velocity { x: 1.0, y: 1.0 });
    ///
    /// // Get archetype and inspect components
    /// let archetype = entity.archetype();
    /// for id in archetype.as_slice() {
    ///     println!("Component in archetype: {}", id);
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// The archetype of the entity. If the entity is not in a table,
    /// returns an empty archetype.
    ///
    /// # See also
    ///
    /// * [`EntityView::table()`] - Get entity's table
    /// * [`EntityView::each_component()`] - Iterate components
    /// * [`Archetype::as_slice()`] - Get component IDs in archetype
    #[doc(alias = "entity_view::type")]
    #[inline(always)]
    pub fn archetype(self) -> Archetype<'a> {
        self.table()
            .map(|t| t.archetype())
            .unwrap_or(Archetype::new(self.world, &[]))
    }

    /// Get the entity's type/table.
    ///
    /// A table stores entities that share the same archetype (same set of components).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flecs_ecs::prelude::*;
    /// #[derive(Component, Default)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// let world = World::new();
    /// let entity = world.entity().add::<Position>();
    ///
    /// if let Some(table) = entity.table() {
    ///     println!(
    ///         "Entity is stored in table with {} total entities",
    ///         table.count()
    ///     );
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// The table containing the entity, or None if the entity is not stored
    /// in a table.
    ///
    /// # See also
    ///
    /// * [`EntityView::archetype()`] - Get entity's archetype
    /// * [`EntityView::range()`] - Get entity's position in table
    /// * [`Table::count()`] - Get number of entities in table
    #[doc(alias = "entity_view::table")]
    #[inline(always)]
    pub fn table(self) -> Option<Table<'a>> {
        NonNull::new(unsafe { sys::ecs_get_table(self.world.world_ptr(), *self.id) })
            .map(|t| Table::new(self.world, t))
    }

    /// Get table range for the entity.
    ///
    /// A table range represents the location of an entity within its table.
    /// This is useful for bulk operations on entities that share the same components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Default)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// let world = World::new();
    /// let entity = world.entity().add::<Position>();
    ///
    /// if let Some(range) = entity.range() {
    ///     println!("Entity is at row {} in its table", range.offset());
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// A range with the entity's row as offset and count set to 1.
    /// Returns None if the entity is not stored in a table.
    ///
    /// # See also
    ///
    /// * [`EntityView::table()`] - Get entity's table
    /// * [`TableRange::offset()`] - Get starting row in table
    /// * [`TableRange::count()`] - Get number of rows in range
    #[doc(alias = "entity_view::range")]
    #[inline]
    pub fn range(self) -> Option<TableRange<'a>> {
        NonNull::new(unsafe { sys::ecs_record_find(self.world.world_ptr(), *self.id) }).map(
            |record| unsafe {
                TableRange::new_raw(
                    self.world,
                    NonNull::new_unchecked((*record.as_ptr()).table),
                    ecs_record_to_row((*record.as_ptr()).row),
                    1,
                )
            },
        )
    }

    /// Iterate over component ids of an entity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flecs_ecs::prelude::*;
    /// #[derive(Component)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    /// #[derive(Component)]
    /// struct Velocity {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// let world = World::new();
    /// let entity = world
    ///     .entity()
    ///     .set(Position { x: 0.0, y: 0.0 })
    ///     .set(Velocity { x: 1.0, y: 1.0 });
    ///
    /// // Print all component ids
    /// entity.each_component(|id| {
    ///     println!("Component: {}", id.id());
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::each_pair()`] - Iterate over pairs
    /// * [`EntityView::each_target()`] - Iterate over relationship targets
    /// * [`EntityView::archetype()`] - Get entity's archetype
    #[doc(alias = "entity_view::each")]
    pub fn each_component(self, mut func: impl FnMut(IdView)) {
        let archetype = self.archetype();

        for &id in archetype.as_slice() {
            let ent = IdView::new_from_id(self.world, id);
            func(ent);
        }
    }

    /// Iterates over matching pair IDs of an entity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flecs_ecs::prelude::*;
    /// #[derive(Component)]
    /// struct Likes;
    ///
    /// let world = World::new();
    /// let apple = world.entity_named("Apple");
    /// let banana = world.entity_named("Banana");
    ///
    /// let entity = world
    ///     .entity()
    ///     .add_first::<Likes>(apple)
    ///     .add_first::<Likes>(banana);
    ///
    /// // Iterate over all "Likes" relationships
    /// entity.each_pair(world.component_id::<Likes>(), flecs::Wildcard::ID, |id| {
    ///     println!("Entity likes: {}", id.second_id().name());
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::each_component()`] - Iterate over all components
    /// * [`EntityView::each_target()`] - Iterate over relationship targets
    #[doc(alias = "entity_view::each")]
    pub fn each_pair(
        &self,
        first: impl Into<Entity>,
        second: impl Into<Entity>,
        mut func: impl FnMut(IdView),
    ) {
        // this is safe because we are only reading the world
        let real_world = self.world.real_world();

        let Some(table) =
            NonNull::new(unsafe { sys::ecs_get_table(real_world.world_ptr(), *self.id) })
        else {
            return;
        };

        let table = Table::new(real_world, table);

        let mut pattern: sys::ecs_id_t = *first.into();
        let second_id = *second.into();
        if second_id != 0 {
            pattern = ecs_pair(pattern, second_id);
        }

        let mut cur: i32 = 0;
        let archetype = table.archetype();
        let ids = archetype.as_slice();

        while {
            cur = unsafe {
                sys::ecs_search_offset(
                    real_world.world_ptr(),
                    table.table_ptr_mut(),
                    cur,
                    pattern,
                    &mut 0,
                )
            };
            cur != -1
        } {
            let ent = IdView::new_from_id(self.world, ids[cur as usize]);
            func(ent);
            cur += 1;
        }
    }

    /// Iterate over targets for a given relationship.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flecs_ecs::prelude::*;
    /// let world = World::new();
    /// let parent = world.entity_named("Parent");
    ///
    /// // Create some child entities
    /// let child1 = world.entity_named("Child1").child_of_id(parent);
    /// let child2 = world.entity_named("Child2").child_of_id(parent);
    ///
    /// // Iterate over all children
    /// parent.each_target_id(flecs::ChildOf::ID, |child| {
    ///     println!("Found child: {}", child.name());
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::each_component()`] - Iterate over all components
    /// * [`EntityView::each_pair()`] - Iterate over pairs
    /// * [`EntityView::target_id_count()`] - Get number of targets
    #[doc(alias = "entity_view::each")]
    pub fn each_target_id(self, relationship: impl Into<Entity>, mut func: impl FnMut(EntityView)) {
        self.each_pair(relationship.into(), ECS_WILDCARD, |id| {
            let obj = id.second_id();
            func(obj);
        });
    }

    /// Iterate over targets for a given relationship type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Likes;
    ///
    /// let world = World::new();
    /// let entity = world.entity();
    /// let apple = world.entity_named("Apple");
    /// let banana = world.entity_named("Banana");
    ///
    /// entity.add_first::<Likes>(apple).add_first::<Likes>(banana);
    ///
    /// entity.each_target::<Likes>(|target| {
    ///    println!("Entity likes: {}", target.name());
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::each_component()`] - Iterate over all components
    /// * [`EntityView::each_pair()`] - Iterate over pairs
    /// * [`EntityView::target_count()`] - Get number of targets
    /// * [`EntityView::each_target_id()`] - Iterate over targets for a specific relationship
    pub fn each_target<T>(self, func: impl FnMut(EntityView))
    where
        T: ComponentId,
    {
        self.each_target_id(EntityView::new_from(self.world, T::id(self.world)), func);
    }

    /// Get the count of targets for a given relationship.
    ///
    /// Returns the number of entities that are targets of the specified relationship.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity();
    ///
    /// let likes = world.entity();
    ///
    /// let apple = world.entity_named("Apple");
    /// let banana = world.entity_named("Banana");
    ///
    /// entity.add_id((likes,apple)).add_id((likes,banana));
    ///
    /// assert_eq!(entity.target_id_count(likes), Some(2));
    /// ```
    ///
    /// # Returns
    ///
    /// * `Some(count)` if the entity has the relationship
    /// * `None` if the entity doesn't have the relationship
    ///
    /// # See also
    ///
    /// * [`EntityView::target_count()`] - Type-safe version
    /// * [`EntityView::each_target_id()`] - Iterate over targets
    pub fn target_id_count(self, relationship: impl Into<Entity>) -> Option<i32> {
        let world = self.world.real_world().ptr_mut();
        let id = ecs_pair(*relationship.into(), ECS_WILDCARD);
        let table = unsafe { sys::ecs_get_table(self.world.world_ptr(), *self.id) };

        let count = unsafe { sys::ecs_rust_rel_count(world, id, table) };

        if count == -1 { None } else { Some(count) }
    }

    /// Get the count of targets for a given relationship.
    /// Typed version of [`EntityView::target_id_count()`].
    ///
    /// Returns the number of entities that are targets of the specified relationship type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flecs_ecs::prelude::*;
    /// #[derive(Component)]
    /// struct Likes;
    ///
    /// let world = World::new();
    /// let entity = world.entity();
    /// let apple = world.entity_named("Apple");
    /// let banana = world.entity_named("Banana");
    ///
    /// entity.add_first::<Likes>(apple).add_first::<Likes>(banana);
    ///
    /// assert_eq!(entity.target_count::<Likes>(), Some(2));
    /// ```
    ///
    /// # Returns
    ///
    /// * `Some(count)` if the entity has the relationship
    /// * `None` if the entity doesn't have the relationship
    ///
    /// # See also
    ///
    /// * [`EntityView::target_id_count()`] - Non-type-safe version
    /// * [`EntityView::each_target()`] - Iterate over targets
    /// * [`EntityView::has()`] - Check for component/relationship
    pub fn target_count<T>(self) -> Option<i32>
    where
        T: ComponentId,
    {
        self.target_id_count(T::id(self.world))
    }

    /// Iterate children for entity
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    pub fn each_child_of_id(
        self,
        relationship: impl Into<Entity>,
        mut func: impl FnMut(EntityView),
    ) -> bool {
        let mut count = 0;
        // When the entity is a wildcard, this would attempt to query for all
        //entities with (ChildOf, *) or (ChildOf, _) instead of querying for
        //the children of the wildcard entity.
        if self.id == flecs::Wildcard::ID || self.id == flecs::Any::ID {
            // this is correct, wildcard entities don't have children
            return false;
        }

        let mut it: sys::ecs_iter_t =
            unsafe { sys::ecs_each_id(self.world_ptr(), ecs_pair(*relationship.into(), *self.id)) };
        while unsafe { sys::ecs_each_next(&mut it) } {
            count += it.count;
            for i in 0..it.count as usize {
                unsafe {
                    let id = it.entities.add(i);
                    let ent = EntityView::new_from(self.world, *id);
                    func(ent);
                }
            }
        }

        count > 0
    }

    /// Iterate children for entity
    ///
    /// # Arguments
    ///
    /// * T - The relationship to follow
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    pub fn each_child_of<T>(self, func: impl FnMut(EntityView)) -> bool
    where
        T: ComponentId,
    {
        self.each_child_of_id(T::id(self.world), func)
    }

    /// Iterate children for entity
    /// This operation follows the `ChildOf` relationship.
    ///
    /// # Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    ///
    /// # Returns
    ///
    /// Returns `true` if the entity has children, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::children`
    #[doc(alias = "entity_view::children")]
    pub fn each_child(self, func: impl FnMut(EntityView)) -> bool {
        self.each_child_of_id(flecs::ChildOf::ID, func)
    }

    /// Returns if the entity has any children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let parent = world.entity();
    /// let child = world.entity().child_of_id(parent);
    ///
    /// assert!(parent.has_children());
    /// assert!(!child.has_children());
    /// ```
    pub fn has_children(self) -> bool {
        let mut it =
            unsafe { sys::ecs_each_id(self.world_ptr(), ecs_pair(flecs::ChildOf::ID, *self.id)) };
        if unsafe { sys::ecs_iter_is_true(&mut it) } {
            return true;
        }
        false
    }

    /// Returns the count of children for the entity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let parent = world.entity();
    /// let child = world.entity().child_of_id(parent);
    ///
    /// assert_eq!(parent.count_children(), 1);
    ///
    /// let child2 = world.entity().child_of_id(parent);
    ///
    /// assert_eq!(parent.count_children(), 2);
    /// ```
    pub fn count_children(self) -> u32 {
        let mut it = unsafe { sys::ecs_children(self.world_ptr(), *self.id) };
        unsafe { sys::ecs_iter_count(&mut it) as u32 }
    }

    /// Returns the count of targets for a given relationship.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow
    ///
    /// # Example
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let parent = world.entity();
    /// let child = world.entity().child_of_id(parent);
    ///
    /// assert_eq!(parent.count_relationship::<flecs::ChildOf>(), 1);
    ///
    /// let child2 = world.entity().child_of_id(parent);
    ///
    /// assert_eq!(parent.count_relationship::<flecs::ChildOf>(), 2);
    /// ```
    pub fn count_relationship<T: ComponentId>(self) -> u32 {
        let world = self.world;
        let mut it = unsafe {
            sys::ecs_each_id(
                world.world_ptr(),
                ecs_pair(*world.component_id::<T>(), *self.id),
            )
        };

        unsafe { sys::ecs_iter_count(&mut it) as u32 }
    }

    /// Returns the count of targets for a given relationship id.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship id to follow
    ///
    /// # Example
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let parent = world.entity();
    /// let child = world.entity().child_of_id(parent);
    ///
    /// assert_eq!(parent.count_relationship_id(flecs::ChildOf::ID), 1);
    ///
    /// let child2 = world.entity().child_of_id(parent);
    ///
    /// assert_eq!(parent.count_relationship_id(flecs::ChildOf::ID), 2);
    /// ```
    pub fn count_relationship_id(self, relationship: impl Into<Entity>) -> u32 {
        let world = self.world;
        let mut it = unsafe {
            sys::ecs_each_id(world.world_ptr(), ecs_pair(*relationship.into(), *self.id))
        };

        unsafe { sys::ecs_iter_count(&mut it) as u32 }
    }
}

pub trait EntityViewGet<Return> {
    /// gets mutable or immutable component(s) and/or relationship(s) from an entity in a callback and return a value.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// - `try_get` assumes when not using `Option` wrapper, that the entity has the component.
    ///   If it does not, it will not run the callback.
    ///   If unsure and you still want to have the callback be ran, use `Option` wrapper instead.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    ///   tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    ///   The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    ///   This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    ///   creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    ///   In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// # Returns
    ///
    /// - If the callback was run, the return value of the callback wrapped in [`Some`]
    /// - Otherwise, returns [`None`]
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world
    ///     .entity()
    ///     .set(Position { x: 10.0, y: 20.0 })
    ///     .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let val = entity.try_get::<&Position>(|(pos)| pos.x);
    /// assert_eq!(val, Some(10.0));
    ///
    /// let val = entity.try_get::<&Velocity>(|(vel)| vel.x);
    /// assert_eq!(val, None);
    ///
    /// let has_run = entity
    ///     .try_get::<(Option<&Velocity>, &Position)>(|(tag, pos)| {
    ///         assert_eq!(pos.x, 10.0);
    ///         assert!(tag.is_none());
    ///     })
    ///     .is_some();
    /// assert!(has_run);
    ///
    /// let has_run = entity
    ///     .try_get::<(&mut (Tag, Position), &Position)>(|(tag_pos_rel, pos)| {
    ///         assert_eq!(pos.x, 10.0);
    ///         assert_eq!(tag_pos_rel.x, 30.0);
    ///     })
    ///     .is_some();
    /// assert!(has_run);
    /// ```
    fn try_get<T: GetTuple>(
        self,
        callback: impl for<'e> FnOnce(T::TupleType<'e>) -> Return,
    ) -> Option<Return>;

    /// gets mutable or immutable component(s) and/or relationship(s) from an entity in a callback and return a value.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    ///   tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    ///   The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    ///   This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    ///   creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    ///   In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// - `get` assumes when not using `Option` wrapper, that the entity has the component.
    ///   This will panic if the entity does not have the component. If unsure, use `Option` wrapper or `try_get` function instead.
    ///   `try_get` does not run the callback if the entity does not have the component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world
    ///     .entity()
    ///     .set(Position { x: 10.0, y: 20.0 })
    ///     .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let val = entity.get::<&Position>(|(pos)| pos.x);
    /// assert_eq!(val, 10.0);
    ///
    /// entity.get::<(Option<&Velocity>, &Position)>(|(vel, pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     assert!(vel.is_none());
    /// });
    ///
    /// entity.get::<(&mut (Tag, Position), &Position)>(|(tag_pos_rel, pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     assert_eq!(tag_pos_rel.x, 30.0);
    /// });
    /// ```
    fn get<T: GetTuple>(self, callback: impl for<'e> FnOnce(T::TupleType<'e>) -> Return) -> Return;
}

impl<Return> EntityViewGet<Return> for EntityView<'_> {
    fn try_get<T: GetTuple>(
        self,
        callback: impl for<'e> FnOnce(T::TupleType<'e>) -> Return,
    ) -> Option<Return> {
        let record = unsafe { sys::ecs_record_find(self.world.world_ptr(), *self.id) };

        if unsafe { (*record).table.is_null() } {
            return None;
        }

        let tuple_data = T::create_ptrs::<false>(self.world, self.id, record);
        let has_all_components = tuple_data.has_all_components();

        if has_all_components {
            let tuple = tuple_data.get_tuple();

            #[cfg(feature = "flecs_safety_readwrite_locks")]
            {
                let table_id = unsafe { ecs_rust_table_id((*record).table) };
                let ids = tuple_data.read_write_ids();
                let components_access = self.world().components_access_map();
                components_access.increment_counters_from_ids(ids, table_id);

                self.world.defer_begin();
                let ret = callback(tuple);
                self.world.defer_end();

                components_access.decrement_counters_from_ids(ids, table_id);
                Some(ret)
            }

            #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
            {
                self.world.defer_begin();
                let ret = callback(tuple);
                self.world.defer_end();
                Some(ret)
            }
        } else {
            None
        }
    }

    fn get<T: GetTuple>(self, callback: impl for<'e> FnOnce(T::TupleType<'e>) -> Return) -> Return {
        let record = unsafe { sys::ecs_record_find(self.world.world_ptr(), *self.id) };

        if unsafe { (*record).table.is_null() } {
            panic!("Entity does not have any components");
        }

        let tuple_data = T::create_ptrs::<true>(self.world, self.id, record);
        let tuple = tuple_data.get_tuple();

        #[cfg(feature = "flecs_safety_readwrite_locks")]
        {
            let table_id = unsafe { ecs_rust_table_id((*record).table) };
            let ids = tuple_data.read_write_ids();
            let components_access = self.world().components_access_map();
            components_access.increment_counters_from_ids(ids, table_id);

            self.world.defer_begin();
            let ret = callback(tuple);
            self.world.defer_end();

            components_access.decrement_counters_from_ids(ids, table_id);
            ret
        }

        #[cfg(not(feature = "flecs_safety_readwrite_locks"))]
        {
            self.world.defer_begin();
            let ret = callback(tuple);
            self.world.defer_end();
            ret
        }
    }
}

impl<'a> EntityView<'a> {
    /// Clones components and/or relationship(s) from an entity and returns it.
    /// each component type must be marked `&`. This helps Rust type checker to determine if it's a relationship.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot clone single component tags with this function.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    ///
    /// # Panics
    ///
    /// - `cloned` assumes when not using `Option` wrapper, that the entity has the component.
    ///   This will panic if the entity does not have the component. If unsure, use `Option` wrapper or `try_cloned` function instead.
    ///   `try_cloned` will return a `None` tuple instead if the entity does not have the component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world
    ///     .entity()
    ///     .set(Position { x: 10.0, y: 20.0 })
    ///     .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let pos = entity.cloned::<&Position>();
    /// assert_eq!(pos.x, 10.0);
    ///
    /// let (vel, pos) = entity.cloned::<(Option<&Velocity>, &Position)>();
    /// assert_eq!(pos.x, 10.0);
    /// assert!(vel.is_none());
    ///
    /// let (tag_pos_rel, pos) = entity.cloned::<(&(Tag, Position), &Position)>();
    /// assert_eq!(pos.x, 10.0);
    /// assert_eq!(tag_pos_rel.x, 30.0);
    /// ```
    #[must_use]
    pub fn cloned<T: ClonedTuple>(self) -> T::TupleType<'a> {
        let record = unsafe { sys::ecs_record_find(self.world.world_ptr(), *self.id) };

        if unsafe { (*record).table.is_null() } {
            panic!("Entity does not have any components");
        }

        let tuple_data = T::create_ptrs::<true>(self.world, self.id, record);

        #[cfg(feature = "flecs_safety_readwrite_locks")]
        {
            let table_id = unsafe { ecs_rust_table_id((*record).table) };
            let read_ids = tuple_data.read_ids();
            let components_access = self.world().components_access_map();
            components_access.panic_if_any_write_is_set(read_ids, table_id);
        }
        tuple_data.get_tuple()
    }

    /// Clones components and/or relationship(s) from an entity and returns an `Option`.
    /// `None` if the entity does not have all components that are not marked `Option`, otherwise `Some(tuple)`.
    /// each component type must be marked `&`. This helps Rust type checker to determine if it's a relationship.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot clone single component tags with this function.
    /// - You can only clone relationships with a payload, so where one is not a tag / not a zst.
    ///
    /// # Returns
    ///
    /// - `Some(tuple)` if the entity has all components, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world
    ///     .entity()
    ///     .set(Position { x: 10.0, y: 20.0 })
    ///     .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let pos = entity.try_cloned::<&Position>();
    /// assert!(pos.is_some());
    /// assert_eq!(pos.unwrap().x, 10.0);
    ///
    /// let (vel, pos) = entity
    ///     .try_cloned::<(Option<&Velocity>, &Position)>()
    ///     .unwrap();
    /// assert_eq!(pos.x, 10.0);
    /// assert!(vel.is_none());
    ///
    /// let (tag_pos_rel, pos) = entity
    ///     .try_cloned::<(&(Tag, Position), &Position)>()
    ///     .unwrap();
    /// assert_eq!(pos.x, 10.0);
    /// assert_eq!(tag_pos_rel.x, 30.0);
    /// ```
    pub fn try_cloned<T: ClonedTuple>(self) -> Option<T::TupleType<'a>> {
        let record = unsafe { sys::ecs_record_find(self.world.world_ptr(), *self.id) };

        if unsafe { (*record).table.is_null() } {
            return None;
        }

        let tuple_data = T::create_ptrs::<false>(self.world, self.id, record);
        //todo we can maybe early return if we don't yet if doesn't have all. Same for try_get
        let has_all_components = tuple_data.has_all_components();

        if has_all_components {
            Some(tuple_data.get_tuple())
        } else {
            None
        }
    }

    /// Get component value or pair as untyped pointer
    ///
    /// # Arguments
    ///
    /// * `component_id` - The component to get
    ///
    /// # Returns
    ///
    /// * `*const c_void` - Pointer to the component value, nullptr if the entity does not have the component
    ///
    /// # Safety
    ///
    /// Ensure the pointer is valid before use. The caller must know the actual type to cast the pointer correctly.
    /// The pointer might get invalided if the table alters.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_untyped(self, component_id: impl IntoId) -> *const c_void {
        unsafe { sys::ecs_get_id(self.world.world_ptr(), *self.id, *component_id.into()) }
    }

    /// Get the pair value as untyped pointer.
    /// This function does not cast the pointer to the actual type, that's up to the caller.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first component type in the pair to get
    ///
    /// # Arguments
    ///
    /// * `second` - The second entity in the component pair
    ///
    /// # Returns
    ///
    /// * `*const c_void` - Pointer to the component value, `nullptr` if the entity does not have the component pair
    ///
    /// # Safety
    ///
    /// Ensure the pointer is valid before use. The caller must know the actual type to cast the pointer correctly.
    /// The pointer might get invalided if the table alters.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_first_untyped<First: ComponentId>(self, second: impl Into<Entity>) -> *const c_void {
        unsafe {
            sys::ecs_get_id(
                self.world.world_ptr(),
                *self.id,
                ecs_pair(First::id(self.world), *second.into()),
            )
        }
    }

    /// Get the pair value as untyped pointer.
    /// This function does not cast the pointer to the actual type, that's up to the caller.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second component type in the pair to get
    ///
    /// # Arguments
    ///
    /// * `first` - The first entity in the component pair
    ///
    /// # Returns
    ///
    /// * `*const c_void` - Pointer to the component value, `nullptr` if the entity does not have the component pair
    ///
    /// # Safety
    ///
    /// Ensure the pointer is valid before use. The caller must know the actual type to cast the pointer correctly.
    /// The pointer might get invalided if the table alters.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    pub fn get_second_untyped<Second: ComponentId>(
        self,
        first: impl Into<Entity>,
    ) -> *const c_void {
        unsafe {
            sys::ecs_get_id(
                self.world.world_ptr(),
                *self.id,
                ecs_pair(*first.into(), Second::id(self.world)),
            )
        }
    }

    /// Get mutable component value or pair (untyped).
    /// This operation returns a mutable ref to the component. If a base entity had
    /// the component, it will be overridden, and the value of the base component
    /// will be copied to the entity before this function returns.
    ///
    /// # Arguments
    ///
    /// * `comp`: The component to get.
    ///
    /// # Returns
    ///
    /// Pointer to the component value.
    ///
    /// # Safety
    ///
    /// Ensure the pointer is valid before use. The caller must know the actual type to cast the pointer correctly.
    /// The pointer might get invalided if the table alters.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get_mut`
    #[doc(alias = "entity_view::get_mut")]
    pub fn get_untyped_mut(self, id: impl IntoId) -> *mut c_void {
        unsafe { sys::ecs_get_mut_id(self.world.world_ptr(), *self.id(), *id.into()) }
    }

    /// Get mutable pair value as untyped pointer.
    /// This function does not cast the pointer to the actual type, that's up to the caller.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first component type in the pair to get
    ///
    /// # Arguments
    ///
    /// * `second` - The second entity in the component pair
    ///
    /// # Returns
    ///
    /// * `*mut c_void` - Pointer to the component value, `nullptr` if the entity does not have the component pair
    ///
    /// # Safety
    ///
    /// Ensure the pointer is valid before use. The caller must know the actual type to cast the pointer correctly.
    /// The pointer might get invalided if the table alters.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get_mut`
    #[doc(alias = "entity_view::get_mut")]
    pub fn get_first_untyped_mut<First: ComponentId>(
        self,
        second: impl Into<Entity>,
    ) -> *mut c_void {
        unsafe {
            sys::ecs_get_mut_id(
                self.world.world_ptr(),
                *self.id,
                ecs_pair(First::id(self.world), *second.into()),
            )
        }
    }

    /// This function does not cast the pointer to the actual type, that's up to the caller.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second component type in the pair to get
    ///
    /// # Arguments
    ///
    /// * `first` - The first entity in the component pair
    ///
    /// # Returns
    ///
    /// * `*mut c_void` - Pointer to the component value, `nullptr` if the entity does not have the component pair
    ///
    /// # Safety
    ///
    /// Ensure the pointer is valid before use. The caller must know the actual type to cast the pointer correctly.
    /// The pointer might get invalided if the table alters.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get_mut`
    #[doc(alias = "entity_view::get_mut")]
    pub fn get_second_untyped_mut<Second: ComponentId>(
        self,
        first: impl Into<Entity>,
    ) -> *mut c_void {
        unsafe {
            sys::ecs_get_mut_id(
                self.world.world_ptr(),
                *self.id,
                ecs_pair(*first.into(), Second::id(self.world)),
            )
        }
    }

    /// Get target for a given pair.
    ///
    /// This operation returns the target for a given pair. The optional
    /// index can be used to iterate through targets, in case the entity `get_has`
    /// multiple instances for the same relationship.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `index` - The index (0 for the first instance of the relationship).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target")]
    pub fn target<First: ComponentId>(self, index: i32) -> Option<EntityView<'a>> {
        let id = unsafe {
            sys::ecs_get_target(
                self.world.world_ptr(),
                *self.id,
                First::id(self.world),
                index,
            )
        };
        if id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world, id))
        }
    }

    /// Get target for a given pair.
    ///
    /// This operation returns the target for a given pair. The optional
    /// index can be used to iterate through targets, in case the entity `get_has`
    /// multiple instances for the same relationship.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair for which to retrieve the target.
    /// * `index` - The index (0 for the first instance of the relationship).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target")]
    pub fn target_id(self, first: impl Into<Entity>, index: i32) -> Option<EntityView<'a>> {
        let id =
            unsafe { sys::ecs_get_target(self.world.world_ptr(), *self.id, *first.into(), index) };
        if id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world, id))
        }
    }

    /// Get the target of a pair for a given relationship id.
    ///
    /// This operation returns the first entity that has the provided id by following
    /// the specified relationship. If the entity itself has the id then entity will
    /// be returned. If the id cannot be found on the entity or by following the
    /// relationship, the operation will return 0.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow.
    /// * `id` - The id to lookup.
    ///
    /// # Returns
    ///
    /// * The entity for which the target `get_has` been found.
    ///
    /// # See also
    ///
    /// * [`EntityView::target_for()`]
    /// * C++ API: `entity_view::target_for`
    #[doc(alias = "entity_view::target_for")]
    pub fn target_for_id(
        &self,
        relationship: impl Into<Entity>,
        component_id: impl IntoId,
    ) -> Option<EntityView<'a>> {
        let id = unsafe {
            sys::ecs_get_target_for_id(
                self.world.world_ptr(),
                *self.id,
                *relationship.into(),
                *component_id.into(),
            )
        };
        if id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world, id))
        }
    }

    /// Get the target for a given component and relationship.
    ///
    /// This function is a convenient wrapper around `get_target_by_relationship_and_component_id`,
    /// allowing callers to provide a type and automatically deriving the component id.
    ///
    /// This operation can be used to lookup, for example, which prefab is providing
    /// a component by specifying the `IsA` pair:
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// # let world = World::new();
    /// # let entity = world.entity();
    /// # #[derive(Component)]
    /// # struct Position(f32, f32, f32);
    /// // Is Position provided by the entity or one of its base entities?
    /// let e = entity.target_for::<Position>(flecs::IsA::ID);
    /// ```
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to use for deriving the id.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to follow.
    ///
    /// # Returns
    ///
    /// * The entity for which the target `get_has` been found.
    ///
    /// # See also
    ///
    /// * [`EntityView::target_for_id()`]
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target_for")]
    #[inline(always)]
    pub fn target_for<T: ComponentOrPairId>(
        self,
        relationship: impl Into<Entity>,
    ) -> Option<EntityView<'a>> {
        self.target_for_id(relationship, T::get_id(self.world))
    }

    // TODO this needs a better name and documentation, the rest of the cpp functions still have to be done as well
    // TODO, I removed the second template parameter and changed the fn parameter second to entityT, check validity
    /// Get the target for a given pair of components and a relationship.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first component type to use for deriving the id.
    ///
    /// # Arguments
    ///
    /// * `second` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// * The entity for which the target has been found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target_for")]
    // TODO needs to be made safe
    pub(crate) fn target_for_first<First: ComponentId + DataComponent>(
        &self,
        second: impl Into<Entity>,
    ) -> *const First {
        let comp_id = First::id(self.world);
        ecs_assert!(
            core::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "First element is size 0"
        );
        unsafe {
            sys::ecs_get_id(
                self.world.world_ptr(),
                comp_id,
                ecs_pair(comp_id, *second.into()),
            ) as *const First
        }
    }

    /// Get the depth for the given relationship.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship for which to get the depth.
    ///
    /// # Returns
    ///
    /// * The depth of the relationship.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::depth`
    #[doc(alias = "entity_view::depth")]
    #[inline(always)]
    pub fn depth_id(self, relationship: impl Into<Entity>) -> i32 {
        unsafe { sys::ecs_get_depth(self.world.world_ptr(), *self.id, *relationship.into()) }
    }

    /// Retrieves the depth for a specified relationship.
    ///
    /// This function is a convenient wrapper around `get_depth_id`, allowing callers
    /// to provide a type and automatically deriving the relationship id.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The relationship type to use for deriving the id.
    ///
    /// # Returns
    ///
    /// * The depth of the relationship.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::depth`
    #[doc(alias = "entity_view::depth")]
    #[inline(always)]
    pub fn depth<T: ComponentId>(self) -> i32 {
        self.depth_id(T::id(self.world))
    }

    /// Retrieves the parent of the entity.
    ///
    /// This function is shorthand for getting the target using the `EcsChildOf` relationship.
    ///
    /// # Returns
    ///
    /// * The parent of the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::parent`
    #[doc(alias = "entity_view::parent")]
    #[inline(always)]
    pub fn parent(self) -> Option<EntityView<'a>> {
        self.target_id(ECS_CHILD_OF, 0)
    }

    /// Lookup an entity by name.
    ///
    /// Lookup an entity in the scope of this entity. The provided path may
    /// contain double colons as scope separators, for example: "`Foo::Bar`".
    ///
    /// # Arguments
    ///
    /// * `path` - The name of the entity to lookup.
    /// * `recursively` - Recursively traverse up the tree until entity is found.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise `None`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    fn try_lookup_impl(self, name: &str, recursively: bool) -> Option<EntityView<'a>> {
        let name = compact_str::format_compact!("{}\0", name);

        ecs_assert!(
            self.id != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid lookup from null handle"
        );
        let id = unsafe {
            sys::ecs_lookup_path_w_sep(
                self.world.world_ptr(),
                *self.id,
                name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                recursively,
            )
        };

        if id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world, id))
        }
    }

    /// Lookup an entity by name.
    /// The entity is searched recursively recursively traversing
    /// up the tree until found.
    ///
    /// The provided path may contain double colons as scope separators,
    /// for example: "`Foo::Bar`".
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise `None`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    pub fn try_lookup_recursive(&self, name: &str) -> Option<EntityView> {
        self.try_lookup_impl(name, true)
    }

    /// Lookup an entity by name, only in the current scope of the entity.
    ///
    /// Lookup an entity in the scope of this entity. The provided path may
    /// contain double colons as scope separators, for example: "`Foo::Bar`".
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise `None`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    pub fn try_lookup(&self, name: &str) -> Option<EntityView> {
        self.try_lookup_impl(name, false)
    }

    /// Lookup an entity by name.
    /// The entity is searched recursively recursively traversing
    /// up the tree until found.
    ///
    /// The provided path may contain double colons as scope separators,
    /// for example: "`Foo::Bar`".
    ///
    /// # Safety
    ///
    /// This function can return an entity with id 0 if the entity is not found.
    /// Ensure that the entity exists before using it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity, entity id will be 0 if not found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    pub fn lookup_recursive(&self, name: &str) -> EntityView {
        self.try_lookup_recursive(name).unwrap_or_else(|| {
            panic!(
                "Entity {} not found, when unsure, use try_lookup_recursive",
                name
            )
        })
    }

    /// Lookup an entity by name, only in the current scope of the entity.
    ///
    /// Lookup an entity in the scope of this entity. The provided path may
    /// contain double colons as scope separators, for example: "`Foo::Bar`".
    ///
    /// # Safety
    ///
    /// This function can return an entity with id 0 if the entity is not found.
    /// Ensure that the entity exists before using it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity, entity id will be 0 if not found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    pub fn lookup(&self, name: &str) -> EntityView {
        self.try_lookup(name)
            .unwrap_or_else(|| panic!("Entity {} not found, when unsure, use try_lookup", name))
    }

    /// Test if an entity has an id.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check.
    ///
    /// # Returns
    ///
    /// True if the entity has or inherits the provided id, false otherwise.
    ///
    /// # See also
    ///
    /// * [`EntityView::has()`]
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    #[inline(always)]
    pub fn has_id(self, id: impl IntoId) -> bool {
        unsafe { sys::ecs_has_id(self.world.world_ptr(), *self.id, *id.into()) }
    }

    /// Check if entity has the provided component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to check.
    ///
    /// # Returns
    ///
    /// True if the entity has or inherits the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * [`EntityView::has_id()`]
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has<T: ComponentOrPairId>(self) -> bool {
        if !T::IS_ENUM {
            unsafe { sys::ecs_has_id(self.world.world_ptr(), *self.id, T::get_id(self.world)) }
        } else {
            let component_id = T::get_id(self.world);
            self.has_id((component_id, ECS_WILDCARD))
        }
    }

    /// Check if entity has the provided enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `constant` - The enum constant to check.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided constant, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has_enum<T>(self, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let component_id: sys::ecs_id_t = T::id(self.world);
        // Safety: we know the enum fields are registered because of the previous T::id call
        let enum_constant_entity_id = unsafe { constant.id_variant_unchecked(self.world) };

        ecs_assert!(
            *enum_constant_entity_id.id != 0,
            FlecsErrorCode::InvalidParameter,
            "Constant was not found in Enum reflection data. Did you mean to use has<E>() instead of has(E)?"
        );

        self.has_id((component_id, enum_constant_entity_id))
    }

    // this is pub(crate) because it's used for development purposes only
    pub(crate) fn has_enum_id<T>(self, enum_id: impl Into<Entity>, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let enum_constant_entity_id = constant.id_variant(self.world);
        self.has_id((enum_id.into(), enum_constant_entity_id))
    }

    /// Check if entity has the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has_first<First: ComponentId>(self, second: impl Into<Entity>) -> bool {
        self.has_id((First::id(self.world), second.into()))
    }

    /// Check if entity has the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has_second<Second: ComponentId>(self, first: impl Into<Entity>) -> bool {
        self.has_id((first.into(), Second::id(self.world)))
    }

    /// Check if entity has the provided pair with an enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The first element of the pair.
    /// * `U` - The second element of the pair as an enum constant.
    ///
    /// # Arguments
    ///
    /// * `constant` - The enum constant.
    ///
    /// # Returns
    ///
    /// True if the entity has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::has`
    #[doc(alias = "entity_view::has")]
    pub fn has_pair_enum<T: ComponentId, U: ComponentId + EnumComponentInfo>(
        &self,
        constant: U,
    ) -> bool {
        let component_id: sys::ecs_id_t = T::id(self.world);
        let enum_constant_entity_id = constant.id_variant(self.world);

        self.has_id((component_id, enum_constant_entity_id))
    }

    /// Check if the entity owns the provided entity (pair, component, entity).
    /// An entity is owned if it is not shared from a base entity.
    ///
    /// # Arguments
    /// - `entity_id`: The entity to check.
    ///
    /// # Returns
    /// - `true` if the entity owns the provided entity, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn owns_id(self, entity_id: impl IntoId) -> bool {
        unsafe { sys::ecs_owns_id(self.world.world_ptr(), *self.id, *entity_id.into()) }
    }

    /// Check if the entity owns the provided component.
    /// A component is owned if it is not shared from a base entity.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component to check.
    ///
    /// # Returns
    ///
    /// - `true` if the entity owns the provided component, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn owns<T: ComponentOrPairId>(self) -> bool {
        unsafe { sys::ecs_owns_id(self.world.world_ptr(), *self.id, T::get_id(self.world)) }
    }

    /// Check if the entity owns the provided pair.
    /// A pair is owned if it is not shared from a base entity.
    ///
    /// # Type Parameters
    /// - `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `second`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// - `true` if the entity owns the provided pair, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn owns_first<First: ComponentId>(self, second: impl Into<Entity>) -> bool {
        unsafe {
            sys::ecs_owns_id(
                self.world.world_ptr(),
                *self.id,
                ecs_pair(First::id(self.world), *second.into()),
            )
        }
    }

    /// Check if the entity owns the provided pair.
    /// A pair is owned if it is not shared from a base entity.
    ///
    /// # Type Parameters
    /// - `Second`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `first`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// - `true` if the entity owns the provided pair, `false` otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::owns`
    #[doc(alias = "entity_view::owns")]
    pub fn owns_second<Second: ComponentId>(self, first: impl Into<Entity>) -> bool {
        unsafe {
            sys::ecs_owns_id(
                self.world.world_ptr(),
                *self.id,
                ecs_pair(*first.into(), Second::id(self.world)),
            )
        }
    }

    /// Test if id is enabled.
    ///
    /// # Arguments
    /// - `id`: The id to test.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_id(self, id: impl IntoId) -> bool {
        unsafe { sys::ecs_is_enabled_id(self.world.world_ptr(), *self.id, *id.into()) }
    }

    /// Test if component is enabled.
    ///
    /// # Type Parameters
    /// - `T`: The component to test.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled<T: ComponentOrPairId>(self) -> bool {
        unsafe { sys::ecs_is_enabled_id(self.world.world_ptr(), *self.id, T::get_id(self.world)) }
    }

    /// Test if pair is enabled.
    ///
    /// # Type Parameters
    /// - `T`: The first element of the pair.
    ///
    /// # Arguments
    /// - `second`: The second element of the pair.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_first<First: ComponentId>(self, second: impl Into<Entity>) -> bool {
        self.is_enabled_id((First::id(self.world), second.into()))
    }

    /// Test if pair is enabled.
    ///
    /// # Type Parameters
    /// - `T`: The second element of the pair.
    ///
    /// # Arguments
    /// - `first`: The second element of the pair.
    ///
    /// # Returns
    /// - `true` if enabled, `false` if not.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::enabled`
    #[doc(alias = "entity_view::enabled")]
    pub fn is_enabled_second<Second: ComponentId>(self, first: impl Into<Entity>) -> bool {
        self.is_enabled_id((first.into(), Second::id(self.world)))
    }

    /// Clones the current entity to a new or specified entity.
    ///
    /// This function creates a clone of the current entity. If `dest_id` is provided
    /// (i.e., not zero), it will clone the current entity to the entity with `dest_id`.
    /// If `dest_id` is zero, it will create a new entity and clone the current entity
    /// to the newly created entity.
    ///
    /// If `copy_value` is set to `true`, the value of the current entity is also copied to
    /// the destination entity. Otherwise, only the entity's structure is cloned without copying the value.
    ///
    /// # Arguments
    /// - `copy_value`: A boolean indicating whether to copy the entity's value to the destination entity.
    /// - `dest_id`: The identifier of the destination entity. If zero, a new entity is created.
    ///
    /// # Returns
    /// - An `Entity` object representing the destination entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::clone`
    #[doc(alias = "entity_view::clone")]
    #[inline(always)]
    pub fn duplicate(self, copy_value: bool) -> EntityView<'a> {
        let dest_entity = EntityView::new(self.world());
        unsafe {
            sys::ecs_clone(
                self.world.world_ptr_mut(),
                *dest_entity.id,
                *self.id,
                copy_value,
            )
        };
        dest_entity
    }

    /// Clones the current entity to a new or specified entity.
    ///
    /// This function creates a clone of the current entity. If `dest_id` is provided
    /// (i.e., not zero), it will clone the current entity to the entity with `dest_id`.
    /// If `dest_id` is zero, it will create a new entity and clone the current entity
    /// to the newly created entity.
    ///
    /// If `copy_value` is set to `true`, the value of the current entity is also copied to
    /// the destination entity. Otherwise, only the entity's structure is cloned without copying the value.
    ///
    /// # Arguments
    /// - `copy_value`: A boolean indicating whether to copy the entity's value to the destination entity.
    /// - `dest_id`: The identifier of the destination entity. If zero, a new entity is created.
    ///
    /// # Returns
    /// - An `Entity` object representing the destination entity.
    ///
    /// # Safety
    /// This function makes use of `unsafe` operations to interact with the underlying ECS.
    /// Ensure that the provided `dest_id` is valid or zero
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::clone`
    #[doc(alias = "entity_view::clone")]
    #[inline(always)]
    pub fn duplicate_into(self, copy_value: bool, dest_id: impl Into<Entity>) -> EntityView<'a> {
        let mut dest_id = *dest_id.into();
        if dest_id == 0 {
            dest_id = unsafe { sys::ecs_new(self.world.world_ptr_mut()) };
        }

        let dest_entity = EntityView::new_from(self.world, dest_id);
        unsafe { sys::ecs_clone(self.world.world_ptr_mut(), dest_id, *self.id, copy_value) };
        dest_entity
    }

    /// Returns a mutable entity handle for the current stage.
    ///
    /// When an entity handle created from the world is used while the world is
    /// in staged mode, it will only allow for readonly operations since
    /// structural changes are not allowed on the world while in staged mode.
    ///
    /// To perform mutations on the entity, this operation provides a handle to the
    /// entity that uses the stage instead of the actual world.
    ///
    /// Note that staged entity handles should never be stored persistently, in
    /// components or elsewhere. An entity handle should always point to the
    /// main world.
    ///
    /// Also note that this operation is not necessary when doing mutations on an
    /// entity outside of a system. It is allowed to perform entity operations
    /// directly on the world, as long as the world is not in staged mode.
    ///
    /// # Arguments
    /// - `stage`: The current stage.
    ///
    /// # Returns
    /// - An entity handle that allows for mutations in the current stage.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::mut`
    #[doc(alias = "entity_view::mut")]
    pub fn mut_current_stage(self, stage: impl WorldProvider<'a>) -> EntityView<'a> {
        ecs_assert!(
            !stage.world().is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use readonly world/stage to create mutable handle"
        );

        EntityView::new_from(stage, *self.id)
    }

    /// Returns a mutable entity handle for the current stage from another entity.
    ///
    /// This operation allows for the construction of a mutable entity handle
    /// from another entity. This is useful in `each` functions, which only
    /// provide a handle to the entity being iterated over.
    ///
    /// # Arguments
    /// - `entity`: Another mutable entity.
    ///
    /// # Returns
    /// - An entity handle that allows for mutations in the current stage.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::mut`
    #[doc(alias = "entity_view::mut")]
    pub fn mut_stage_of<T>(self, entity: T) -> EntityView<'a>
    where
        T: Into<Entity> + WorldProvider<'a>,
    {
        ecs_assert!(
            !entity.world().is_readonly(),
            FlecsErrorCode::InvalidParameter,
            "cannot use entity created for readonly world/stage to create mutable handle"
        );

        EntityView::new_from(entity.world(), *self.id)
    }

    //might not be needed, in the original c++ impl it was used in the get_mut functions.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::set_stage`
    #[doc(alias = "entity_view::set_stage")]
    #[doc(hidden)]
    fn set_stage(self, stage: impl WorldProvider<'a>) -> EntityView<'a> {
        EntityView::new_from(stage, *self.id)
    }
}

// Event mixin
impl EntityView<'_> {
    /// Emit event for entity.
    ///
    /// # Safety
    /// Caller must ensure that any type associated with `event` is a ZST
    ///
    /// # Arguments
    ///
    /// * event - the event to emit
    ///
    /// # See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue_id()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_view::emit`
    #[doc(alias = "entity_view::emit")]
    pub unsafe fn emit_id(self, event: impl Into<Entity>) {
        unsafe {
            self.world().event_id(event).entity(self).emit(&());
        }
    }

    /// Emit event with an immutable payload for entity.
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to emit.
    ///
    /// # See also
    ///
    /// * [`EntityView::emit_id()`]
    /// * [`EntityView::enqueue_id()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_view::emit`
    #[doc(alias = "entity_view::emit")]
    pub fn emit<T: ComponentId>(self, event: &T) {
        self.world().event().entity(self).emit(event);
    }

    /// Enqueue event for entity.
    ///
    /// # Safety
    ///
    ///
    /// # Arguments
    ///
    /// * event - the event to enqueue
    ///
    /// # See also
    ///
    /// * [`EntityView::emit_id()`]
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    pub unsafe fn enqueue_id(self, event: impl Into<Entity>) {
        unsafe {
            self.world().event_id(event).entity(self).enqueue(());
        }
    }

    /// Enqueue event for entity.
    ///
    /// # Type Parameters
    ///
    /// * T - the event type to enqueue.
    ///
    /// # Usage:
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// # let world = World::new();
    /// # let entity = world.entity();
    /// #[derive(Component)]
    /// struct Resize {
    ///     width: i32,
    ///     height: i32,
    /// }
    ///
    /// world.defer(|| {
    ///     entity.enqueue(Resize {
    ///         width: 10,
    ///         height: 20,
    ///     });
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::emit_id()`]
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue_id()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_view::enqueue`
    #[doc(alias = "entity_view::enqueue")]
    pub fn enqueue<T: ComponentId>(self, event: T) {
        self.world().event().entity(self).enqueue(event);
    }
}

// Event/Observe mixin
impl EntityView<'_> {
    /// Register the callback for the entity observer for empty events.
    ///
    /// The "empty" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func()`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe_entity()`]
    /// * [`EntityView::observe_payload_entity()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe<C>(self, func: impl FnMut() + 'static) -> Self
    where
        C: ComponentId + TagComponent,
    {
        self.observe_impl::<C, _>(func)
    }

    fn observe_impl<C, Func>(self, func: Func) -> Self
    where
        Func: FnMut() + 'static,
        C: ComponentId,
    {
        let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.empty = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_empty = Some(Self::on_free_empty);

        Self::entity_observer_create(
            self.world.world_ptr_mut(),
            C::id(self.world),
            *self.id,
            binding_ctx,
            Some(Self::run_empty::<Func> as unsafe extern "C-unwind" fn(_)),
        );
        self
    }

    /// Register the callback for the entity observer for empty events with entity parameter.
    ///
    /// The `empty_entity` iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut EntityView)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload_entity()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe_entity<C>(self, func: impl FnMut(&mut EntityView) + 'static) -> Self
    where
        C: ComponentId + TagComponent,
    {
        self.observe_entity_impl::<C, _>(func)
    }

    fn observe_entity_impl<C, Func>(self, func: Func) -> Self
    where
        Func: FnMut(&mut EntityView) + 'static,
        C: ComponentId,
    {
        let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.empty_entity = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_empty_entity = Some(Self::on_free_empty_entity);

        Self::entity_observer_create(
            self.world.world_ptr_mut(),
            C::id(self.world),
            *self.id,
            binding_ctx,
            Some(Self::run_empty_entity::<Func> as unsafe extern "C-unwind" fn(_)),
        );
        self
    }

    /// Register the callback for the entity observer for `payload` events.
    ///
    /// The "payload" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut EventData)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe_entity()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload_entity()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe_payload<C>(self, func: impl FnMut(&C) + 'static) -> Self
    where
        C: ComponentId + DataComponent,
    {
        self.observe_payload_impl::<C, _>(func)
    }

    fn observe_payload_impl<C, Func>(self, func: Func) -> Self
    where
        Func: FnMut(&C) + 'static,
        C: ComponentId,
    {
        let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.payload = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_payload = Some(Self::on_free_payload::<C>);

        Self::entity_observer_create(
            self.world.world_ptr_mut(),
            C::id(self.world),
            *self.id,
            binding_ctx,
            Some(Self::run_payload::<C, Func> as unsafe extern "C-unwind" fn(_)),
        );
        self
    }

    /// Register the callback for the entity observer for an event with payload and entity parameter.
    ///
    /// The "payload" iterator accepts a function that is invoked for each matching event.
    /// The following function signature is valid:
    ///  - `func(&mut EntityView, &mut EventData)`
    ///
    /// # Arguments
    ///
    /// * `func` - The callback function
    ///
    /// See also
    ///
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`EntityView::observe_entity()`]
    /// * [`EntityView::observe()`]
    /// * [`EntityView::observe_payload()`]
    /// * [`World::event_id()`]
    /// * [`World::event()`]
    /// * C++ API: `entity_builder::observe`
    #[doc(alias = "entity_builder::observe")]
    pub fn observe_payload_entity<C>(self, func: impl FnMut(&mut EntityView, &C) + 'static) -> Self
    where
        C: ComponentId + DataComponent,
    {
        self.observe_payload_entity_impl::<C, _>(func)
    }

    fn observe_payload_entity_impl<C, Func>(self, func: Func) -> Self
    where
        Func: FnMut(&mut EntityView, &C) + 'static,
        C: ComponentId,
    {
        let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
        let binding_ctx = Box::leak(new_binding_ctx);

        let empty_func = Box::new(func);
        let empty_static_ref = Box::leak(empty_func);

        binding_ctx.payload_entity = Some(empty_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_payload_entity = Some(Self::on_free_payload_entity::<C>);

        Self::entity_observer_create(
            self.world.world_ptr_mut(),
            C::id(self.world),
            *self.id,
            binding_ctx,
            Some(Self::run_payload_entity::<C, Func> as unsafe extern "C-unwind" fn(_)),
        );
        self
    }
}

// entity observer creation
impl EntityView<'_> {
    pub(crate) fn entity_observer_create(
        world: *mut sys::ecs_world_t,
        event: sys::ecs_entity_t,
        entity: sys::ecs_entity_t,
        binding_ctx: *mut ObserverEntityBindingCtx,
        callback: sys::ecs_iter_action_t,
    ) {
        let mut desc = sys::ecs_observer_desc_t::default();
        desc.events[0] = event;
        desc.query.terms[0].id = ECS_ANY;
        desc.query.terms[0].src.id = entity;
        desc.callback = callback;
        desc.callback_ctx = binding_ctx as *mut c_void;
        desc.callback_ctx_free = Some(Self::binding_entity_ctx_drop);

        let observer = unsafe { sys::ecs_observer_init(world, &desc) };
        ecs_add_pair(world, observer, ECS_CHILD_OF, entity);
    }

    /// Callback of the observe functionality
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator which gets passed in from `C`
    ///
    /// # See also
    ///
    /// * C++ API: `entity_observer_delegate::invoke`
    #[doc(alias = "entity_observer_delegate::invoke")]
    pub(crate) unsafe extern "C-unwind" fn run_empty<Func>(iter: *mut sys::ecs_iter_t)
    where
        Func: FnMut(),
    {
        unsafe {
            let ctx: *mut ObserverEntityBindingCtx = (*iter).callback_ctx as *mut _;
            let empty = (*ctx).empty.unwrap();
            let empty = &mut *(empty as *mut Func);
            let iter_count = (*iter).count as usize;

            sys::ecs_table_lock((*iter).world, (*iter).table);

            for _i in 0..iter_count {
                empty();
            }

            sys::ecs_table_unlock((*iter).world, (*iter).table);
        }
    }

    /// Callback of the observe functionality
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator which gets passed in from `C`
    ///
    /// # See also
    ///
    /// * C++ API: `entity_observer_delegate::invoke`
    #[doc(alias = "entity_observer_delegate::invoke")]
    pub(crate) unsafe extern "C-unwind" fn run_empty_entity<Func>(iter: *mut sys::ecs_iter_t)
    where
        Func: FnMut(&mut EntityView),
    {
        unsafe {
            let ctx: *mut ObserverEntityBindingCtx = (*iter).callback_ctx as *mut _;
            let empty = (*ctx).empty_entity.unwrap();
            let empty = &mut *(empty as *mut Func);
            let iter_count = (*iter).count as usize;

            sys::ecs_table_lock((*iter).world, (*iter).table);

            for _i in 0..iter_count {
                let world = WorldRef::from_ptr((*iter).world);
                empty(&mut EntityView::new_from(
                    world,
                    sys::ecs_field_src(iter, 0),
                ));
            }

            sys::ecs_table_unlock((*iter).world, (*iter).table);
        }
    }

    /// Callback of the observe functionality
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator which gets passed in from `C`
    ///
    /// # See also
    ///
    /// * C++ API: `entity_payload_observer_delegate::invoke`
    #[doc(alias = "entity_payload_observer_delegate::invoke")]
    pub(crate) unsafe extern "C-unwind" fn run_payload<C, Func>(iter: *mut sys::ecs_iter_t)
    where
        Func: FnMut(&C),
    {
        unsafe {
            let ctx: *mut ObserverEntityBindingCtx = (*iter).callback_ctx as *mut _;
            let empty = (*ctx).payload.unwrap();
            let empty = &mut *(empty as *mut Func);
            let iter_count = (*iter).count as usize;

            sys::ecs_table_lock((*iter).world, (*iter).table);

            for _i in 0..iter_count {
                let data = (*iter).param as *mut C;
                let data_ref = &mut *data;
                empty(data_ref);
            }

            sys::ecs_table_unlock((*iter).world, (*iter).table);
        }
    }

    /// Callback of the observe functionality
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator which gets passed in from `C`
    ///
    /// # See also
    ///
    /// * C++ API: `entity_payload_observer_delegate::invoke`
    #[doc(alias = "entity_payload_observer_delegate::invoke")]
    pub(crate) unsafe extern "C-unwind" fn run_payload_entity<C, Func>(iter: *mut sys::ecs_iter_t)
    where
        Func: FnMut(&mut EntityView, &C),
    {
        unsafe {
            let ctx: *mut ObserverEntityBindingCtx = (*iter).callback_ctx as *mut _;
            let empty = (*ctx).payload_entity.unwrap();
            let empty = &mut *(empty as *mut Func);
            let iter_count = (*iter).count as usize;

            sys::ecs_table_lock((*iter).world, (*iter).table);

            for _i in 0..iter_count {
                let data = (*iter).param as *mut C;
                let data_ref = &mut *data;
                let world = WorldRef::from_ptr((*iter).world);
                empty(
                    &mut EntityView::new_from(world, sys::ecs_field_src(iter, 0)),
                    data_ref,
                );
            }

            sys::ecs_table_unlock((*iter).world, (*iter).table);
        }
    }

    /// Callback to free the memory of the `empty` callback
    pub(crate) extern "C-unwind" fn on_free_empty(ptr: *mut c_void) {
        let ptr_func: *mut fn() = ptr as *mut fn();
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Callback to free the memory of the `empty_entity` callback
    pub(crate) extern "C-unwind" fn on_free_empty_entity(ptr: *mut c_void) {
        let ptr_func: *mut fn(&mut EntityView) = ptr as *mut fn(&mut EntityView);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Callback to free the memory of the `payload` callback
    pub(crate) extern "C-unwind" fn on_free_payload<C>(ptr: *mut c_void) {
        let ptr_func: *mut fn(&mut C) = ptr as *mut fn(&mut C);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Callback to free the memory of the `payload_entity` callback
    pub(crate) extern "C-unwind" fn on_free_payload_entity<C>(ptr: *mut c_void) {
        let ptr_func: *mut fn(&mut EntityView, &mut C) = ptr as *mut fn(&mut EntityView, &mut C);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Executes the drop for the system binding context, meant to be used as a callback
    pub(crate) extern "C-unwind" fn binding_entity_ctx_drop(ptr: *mut c_void) {
        let ptr_struct: *mut ObserverEntityBindingCtx = ptr as *mut ObserverEntityBindingCtx;
        unsafe {
            ptr::drop_in_place(ptr_struct);
        }
    }
}
