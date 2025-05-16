use crate::core::*;
use crate::sys;

impl<'a> IdOperations<'a> for EntityView<'a> {
    type IdType = Entity;

    #[inline(always)]
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
    /// * C API: `ecs_id_t`
    fn new_from_id(world: impl WorldProvider<'a>, id: impl IntoId) -> Self {
        Self {
            world: world.world(),
            id: Entity::from(*id.into_id(world)),
        }
    }

    /// Wraps an id or pair from an expression
    ///
    /// # Arguments
    ///
    /// * `world` - The optional world to the id belongs to
    /// * `expr` - The expression to wrap
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

    impl<'a> PartialEq<&EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &&EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    impl<'a> PartialEq<&mut EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn eq(&self, other: &&mut EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    impl<'a> PartialEq<EntityView<'a>> for &EntityView<'a> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.id == other.id
        }
    }

    impl<'a> PartialEq<EntityView<'a>> for &mut EntityView<'a> {
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

    impl Eq for EntityView<'_> {}
}

mod ord_operations {
    use super::*;

    impl<'a> PartialOrd<EntityView<'a>> for u64 {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<core::cmp::Ordering> {
            Some(self.cmp(&other.id))
        }
    }

    impl PartialOrd<u64> for EntityView<'_> {
        #[inline]
        fn partial_cmp(&self, other: &u64) -> Option<core::cmp::Ordering> {
            Some(self.id.0.cmp(other))
        }
    }

    impl PartialOrd<Entity> for EntityView<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Entity) -> Option<core::cmp::Ordering> {
            Some(self.id.cmp(other))
        }
    }

    impl PartialOrd<Id> for EntityView<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Id) -> Option<core::cmp::Ordering> {
            Some(self.id.0.cmp(&other.0))
        }
    }

    impl<'a> PartialOrd<IdView<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &IdView<'a>) -> Option<core::cmp::Ordering> {
            Some(self.id.0.cmp(&other.id.0))
        }
    }

    impl<'a> PartialOrd<EntityView<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<core::cmp::Ordering> {
            Some(self.id.cmp(&other.id))
        }
    }

    impl<'a, T> PartialOrd<Component<'a, T>> for EntityView<'a>
    where
        T: ComponentId,
    {
        #[inline]
        fn partial_cmp(&self, other: &Component<'a, T>) -> Option<core::cmp::Ordering> {
            Some(self.id.cmp(&other.base.entity.id))
        }
    }

    impl<'a> PartialOrd<UntypedComponent<'a>> for EntityView<'a> {
        #[inline]
        fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<core::cmp::Ordering> {
            Some(self.id.cmp(&other.entity.id))
        }
    }

    impl Ord for EntityView<'_> {
        #[inline]
        fn cmp(&self, other: &EntityView) -> core::cmp::Ordering {
            self.id.cmp(&other.id)
        }
    }
}
