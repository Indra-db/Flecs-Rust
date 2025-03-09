use crate::core::*;
use crate::sys;

impl<'a> IdOperations<'a> for EntityView<'a> {
    type IdType = Entity;

    /// Returns the entity identifier associated with this view.
    ///
    /// This method allows you to access the unique identifier of the entity
    /// that this view represents.
    ///
    /// # Returns
    ///
    /// The entity identifier.

    #[inline(always)]
    fn id(&self) -> Self::IdType {
        self.id
    }

    /// Creates a new EntityView from an identifier.
    ///
    /// This function wraps an entity identifier or pair and associates it with a world,
    /// creating a view that allows for operations on that entity.
    ///
    /// # Arguments
    ///
    /// * `world` - The world that the entity belongs to
    /// * `id` - The entity identifier or pair to wrap
    ///
    /// # Returns
    ///
    /// A new EntityView instance.
    fn new_from_id(world: impl WorldProvider<'a>, id: impl IntoId) -> Self {
        Self {
            world: world.world(),
            id: Entity::from(*id.into()),
        }
    }

    /// Creates a new EntityView from a string expression.
    ///
    /// This function parses the provided string expression and resolves it to
    /// an entity identifier within the specified world.
    ///
    /// # Arguments
    ///
    /// * `world` - The world that the entity belongs to
    /// * `expr` - The string expression to parse into an entity identifier
    ///
    /// # Returns
    ///
    /// A new EntityView instance.
    ///
    /// # Example
    fn new_from_str(world: impl WorldProvider<'a>, expr: &str) -> Self {
        let expr = compact_str::format_compact!("{}\0", expr);
        let id = unsafe { sys::ecs_id_from_str(world.world_ptr(), expr.as_ptr() as *const _) };
        Self {
            world: world.world(),
            id: Entity(id),
        }
    }
}

mod eq_operations {
    use super::*;
    /// Implements equality comparison between an unsigned 64-bit integer and an EntityView.
    ///
    /// This allows direct comparison of raw entity IDs with entity views.
    impl PartialEq<EntityView<'_>> for u64 {
        #[inline]
        fn eq(&self, other: &EntityView<'_>) -> bool {
            *self == *other.id
        }
    }


    /// Implements equality comparison between an EntityView and an unsigned 64-bit integer.
    ///
    /// This allows direct comparison of entity views with raw entity IDs.
    impl PartialEq<u64> for EntityView<'_> {
        #[inline]
        fn eq(&self, other: &u64) -> bool {
            self.id == *other
        }
    }

    /// Implements equality comparison between an EntityView and an Entity.
    ///
    /// This allows direct comparison of entity views with Entity objects.
    impl PartialEq<Entity> for EntityView<'_> {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.id == *other
        }
    }

    /// Implements equality comparison between an EntityView and an Id.
    ///
    /// This allows direct comparison of entity views with Id objects.
    impl PartialEq<Id> for EntityView<'_> {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.id == *other
        }
    }


    /// Implements equality comparison between an EntityView and an Id.
    ///
    /// This allows direct comparison of entity views with Id objects.
    impl<'a> PartialEq<EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    /// Implements equality comparison between an EntityView and a reference to an EntityView.
    ///
    /// This allows comparing an entity view to a reference of another entity view.
    impl<'a> PartialEq<&EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &&EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    /// Implements equality comparison between an EntityView and a mutable reference to an EntityView.
    ///
    /// This allows comparing an entity view to a mutable reference of another entity view.
    impl<'a> PartialEq<&mut EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &&mut EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    /// Implements equality comparison between a reference to an EntityView and an EntityView.
    ///
    /// This allows comparing a reference of an entity view to another entity view.
    impl<'a> PartialEq<EntityView<'a>> for &EntityView<'a> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    /// Implements equality comparison between a mutable reference to an EntityView and an EntityView.
    ///
    /// This allows comparing a mutable reference of an entity view to another entity view.
    impl<'a> PartialEq<EntityView<'a>> for &mut EntityView<'a> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    /// Implements equality comparison between an EntityView and an IdView.
    ///
    /// This allows direct comparison of entity views with id views.
    impl<'a> PartialEq<IdView<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &IdView<'a>) -> bool {
            self.id == other.id
        }
    }

    /// Implements equality comparison between an EntityView and a Component.
    ///
    /// This allows direct comparison of entity views with components to check if
    /// the component belongs to the entity.
    impl<'a, T> PartialEq<Component<'a, T>> for EntityView<'a>
    where
        T: ComponentId,
    {
        #[inline]
        fn eq(&self, other: &Component<'a, T>) -> bool {
            self.id == other.base.entity.id
        }
    }

    /// Implements equality comparison between an EntityView and an UntypedComponent.
    ///
    /// This allows direct comparison of entity views with untyped components to check if
    /// the component belongs to the entity.
    
    impl<'a> PartialEq<UntypedComponent<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &UntypedComponent<'a>) -> bool {
            self.id == other.entity.id
        }
    }

    /// Implements the Eq trait for EntityView.
    ///
    /// This confirms that EntityView implements full equivalence relation.
    impl Eq for EntityView<'_> {}
}

