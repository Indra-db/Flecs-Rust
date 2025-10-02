use super::*;

/// Component mixin implementation
impl World {
    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    pub fn component<T: ComponentId>(&self) -> Component<'_, T::UnderlyingType> {
        Component::<T::UnderlyingType>::new(self)
    }

    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the component.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    pub fn component_named<'a, T: ComponentId>(
        &'a self,
        name: &str,
    ) -> Component<'a, T::UnderlyingType> {
        Component::<T::UnderlyingType>::new_named(self, name)
    }

    /// Create new untyped component.
    pub fn component_untyped(&self) -> UntypedComponent<'_> {
        UntypedComponent::new(self)
    }

    /// Create new named untyped component.
    pub fn component_untyped_named(&self, name: &str) -> UntypedComponent<'_> {
        UntypedComponent::new_named(self, name)
    }

    /// Find or register untyped component.
    ///
    /// # Arguments
    ///
    /// * `id` - The component id.
    ///
    /// # Returns
    ///
    /// The found or registered untyped component.
    pub fn component_untyped_from(&self, id: impl IntoEntity) -> UntypedComponent<'_> {
        UntypedComponent::new_from(self, id)
    }

    /// Convert enum constant to entity
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to convert.
    pub fn to_entity<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &self,
        enum_value: T,
    ) -> EntityView<'_> {
        EntityView::new_from(self, enum_value.id_variant(self))
    }
}
