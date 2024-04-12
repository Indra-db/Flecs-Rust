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

mod eq_operations {
    use super::*;

    impl PartialEq<EntityView<'_>> for u64 {
        #[inline]
        fn eq(&self, other: &EntityView<'_>) -> bool {
            *self == *other.id
        }
    }

    impl PartialEq<u64> for EntityView<'_> {
        #[inline]
        fn eq(&self, other: &u64) -> bool {
            self.id == *other
        }
    }

    impl PartialEq<Entity> for EntityView<'_> {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.id == *other
        }
    }

    impl PartialEq<Id> for EntityView<'_> {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.id == *other
        }
    }

    impl<'a> PartialEq<EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    impl<'a> PartialEq<IdView<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &IdView<'a>) -> bool {
            self.id == other.id
        }
    }

    impl<'a, T> PartialEq<Component<'a, T>> for EntityView<'a>
    where
        T: ComponentId,
    {
        #[inline]
        fn eq(&self, other: &Component<'a, T>) -> bool {
            self.id == other.base.entity.id
        }
    }

    impl<'a> PartialEq<UntypedComponent<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &UntypedComponent<'a>) -> bool {
            self.id == other.entity.id
        }
    }

    impl<'a> Eq for EntityView<'a> {}
}

mod ord_operations {
    use super::*;

    impl<'a> PartialOrd<EntityView<'a>> for u64 {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<std::cmp::Ordering> {
            Some(self.cmp(&other.id))
        }
    }

    impl<'a> PartialOrd<u64> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
            Some(self.id.0.cmp(other))
        }
    }

    impl<'a> PartialOrd<Entity> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &Entity) -> Option<std::cmp::Ordering> {
            Some(self.id.cmp(other))
        }
    }

    impl<'a> PartialOrd<Id> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &Id) -> Option<std::cmp::Ordering> {
            Some(self.id.0.cmp(&other.0))
        }
    }

    impl<'a> PartialOrd<IdView<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &IdView<'a>) -> Option<std::cmp::Ordering> {
            Some(self.id.0.cmp(&other.id.0))
        }
    }

    impl<'a> PartialOrd<EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<std::cmp::Ordering> {
            Some(self.id.cmp(&other.id))
        }
    }

    impl<'a, T> PartialOrd<Component<'a, T>> for EntityView<'a>
    where
        T: ComponentId,
    {
        #[inline]
        fn partial_cmp(&self, other: &Component<'a, T>) -> Option<std::cmp::Ordering> {
            Some(self.id.cmp(&other.base.entity.id))
        }
    }

    impl<'a> PartialOrd<UntypedComponent<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<std::cmp::Ordering> {
            Some(self.id.cmp(&other.entity.id))
        }
    }

    impl<'a> Ord for EntityView<'a> {
        #[inline]
        fn cmp(&self, other: &EntityView) -> std::cmp::Ordering {
            self.id.cmp(&other.id)
        }
    }
}