mod ord_operations {
    use super::*;

     /// Implements ordering comparison between an unsigned 64-bit integer and an EntityView.
    ///
    /// This allows sorting collections containing both raw entity IDs and entity views.
    impl<'a> PartialOrd<EntityView<'a>> for u64 {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<core::cmp::Ordering> {
            Some(self.cmp(&other.id))
        }
    }
 
    /// Implements ordering comparison between an EntityView and an unsigned 64-bit integer.
    ///
    /// This allows sorting collections containing both entity views and raw entity IDs.
    impl PartialOrd<u64> for EntityView<'_> {
        #[inline]
        fn partial_cmp(&self, other: &u64) -> Option<core::cmp::Ordering> {
            Some(self.id.0.cmp(other))
        }
    }

    /// Implements ordering comparison between an EntityView and an Entity.
    ///
    /// This allows sorting collections containing both entity views and Entity objects.
    impl PartialOrd<Entity> for EntityView<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Entity) -> Option<core::cmp::Ordering> {
            Some(self.id.cmp(other))
        }
    }

    /// Implements ordering comparison between an EntityView and an Id.
    ///
    /// This allows sorting collections containing both entity views and Id objects.
    impl PartialOrd<Id> for EntityView<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Id) -> Option<core::cmp::Ordering> {
            Some(self.id.0.cmp(&other.0))
        }
    }

    /// Implements ordering comparison between an EntityView and an IdView.
    ///
    /// This allows sorting collections containing both entity views and id views.
    impl<'a> PartialOrd<IdView<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &IdView<'a>) -> Option<core::cmp::Ordering> {
            Some(self.id.0.cmp(&other.id.0))
        }
    }

    /// Implements ordering comparison between two EntityView instances.
    ///
    /// This allows sorting collections of entity views based on their entity identifiers.
    impl<'a> PartialOrd<EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<core::cmp::Ordering> {
            Some(self.id.cmp(&other.id))
        }
    }

    /// Implements ordering comparison between an EntityView and a Component.
    ///
    /// This allows sorting collections containing both entity views and components.
    impl<'a, T> PartialOrd<Component<'a, T>> for EntityView<'a>
    where
        T: ComponentId,
    {
        #[inline]
        fn partial_cmp(&self, other: &Component<'a, T>) -> Option<core::cmp::Ordering> {
            Some(self.id.cmp(&other.base.entity.id))
        }
    }

    /// Implements ordering comparison between an EntityView and an UntypedComponent.
    ///
    /// This allows sorting collections containing both entity views and untyped components.
    impl<'a> PartialOrd<UntypedComponent<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<core::cmp::Ordering> {
            Some(self.id.cmp(&other.entity.id))
        }
    }

    /// Implements the Ord trait for EntityView.
    ///
    /// This provides a total ordering for EntityView based on the entity identifier,
    /// allowing EntityView to be used as a key in sorted collections such as BTreeMap.
    impl Ord for EntityView<'_> {
        #[inline]
        fn cmp(&self, other: &EntityView) -> core::cmp::Ordering {
            self.id.cmp(&other.id)
        }
    }
}
