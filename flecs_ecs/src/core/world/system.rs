use crate::addons::system::{System, SystemBuilder};

use super::*;

/// Systems mixin implementation
impl World {
    /// Constructs a `System` from an existing entity.
    ///
    /// This function upcasts the given `entity` to a `System`, assuming the entity represents a system.
    /// The purpose is to facilitate the interaction with entities that are specifically systems within the ECS.
    ///
    /// # Arguments
    /// * `entity` - An `EntityView` that represents a system within the world.
    pub fn system_from<'a>(&'a self, entity: EntityView<'a>) -> System<'a> {
        System::new_from_existing(entity)
    }

    /// Creates a new `SystemBuilder` instance for constructing systems.
    ///
    /// This function initializes a `SystemBuilder` which is used to create systems that match specific components.
    /// It is a generic method that works with any component types that implement the `QueryTuple` trait.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    ///
    /// # See also
    ///
    /// * [`World::system_named()`]
    pub fn system<Components>(&self) -> SystemBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        SystemBuilder::<Components>::new(self)
    }

    /// Creates a new named `SystemBuilder` instance.
    ///
    /// Similar to `system_builder`, but allows naming the system for easier identification and debugging.
    /// The name does not affect the system's behavior.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the system.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    ///
    /// # See also
    ///
    /// * [`World::system()`]
    pub fn system_named<'a, Components>(&'a self, name: &str) -> SystemBuilder<'a, Components>
    where
        Components: QueryTuple,
    {
        SystemBuilder::<Components>::new_named(self, name)
    }

    /// Creates a `SystemBuilder` from a system description.
    ///
    /// This function allows creating a system based on a predefined system description,
    /// facilitating more dynamic or configuration-driven system creation.
    ///
    /// # Arguments
    /// * `desc` - A system description that outlines the parameters for the system builder.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    pub fn system_builder_from_desc<Components>(
        &self,
        desc: sys::ecs_system_desc_t,
    ) -> SystemBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        SystemBuilder::<Components>::new_from_desc(self, desc)
    }
}
