/// A macro to generate a newtype wrapper for [`EntityView`][crate::core::EntityView] with various utility implementations.
///
/// This macro takes the full struct definition as input, so you can define the visibility of the struct and its fields
/// however you like. It also provides implementations for common traits like `Deref`, `DerefMut`, `From`, `Debug`
/// and `Display` to make working with the new type seamless.
///
/// # Parameters
///
/// - `$struct_vis`: The visibility of the struct (e.g., `pub`).
/// - `$struct_def`: The name of the struct being defined.
/// - `$field_vis`: The visibility of the inner field (e.g., `pub`).
///
/// # Generated Code
///
/// - Implements `Deref` and `DerefMut` traits to allow easy access to the underlying `EntityView`.
/// - Implements `From` for conversions between the newtype and `EntityView`.
/// - Provides `Debug` and `Display` implementations for better formatting support.
///
/// # Example Usage & Compile test
///
/// ```rust
/// use flecs_ecs::newtype_of_entity_view;
/// use flecs_ecs::prelude::*;
/// newtype_of_entity_view!(pub struct MyEntityView1(EntityView)); // public w/ private field
/// newtype_of_entity_view!(pub struct MyEntityView2(pub EntityView)); // public w/ public field
/// newtype_of_entity_view!(struct MyEntityView3(pub EntityView)); // private w/ public field
/// newtype_of_entity_view!(struct MyEntityView4(EntityView)); // private w/ private field
/// newtype_of_entity_view!(pub(crate) struct MyEntityView5(pub EntityView)); // public(crate) w/ public field
/// ```
#[macro_export]
macro_rules! newtype_of_entity_view {
    (
        $struct_vis:vis struct $name:ident($field_vis:vis EntityView)
    ) => {
        #[derive(Clone, Copy)]
        $struct_vis struct $name<'a>($field_vis EntityView<'a>);

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
/// This macro takes the full struct definition as input, so you can define the visibility of the struct and its fields
/// however you like. It also provides implementations for common traits like `Deref`, `DerefMut`, `From`, `Debug`
/// and `Display` to make working with the new type seamless.
///
/// # Parameters
///
/// - `$struct_vis`: The visibility of the struct (e.g., `pub`).
/// - `$name`: The name of the struct being defined.
/// - `$field_vis`: The visibility of the inner field (e.g., `pub`).
///
/// # Generated Code
///
/// - Implements `Deref` and `DerefMut` traits to allow easy access to the underlying `Entity`.
/// - Implements `From` for conversions between the newtype and `Entity`.
/// - Provides `Debug` and `Display` implementations for better formatting support.
///
/// # Example Usage & Compile test
///
/// ```rust
/// use flecs_ecs::newtype_of_entity;
/// use flecs_ecs::prelude::*;
/// newtype_of_entity!(pub struct MyEntity1(Entity)); // public w/ private field
/// newtype_of_entity!(pub struct MyEntity2(pub Entity)); // public w/ public field
/// newtype_of_entity!(struct MyEntity3(pub Entity)); // private w/ public field
/// newtype_of_entity!(struct MyEntity4(Entity)); // private w/ private field
/// newtype_of_entity!(pub(crate) struct MyEntity5(pub Entity)); // public(crate) w/ public field
/// ```
#[macro_export]
macro_rules! newtype_of_entity {
    // Match for structs with visibility and a field
    (
        $struct_vis:vis struct $name:ident($field_vis:vis Entity)
    ) => {
        #[derive(Clone, Copy, Component)]
        $struct_vis struct $name($field_vis Entity);

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

/// A macro to generate newtype wrappers for both [`Entity`][crate::core::Entity] and [`EntityView`][crate::core::EntityView] with various utility implementations.
/// It uses the macros [`newtype_of_entity`] and [`newtype_of_entity_view`] internally to generate the newtypes.
///
/// This macro takes the full struct definitions for both the entity and the view newtypes, allowing you to define the visibility of each struct and its fields.
/// It provides implementations for common traits such as `Deref`, `DerefMut`, `From`, `Debug`, and `Display`, and also implements conversion methods between the entity and view types.
///
/// # Parameters
///
/// - `$entity_struct_vis`: The visibility of the entity struct (e.g., `pub`).
/// - `$entity_name`: The name of the entity struct being defined.
/// - `$entity_field_vis`: The visibility of the inner field of the entity (e.g., `pub`).
/// - `$view_struct_vis`: The visibility of the view struct (e.g., `pub`).
/// - `$view_name`: The name of the view struct being defined.
/// - `$view_field_vis`: The visibility of the inner field of the view (e.g., `pub`).
///
/// # Generated Code
///
/// - A struct named `$entity_name` is created, wrapping an [`Entity`][crate::core::Entity].
/// - A struct named `$view_name` is created, wrapping an [`EntityView`][crate::core::EntityView].
/// - Implements methods to convert between the entity and view newtypes (`entity.view(world)` and `view.id()`).
/// - Implements `From` for conversions between `$view_name` and `$entity_name`.
///
/// # Example Usage
///
/// ```rust
/// use flecs_ecs::newtype_of_entity_and_view;
/// use flecs_ecs::prelude::*;
///
/// newtype_of_entity_and_view!(
///     pub struct MyEntity(pub Entity),
///     pub struct MyEntityView(pub EntityView)
/// );
///
/// let world = World::new();
/// let entity = world.entity();
///
/// let my_entity = MyEntity(entity.id());
/// let my_entity_view = my_entity.view(&world);
/// let my_entity_from_view: MyEntity = my_entity_view.into();
/// ```
#[macro_export]
macro_rules! newtype_of_entity_and_view {
    (
        $entity_struct_vis:vis struct $entity_name:ident($entity_field_vis:vis Entity),
        $view_struct_vis:vis struct $view_name:ident($view_field_vis:vis EntityView)
    ) => {
        // Define the entity newtype
        flecs_ecs::newtype_of_entity!($entity_struct_vis struct $entity_name($entity_field_vis Entity));

        // Define the entity view newtype
        flecs_ecs::newtype_of_entity_view!($view_struct_vis struct $view_name($view_field_vis EntityView));

        // Implement conversion from view to entity
        impl From<$view_name<'_>> for $entity_name {
            fn from(view: $view_name) -> Self {
                Self(view.0.id())
            }
        }

        // Implement method to convert entity to view
        impl $entity_name {
            pub fn view<'a>(&self, world: impl WorldProvider<'a>) -> $view_name<'a> {
                $view_name(EntityView::new_from(world, self.0))
            }
        }

        // Implement method to convert view to entity
        impl<'a> $view_name<'a> {
            pub fn id(&self) -> $entity_name {
                $entity_name(self.0.id())
            }
        }
    };
}
