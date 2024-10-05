/// A macro to generate a newtype wrapper for [`EntityView`][crate::core::EntityView] with various utility implementations.
///
/// This macro creates a new struct with the specified name that wraps around `EntityView<'a>`.
/// It also provides implementations for common traits like `Deref`, `DerefMut`, `From`, `Debug` and `Display`
/// to make working with the new type seamless.
///
/// # Parameters
/// - `$name`: The name of the new struct that wraps `EntityView<'a>`.
///
/// # Generated Code
///
/// - A struct named `$name` is created, wrapping an `EntityView<'a>`.
/// - A constructor `new` that takes an `EntityView<'a>` is generated for the newtype.
/// - Implements `Deref` and `DerefMut` traits to allow easy access to the underlying `EntityView<'a>`.
/// - Implements `From` for both conversions between `EntityView<'a>` and the new type, and between the new type and [`Entity`][crate::core::Entity].
/// - Provides `Debug` and `Display` implementations for better formatting support.
#[macro_export]
macro_rules! newtype_of_entity_view {
    ($name:ident) => {
        #[derive(Clone, Copy)]
        pub struct $name<'a>(pub EntityView<'a>);

        impl<'a> $name<'a> {
            pub fn new(entity: EntityView<'a>) -> Self {
                Self(entity)
            }
        }

        impl<'a> core::ops::Deref for $name<'a> {
            type Target = EntityView<'a>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<'a> core::ops::DerefMut for $name<'a> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<'a> From<EntityView<'a>> for $name<'a> {
            fn from(entity: EntityView<'a>) -> Self {
                Self::new(entity)
            }
        }

        impl<'a> From<$name<'a>> for EntityView<'a> {
            fn from(entity: $name<'a>) -> Self {
                entity.0
            }
        }

        impl From<$name<'_>> for Entity {
            #[inline]
            fn from(name: $name) -> Self {
                name.0.id()
            }
        }

        impl<'a> core::fmt::Debug for $name<'a> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl<'a> core::fmt::Display for $name<'a> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

/// A macro to generate a newtype wrapper for [`Entity`][crate::core::Entity] with various utility implementations.
///
/// This macro creates a new struct with the specified name that wraps around [`Entity`][crate::core::Entity].
/// It also provides implementations for common traits like `Deref`, `DerefMut`, `From`, `Debug` and `Display`
/// to make working with the new type seamless.
///
/// # Parameters
///
/// - `$name`: The name of the new struct that wraps [`Entity`][crate::core::Entity].
///
/// # Generated Code
///
/// - A struct named `$name` is created, wrapping an [`Entity`][crate::core::Entity].
/// - A constructor `new` that takes an [`Entity`][crate::core::Entity] is generated for the newtype.
/// - Implements `Deref` and `DerefMut` traits to allow easy access to the underlying [`Entity`][crate::core::Entity].
/// - Implements `From` for both conversions between [`Entity`][crate::core::Entity] and the new type.
/// - Provides `Debug` and `Display` implementations for better formatting support.
#[macro_export]
macro_rules! newtype_of_entity {
    ($name:ident) => {
        #[derive(Clone, Copy, Component)]
        pub struct $name(pub Entity);

        impl $name {
            pub fn new(entity: impl Into<Entity>) -> Self {
                Self(entity.into())
            }
        }

        impl core::ops::Deref for $name {
            type Target = Entity;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<$name> for Entity {
            fn from(entity: $name) -> Self {
                entity.0
            }
        }

        impl From<&$name> for Entity {
            fn from(entity: &$name) -> Self {
                entity.0
            }
        }

        impl From<&mut $name> for Entity {
            fn from(entity: &mut $name) -> Self {
                entity.0
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

/// A macro to generate a newtype wrapper for [`Entity`][crate::core::Entity] and [`EntityView`][crate::core::EntityView] with various utility implementations.
///
/// This macro creates a new struct with the specified name that wraps around [`Entity`][crate::core::Entity] and [`EntityView`][crate::core::EntityView].
/// See the documentation of [`newtype_of_entity!`] and [`newtype_of_entity_view!`] for more details.
/// It also implements methods to convert between the newtype entity and view. `entity.view(world)` and `view.id()`
///
/// # Parameters
///
/// - `$name_entity`: The name of the new struct that wraps [`Entity`][crate::core::Entity].
///
/// # Generated Code
///
/// - A struct named `$name_entity` is created, wrapping an [`Entity`][crate::core::Entity]. See [`newtype_of_entity!`] for more details.
/// - A struct named `$name_view` is created, wrapping an [`EntityView`][crate::core::EntityView] See [`newtype_of_entity_view!`] for more details.
/// - Implements methods to convert between the newtype entity and view. `entity.view(world)` and `view.id()`
/// - Implements `From` for conversion between `name_view` and `name_entity`
///
/// # Example
///
/// ```rust
/// use flecs_ecs::newtype_of_entity_and_view;
/// use flecs_ecs::prelude::*;
///
/// newtype_of_entity_and_view!(MyEntity, MyEntityView);
///
/// let world = World::new();
///
/// let entity = world.entity();
///
/// let my_entity_view = MyEntityView::new(entity);
/// let my_entity: MyEntity = my_entity_view.id();
///
/// let my_entity_view: MyEntityView = my_entity.view(&world);
/// let my_entity: MyEntity = my_entity_view.into();
/// let my_entity: MyEntity = MyEntity::new(my_entity_view);
/// ```
#[macro_export]
macro_rules! newtype_of_entity_and_view {
    ($name_entity: ident, $name_view:ident) => {
        flecs_ecs::newtype_of_entity!($name_entity);
        flecs_ecs::newtype_of_entity_view!($name_view);

        impl From<$name_view<'_>> for $name_entity {
            fn from(entity: $name_view) -> Self {
                Self::new(entity.0.id())
            }
        }

        impl $name_entity {
            pub fn view<'a>(&self, world: impl WorldProvider<'a>) -> $name_view<'a> {
                $name_view::new(EntityView::new_from(world, self.0))
            }
        }

        impl $name_view<'_> {
            pub fn id(&self) -> $name_entity {
                $name_entity::new(self.0.id())
            }
        }
    };
}
