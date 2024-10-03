/// A macro to generate a newtype wrapper for `EntityView` with various utility implementations.
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
/// - Implements `From` for both conversions between `EntityView<'a>` and the new type, and between the new type and `Entity`.
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
