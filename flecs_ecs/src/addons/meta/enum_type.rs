use crate::prelude::*;

/// Enum type reflection wrapper for querying enum constants at runtime.
///
/// Provides methods to inspect enum types: lookup constants by value, check validity, get indices.
///
/// # Example
/// ```ignore
/// let world = World::new();
/// #[derive(Component, PartialOrd, PartialEq, Clone, Copy, Debug)]
/// enum Color { Red = 1, Green = 2, Blue = 3 }
///
/// let color_enum = world.enum_type::<Color>();
/// assert_eq!(color_enum.first(), 0);
/// assert_eq!(color_enum.last(), 2);
/// assert_eq!(color_enum.index_by_value(Color::Red as i32), 0);
/// assert!(color_enum.is_valid(Color::Green as i32));
/// ```
pub struct EnumType<'a, T: ComponentId> {
    world: WorldRef<'a>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: ComponentId> EnumType<'a, T> {
    /// Create enum type reflection wrapper for a component type.
    pub(crate) fn new(world: WorldRef<'a>) -> Self {
        EnumType {
            world,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the first constant index (0 if constants exist, -1 otherwise).
    pub fn first(&self) -> i32 {
        if self.last() >= 0 {
            0
        } else {
            -1
        }
    }

    /// Get the last constant index (-1 if no constants).
    pub fn last(&self) -> i32 {
        // Count enum values via iteration (relies on enum having PartialOrd+PartialEq)
        let count = T::UnderlyingEnumType::iter().count();
        if count > 0 {
            (count - 1) as i32
        } else {
            -1
        }
    }

    /// Get the constant index for a given underlying value.
    /// Returns -1 if the value is not a valid constant.
    pub fn index_by_value(&self, value: i64) -> i32 {
        for (idx, constant) in T::UnderlyingEnumType::iter().enumerate() {
            let const_val = constant.enum_index() as i64;
            if const_val == value {
                return idx as i32;
            }
        }
        -1
    }

    /// Check if a given value is a valid enum constant.
    pub fn is_valid(&self, value: i64) -> bool {
        self.index_by_value(value) >= 0
    }

    /// Get the entity ID for a given enum constant value.
    /// Returns 0 if the value is not a valid constant.
    pub fn entity_id(&self, value: i64) -> u64 {
        let index = self.index_by_value(value);
        if index < 0 {
            return 0;
        }

        // Get the entity ID for the enum constant
        T::UnderlyingEnumType::iter()
            .nth(index as usize)
            .and_then(|constant| {
                let entity_id = unsafe {
                    T::UnderlyingEnumType::id_variant_of_index_unchecked(
                        constant.enum_index(),
                        self.world,
                    )
                };
                if entity_id != 0 {
                    Some(entity_id)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    /// Get the entity for the enum type itself.
    pub fn entity_type(&self) -> u64 {
        *self.world.component_id::<T>()
    }
}

impl<'a> WorldRef<'a> {
    /// Get enum type reflection wrapper for a component type.
    ///
    /// # Example
    /// ```ignore
    /// #[derive(Component, PartialOrd, PartialEq)]
    /// enum Color { Red = 1, Green = 2, Blue = 3 }
    ///
    /// let world = World::new();
    /// let color_enum = world.enum_type::<Color>();
    /// println!("First constant: {}", color_enum.first());
    /// ```
    pub fn enum_type<T: ComponentId>(&self) -> EnumType<T> {
        EnumType::new(*self)
    }
}
