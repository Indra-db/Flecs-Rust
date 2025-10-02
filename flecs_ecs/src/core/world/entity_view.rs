use core::ffi::CStr;

use super::*;

/// `EntityView` mixin implementation
impl World {
    /// Convert enum constant to entity
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to convert.
    ///
    /// # Returns
    ///
    /// `EntityView` wrapping the id of the enum constant.
    #[doc(alias = "world::id")] //enum mixin implementation
    pub fn entity_from_enum<T>(&self, enum_value: T) -> EntityView<'_>
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        EntityView::new_from(self, enum_value.id_variant(self))
    }

    /// Create an entity that's associated with a type and name
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new entity.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new entity.
    pub fn entity_from_named<'a, T: ComponentId>(&'a self, name: &str) -> EntityView<'a> {
        EntityView::new_from(self, T::__register_or_get_id_named::<true>(self, name))
    }

    /// Create an entity that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new entity.
    pub fn entity_from<T: ComponentId>(&self) -> EntityView<'_> {
        EntityView::new_from(self, T::entity_id(self))
    }

    /// Create an entity that's associated with a name.
    /// The name does an extra allocation if it's bigger than 24 bytes. To avoid this, use `entity_named_cstr`.
    /// length of 24 bytes: `"hi this is 24 bytes long"`
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity_named("Foo");
    /// assert_eq!(entity.get_name(), Some("Foo".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::entity()`]
    /// * [`World::entity_named_cstr()`]
    pub fn entity_named(&self, name: &str) -> EntityView<'_> {
        EntityView::new_named(self, name)
    }

    /// Create an entity that's associated with a name within a scope, using a custom separator and root separator.
    /// The name does an extra allocation if it's bigger than 24 bytes. To avoid this, use `entity_named_cstr`.
    /// length of 24 bytes: `"hi this is 24 bytes long"`
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by the sep you use.
    /// For example: "`Foo-Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    /// Note, this does still create the hierarchy as `Foo::Bar`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity_named_scoped("Foo-Bar", "-", "::");
    /// assert_eq!(entity.get_name(), Some("Bar".to_string()));
    /// assert_eq!(entity.path(), Some("::Foo::Bar".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::entity()`]
    /// * [`World::entity_named_cstr()`]
    pub fn entity_named_scoped(&self, name: &str, sep: &str, root_sep: &str) -> EntityView<'_> {
        EntityView::new_named_scoped(self, name, sep, root_sep)
    }

    /// Create an entity that's associated with a name.
    /// The name must be a valid C str. No extra allocation is done.
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity_named("Foo");
    /// assert_eq!(entity.get_name(), Some("Foo".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::entity()`]
    /// * [`World::entity_named()`]
    pub fn entity_named_cstr(&self, name: &CStr) -> EntityView<'_> {
        EntityView::new_named_cstr(self, name)
    }

    /// Create a new entity.
    ///
    /// # See also
    ///
    /// * [`World::entity_named()`]
    /// * [`World::entity_named_cstr()`]
    #[inline(always)]
    pub fn entity(&self) -> EntityView<'_> {
        EntityView::new(self)
    }

    /// Create entity with id 0.
    /// This function is useful when the API must provide an entity that
    /// belongs to a world, but the entity id is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    /// let entity = world.entity_null();
    /// assert_eq!(entity.id(), 0);
    /// ```
    pub fn entity_null(&self) -> EntityView<'_> {
        EntityView::new_null(self)
    }

    /// wraps an [`EntityView`] with the provided id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to use for the new entity.
    pub fn entity_from_id(&self, id: impl Into<Entity>) -> EntityView<'_> {
        EntityView::new_from(self, id.into())
    }

    /// Creates a prefab
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * [`World::prefab_named()`]
    /// * [`World::prefab_type()`]
    /// * [`World::prefab_type_named()`]
    pub fn prefab(&self) -> EntityView<'_> {
        let result = EntityView::new(self);
        result.add(id::<flecs::Prefab>());
        result
    }

    /// Creates a named prefab
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * [`World::prefab()`]
    /// * [`World::prefab_type()`]
    /// * [`World::prefab_type_named()`]
    pub fn prefab_named<'a>(&'a self, name: &str) -> EntityView<'a> {
        let result = EntityView::new_named(self, name);
        unsafe { result.add_id_unchecked(ECS_PREFAB) };
        result
    }

    /// Creates a prefab that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * [`World::prefab()`]
    /// * [`World::prefab_named()`]
    /// * [`World::prefab_type_named()`]
    pub fn prefab_type<T: ComponentId>(&self) -> EntityView<'_> {
        let result = self.entity_from::<T>();
        result.add(ECS_PREFAB);
        result
    }

    /// Creates a named prefab that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new prefab.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * [`World::prefab()`]
    /// * [`World::prefab_named()`]
    /// * [`World::prefab_type()`]
    pub fn prefab_type_named<'a, T: ComponentId>(&'a self, name: &str) -> EntityView<'a> {
        let result = self.entity_from_named::<T>(name);
        result.add(ECS_PREFAB);
        result
    }
}
