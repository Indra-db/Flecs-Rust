use super::*;

/// Id mixin implementation
impl World {
    /// Get the id of the provided component type.
    /// This returns the id of a component type which has been registered with [`ComponentId`] trait.
    #[inline(always)]
    pub fn component_id<T: ComponentId>(&self) -> Entity {
        Entity(T::entity_id(self))
    }

    /// Get the id of the provided component type.
    /// This returns the id of a component type which has potentially not been registered with `ComponentId` trait.
    /// This holds ids for external components (not marked with the trait), such as in cases of meta fields.
    /// When meta is enabled, this will also hold ids for components that are registered with the `ComponentId` trait.
    pub(crate) fn component_id_map<T: 'static>(&self) -> u64 {
        *self.components_map()
            .get(&core::any::TypeId::of::<T>())
            .unwrap_or_else(|| panic!("Component with name: {} is not registered, pre-register components with `world.component::<T>() or world.component_ext::<T>(id)`", core::any::type_name::<T>()))
    }

    /// Get the id of the provided pair of components.
    pub fn id_from<T: IntoId>(&self, id: T) -> Id {
        Id(*id.into_id(self))
    }

    /// get `IdView` from an id or from a relationship pair
    ///
    /// # Arguments
    ///
    /// * `id` - The id to convert to an `IdView`.
    ///
    /// # Returns
    ///
    /// The `IdView` from the provided id.
    pub fn id_view_from<Id: IntoId>(&self, id: Id) -> IdView<'_> {
        let id = *id.into_id(self);
        if Id::IS_PAIR {
            ecs_assert!(
                {
                    let first = ecs_first(id, self);
                    let second = ecs_second(id, self);
                    !ecs_is_pair(first) && !ecs_is_pair(second)
                },
                FlecsErrorCode::InvalidParameter,
                "cannot create nested pairs"
            );
        }

        IdView::new_from_id(self, id)
    }
}
