use crate::core::*;

impl<'a> IdOperations<'a> for EntityView<'a> {
    type IdType = Entity;

    fn id(&self) -> Self::IdType {
        self.id
    }

    /// Wraps an id or pair
    ///
    /// # Arguments
    ///
    /// * `world` - The optional world to the id belongs to
    /// * `with` - The id or pair to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `Id::Id`
    #[doc(alias = "Id::Id")]
    /// * C API: `ecs_id_t`
    #[doc(alias = "ecs_id_t")]
    fn new_from(world: impl IntoWorld<'a>, id: impl IntoId) -> Self {
        Self {
            world: world.world(),
            id: Entity::from(*id.into()),
        }
    }
}
