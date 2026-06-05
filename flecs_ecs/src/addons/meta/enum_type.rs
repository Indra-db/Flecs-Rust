use crate::prelude::*;

/// Enum type reflection wrapper for querying enum constants at runtime.
///
/// Provides methods to inspect enum types: lookup constants by value, check validity, get indices.
///
/// # Example
/// ```ignore
/// #[derive(Component, PartialOrd, PartialEq, Clone, Copy, Debug)]
/// enum Color { Red = 1, Green = 2, Blue = 3 }
///
/// let world = World::new();
/// let color_enum = world.enum_type::<Color>();
/// assert_eq!(color_enum.first(), 0);
/// assert_eq!(color_enum.last(), 2);
/// assert_eq!(color_enum.index_by_value(1), 0);  // Red's discriminant is 1
/// assert_eq!(color_enum.index_by_value(2), 1);  // Green's discriminant is 2
/// assert!(color_enum.is_valid(1));
/// assert!(!color_enum.is_valid(99));
/// ```
pub struct EnumType<'a, T: ComponentId> {
    world: WorldRef<'a>,
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: ComponentId> EnumType<'a, T> {
    /// Create enum type reflection wrapper for a component type.
    pub(crate) fn new(world: WorldRef<'a>) -> Self {
        EnumType {
            world,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Get the first constant index (0 if constants exist, -1 otherwise).
    pub fn first(&self) -> i32 {
        if T::UnderlyingEnumType::iter().next().is_some() {
            0
        } else {
            -1
        }
    }

    /// Get the last constant index (-1 if no constants).
    pub fn last(&self) -> i32 {
        // Count enum values via iteration (relies on enum having PartialOrd+PartialEq)
        let count = T::UnderlyingEnumType::iter().count();
        if count > 0 { (count - 1) as i32 } else { -1 }
    }

    /// Get the constant index for a given underlying value.
    /// Returns -1 if the value is not a valid constant.
    pub fn index_by_value(&self, value: i64) -> i32 {
        let size = core::mem::size_of::<T::UnderlyingTypeOfEnum>();
        for (idx, constant) in T::UnderlyingEnumType::iter().enumerate() {
            // Extract the discriminant value from the enum variant
            let discriminant = match size {
                1 => unsafe { *(&constant as *const _ as *const i8) as i64 },
                2 => unsafe { *(&constant as *const _ as *const i16) as i64 },
                4 => unsafe { *(&constant as *const _ as *const i32) as i64 },
                8 => unsafe { *(&constant as *const _ as *const i64) },
                _ => continue,
            };
            if discriminant == value {
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
        let size = core::mem::size_of::<T::UnderlyingTypeOfEnum>();
        for constant in T::UnderlyingEnumType::iter() {
            // Extract the discriminant value from the enum variant
            let discriminant = match size {
                1 => unsafe { *(&constant as *const _ as *const i8) as i64 },
                2 => unsafe { *(&constant as *const _ as *const i16) as i64 },
                4 => unsafe { *(&constant as *const _ as *const i32) as i64 },
                8 => unsafe { *(&constant as *const _ as *const i64) },
                _ => continue,
            };
            if discriminant == value {
                // Found matching constant, get its entity ID
                let entity_id = unsafe {
                    T::UnderlyingEnumType::id_variant_of_index_unchecked(
                        constant.enum_index(),
                        self.world,
                    )
                };
                return entity_id;
            }
        }
        0
    }

    /// Get the entity for the enum type itself.
    pub fn entity_type(&self) -> u64 {
        *self.world.component_id::<T>()
    }
}

impl WorldRef<'_> {
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
    pub fn enum_type<T: ComponentId>(&self) -> EnumType<'_, T> {
        EnumType::new(*self)
    }
}

impl crate::prelude::World {
    /// Get enum type reflection wrapper for a component type.
    ///
    /// Provides runtime reflection for enum constants, including:
    /// - `.first()` / `.last()` for constant index boundaries
    /// - `.index_by_value(value)` to lookup constant by discriminant
    /// - `.is_valid(value)` to check if a value is a valid constant
    /// - `.entity_id(value)` to get the ECS entity for a constant
    ///
    /// # Example
    /// ```ignore
    /// #[derive(Component, PartialOrd, PartialEq)]
    /// enum Color { Red = 1, Green = 2, Blue = 3 }
    ///
    /// let world = World::new();
    /// let color_enum = world.enum_type::<Color>();
    /// assert_eq!(color_enum.first(), 0);
    /// assert_eq!(color_enum.last(), 2);
    /// assert!(color_enum.is_valid(1)); // Red
    /// ```
    pub fn enum_type<T: ComponentId>(&self) -> EnumType<'_, T> {
        EnumType::new(self.into())
    }
}
