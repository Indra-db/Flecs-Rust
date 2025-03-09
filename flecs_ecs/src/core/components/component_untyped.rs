use core::{
    fmt::{Debug, Display},
    ops::Deref,
};

use crate::core::*;

/// Untyped component class.
#[derive(Clone, Copy)]
pub struct UntypedComponent<'a> {
    pub entity: EntityView<'a>,
}

impl Display for UntypedComponent<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.entity)
    }
}

impl Debug for UntypedComponent<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.entity)
    }
}

impl<'a> Deref for UntypedComponent<'a> {
    type Target = EntityView<'a>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a> UntypedComponent<'a> {
    /// Create a new untyped component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `id`: the id of the component to reference.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::untyped_component`
    #[doc(alias = "untyped_component::untyped_component")]
    pub(crate) fn new(world: impl WorldProvider<'a>) -> Self {
        UntypedComponent {
            entity: EntityView::new(world),
        }
    }

    /// Create a new untyped component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `id`: the id of the component to reference.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::untyped_component`
    #[doc(alias = "untyped_component::untyped_component")]
    pub(crate) fn new_named(world: impl WorldProvider<'a>, name: &str) -> Self {
        UntypedComponent {
            entity: EntityView::new_named(world, name),
        }
    }

    /// Wrap an existing component into untyped component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `id`: the id of the component to reference.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::untyped_component`
    #[doc(alias = "untyped_component::untyped_component")]
    #[inline(never)]
    pub(crate) fn new_from(world: impl WorldProvider<'a>, id: impl Into<Entity>) -> Self {
        UntypedComponent {
            entity: EntityView::new_from(world, id),
        }
    }

    /// Get the id of the component.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::entity`
    #[doc(alias = "untyped_component::entity")]
    pub fn as_entity(&self) -> EntityView<'a> {
        self.entity
    }
}

#[cfg(feature = "flecs_meta")]
impl UntypedComponent<'_> {}

#[cfg(feature = "flecs_metrics")]
impl UntypedComponent<'_> {}

mod eq_operations {
    use super::*;

    impl<'a> PartialEq<UntypedComponent<'a>> for u64 {
        #[inline]
        fn eq(&self, other: &UntypedComponent<'a>) -> bool {
            *self == other.entity.id
        }
    }

    impl PartialEq<u64> for UntypedComponent<'_> {
        #[inline]
        fn eq(&self, other: &u64) -> bool {
            self.entity.id == *other
        }
    }

    impl PartialEq<Entity> for UntypedComponent<'_> {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.entity.id == *other
        }
    }

    impl PartialEq<Id> for UntypedComponent<'_> {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.entity.id == *other
        }
    }

    impl<'a> PartialEq<EntityView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.entity == *other
        }
    }

    impl<'a> PartialEq<IdView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &IdView<'a>) -> bool {
            self.entity == other.id
        }
    }

    impl<'a, T> PartialEq<Component<'a, T>> for UntypedComponent<'a>
    where
        T: ComponentId,
    {
        #[inline]
        fn eq(&self, other: &Component<'a, T>) -> bool {
            self.entity == other.base.entity
        }
    }

    impl PartialEq for UntypedComponent<'_> {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.entity == other.entity
        }
    }

    impl Eq for UntypedComponent<'_> {}
}

mod ord_operations {
    use super::*;

    impl<'a> PartialOrd<UntypedComponent<'a>> for u64 {
        #[inline]
        fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<core::cmp::Ordering> {
            self.partial_cmp(&other.entity.id)
        }
    }

    impl PartialOrd<u64> for UntypedComponent<'_> {
        #[inline]
        fn partial_cmp(&self, other: &u64) -> Option<core::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl PartialOrd<Entity> for UntypedComponent<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Entity) -> Option<core::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl PartialOrd<Id> for UntypedComponent<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Id) -> Option<core::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl<'a> PartialOrd<EntityView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<core::cmp::Ordering> {
            self.entity.partial_cmp(other)
        }
    }

    impl<'a> PartialOrd<IdView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &IdView<'a>) -> Option<core::cmp::Ordering> {
            self.entity.partial_cmp(&other.id)
        }
    }

    impl PartialOrd for UntypedComponent<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
            Some(self.entity.cmp(&other.entity))
        }
    }

    impl Ord for UntypedComponent<'_> {
        #[inline]
        fn cmp(&self, other: &Self) -> core::cmp::Ordering {
            self.entity.cmp(&other.entity)
        }
    }
}
