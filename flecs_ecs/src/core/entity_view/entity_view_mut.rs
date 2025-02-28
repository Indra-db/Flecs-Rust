use core::ffi::c_void;

use flecs_ecs::core::*;
use sys::EcsIsA;

use crate::sys;

use self::flecs::FlecsTrait;

// functions in here match most of the functions in the c++ entity and entity_builder class
impl<'a> EntityView<'a> {
    /// Adds an ID to the entity.
    ///
    /// The provided `id` can represent various types, including a component, a pair, a tag, or another entity.
    ///
    /// # Panics
    ///
    /// This function will panic if the `id` does not meet the following constraints:
    /// - The `id` must be either a zero-sized type (ZST), an entity, or a type that implements a constructor hook.
    /// - Types that implement the [`Default`][core::default::Default] trait automatically fulfill the constructor hook requirement.
    ///
    /// This panic occurs because an invalid `id` type may result in reading uninitialized data, leading to undefined behavior.
    ///
    /// # Usage
    ///
    /// For types that are not ZST and do not implement a constructor hook, use the `set_id` method to safely initialize the `id`.
    pub fn add_id(self, id: impl IntoId) -> Self {
        let id = *id.into();
        let world = self.world.world_ptr_mut();

        check_add_id_validity(world, id);

        unsafe { sys::ecs_add_id(world, *self.id, id) }
        self
    }

    /// Adds an ID to the entity unchecked. Useful for run-time components.
    ///
    /// The provided `id` can represent various types, including a component, a pair, a tag, or another entity.
    ///
    /// # Safety
    /// This function is unsafe because it does not check if the `id` is a valid type nor if
    /// the `id` implements a constructor hook if it is not a zero-sized type (ZST).
    /// If the id is a type without a constructor hook, it could cause you to read uninitialized data.
    /// The caller must ensure:
    /// - The `id` is a valid type
    /// - Component data is initialized before use if the type is not a ZST
 

    /// # See Also
    ///
    /// * [`add_id`](Self::add_id) - The safe version of this function
    /// * [`set_id`](Self::set_id) - For setting component data
    pub unsafe fn add_id_unchecked(self, id: impl IntoId) -> Self {
        let id = *id.into();
        let world = self.world.world_ptr_mut();

        unsafe { sys::ecs_add_id(world, *self.id, id) }
        self
    }

  

    /// Add a component or pair to an entity.
    ///
    /// This is a type-safe way to add a component or relationship to an entity.
    ///
 
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component or pair to add
    
    pub fn add<T>(self) -> Self
    where
        T: ComponentOrPairId,
    {
        let world = self.world;
        self.add_id(T::get_id(world))
    }

    /// Adds a flecs trait to the entity.
    ///
    /// Traits are special components that can be added to an entity to 
    /// provide additional functionality.
      
    /// # Type Parameters
    ///
    /// * `T`: The trait component to add, must implement `FlecsTrait`
   
    pub fn add_trait<T>(self) -> Self
    where
        T: ComponentOrPairId,
        T::First: FlecsTrait,
    {
        let world = self.world;
        unsafe { self.add_id_unchecked(T::get_id(world)) }
    }

    /// Override a component on an entity.
    /// 
    /// This is useful when you want to override a component that is inherited from a prefab
    /// on a per-entity basis.
    ///
    /// # Panics
    ///
    /// This function panics if the entity does not have the component to override
    /// (typically through inheritance from a prefab).
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type to override
    pub fn override_type<T>(self) -> Self
    where
        T: ComponentOrPairId,
    {
        let id = T::get_id(self.world);
        let world_ptr = self.world.world_ptr_mut();

        if unsafe { sys::ecs_get_target_for_id(world_ptr, *self.id, EcsIsA, id) } == 0 {
            panic!("Entity does not have the component to override");
        }
        unsafe { self.add_id_unchecked(id) }
    }

    /// Adds a pair to the entity using a component for the first element.
    ///
    /// This creates a relationship where the first element is the specified component type
    /// and the second element is the provided entity.
    ///
    /// # Panics
    ///
    /// This function panics if:
    /// - The first element is not a tag (zero-sized type) and does not implement Default
    /// - The second element is not a valid component or entity
    ///
    /// # Type Parameters
    ///
    /// * `First`: Component type for the first element of the pair
    pub fn add_first<First: ComponentId>(self, second: impl Into<Entity>) -> Self {
        const {
            if !First::IS_TAG && !First::IMPLS_DEFAULT {
                panic!("Adding an element that is not a Tag / Zero sized type requires to implement Default");
            }
        }

        let world = self.world;
        let world_ptr = world.world_ptr();

        let second = *second.into();

        let is_valid_id = unsafe { sys::ecs_id_is_valid(world_ptr, second) };

        if !is_valid_id {
            panic!("Id is not a valid component or entity.");
        }

        if First::IS_TAG {
            let is_second_not_tag = unsafe { sys::ecs_get_typeid(world_ptr, second) != 0 };

            if is_second_not_tag {
                assert!(has_default_hook(world_ptr,second),"second id is not a zero-sized type (ZST) such as a Tag or Entity or does not implement the Default hook for a non ZST type. Default hooks are automatically implemented if the type has a Default trait.");
            }
        }

        // SAFETY: we know that the id is a valid because first is a Type and second has been checked
        unsafe { self.add_id_unchecked((First::id(world), second)) }
    }

    /// Adds a pair to the entity using a component for the second element.
    ///
    /// This creates a relationship where the first element is the provided entity
    /// and the second element is the specified component type.
    /// # Safety
    ///
    /// Caller must ensure the id is a non-ZST type or implements Default.
    /// Otherwise, it could cause you to read uninitialized payload data.
    /// Use `set_second` for non-ZST types without Default.
    
    ///
    /// # Type Parameters
    ///
    /// * `Second`: Component type for the second element of the pair
    pub fn add_second<Second: ComponentId>(self, first: impl Into<Entity>) -> Self {
        let world = self.world;
        let world_ptr = world.world_ptr();

        let first = *first.into();

        let is_valid = unsafe { sys::ecs_id_is_valid(world_ptr, first) };

        if !is_valid {
            panic!("Id is not a valid component or entity.");
        }

        let is_first_tag = unsafe { sys::ecs_get_typeid(world_ptr, first) == 0 };

        if is_first_tag {
            if !Second::IS_TAG && !Second::IMPLS_DEFAULT {
                panic!("first id is a tag type such as a Tag or Entity, but second id is not a zero-sized type (ZST) such as a Tag or Entity or does not implement the Default hook for a non ZST type. Default hooks are automatically implemented if the type has a Default trait.");
            }
        } else {
            assert!(has_default_hook(world_ptr,first),"first id is not a zero-sized type (ZST) such as a Tag or Entity and does not implement the Default hook.  Default hooks are automatically implemented if the type has a Default trait.
                Use `set_id` or `set_pair`.");
        }

        // SAFETY: we know that the id is a valid because first is a Type and second has been checked
        self.add_id((first, Second::id(world)))
    }

    /// Adds a pair to the entity composed of a tag and an enum constant.
    ///
    /// This method is specifically designed for working with Flecs enums,
    /// where the first element is a tag and the second element is an enum value.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The tag component type (first element of pair)
    /// * `Second`: The enum component type (second element of pair)
    pub fn add_pair_enum<First, Second>(self, enum_value: Second) -> Self
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        const {
            if !First::IS_TAG && !First::IMPLS_DEFAULT {
                panic!("Adding an element that is not a Tag / Zero sized type requires to implement Default");
            }
        }
        let world = self.world;
        let enum_id = enum_value.id_variant(world);
        unsafe { self.add_id_unchecked((First::id(world), enum_id)) }
    }

   
    /// Adds an enum constant to an entity.
    ///
    /// Creates a pair where the first element is the enum type,
    /// and the second element is the specific enum constant.
    /// This function works only with regular (C style) enumerations.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type with required trait implementations
    pub fn add_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        self,
        enum_value: T,
    ) -> Self {
        let world = self.world;
        let first = T::id(world);
        // SAFETY: we know that the enum_value is a valid because of the T::id call
        let second = unsafe { enum_value.id_variant_unchecked(world) };
        ecs_assert!(
            second != 0,
            FlecsErrorCode::InvalidParameter,
            "Component was not found in reflection data."
        );
        unsafe { self.add_id_unchecked((first, second)) }
    }

   
    /// Conditionally adds or removes an ID based on a condition.
    ///
    /// This operation adds the ID if the condition is true, and removes it if false.
    /// This provides a convenient way to toggle components based on runtime conditions.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Type that can be converted to an ID
    ///
    /// # Arguments
    ///
    /// * `id`: The ID to conditionally add or remove
    /// * `condition`: Whether to add the ID (true) or remove it (false)
    pub fn add_id_if<T>(self, id: T, condition: bool) -> Self
    where
        T: IntoId,
    {
        if condition {
            self.add_id(id)
        } else {
            // the compiler will optimize this branch away since it's known at compile time
            if T::IS_PAIR {
                // If second is 0 or if relationship is exclusive, use wildcard for
                // second which will remove all instances of the relationship.
                // Replacing 0 with Wildcard will make it possible to use the second
                // as the condition.
                let first = id.get_id_first();
                let mut second = id.get_id_second();
                if second == 0
                    || unsafe { sys::ecs_has_id(self.world.world_ptr(), *first, ECS_EXCLUSIVE) }
                {
                    second = ECS_WILDCARD.into();
                }
                self.remove_id((first, second))
            } else {
                self.remove_id(id)
            }
        }
    }

    /// Conditionally adds or removes a component based on a condition.
    ///
    /// This operation adds the component if the condition is true, and removes it if false.
    /// This provides a type-safe way to toggle components based on runtime conditions.
   
    /// /// # Type Parameters
    ///
    /// * `T`: The component type to conditionally add or remove
    ///
    /// # Arguments
    ///
    /// * `condition`: Whether to add the component (true) or remove it (false)
    pub fn add_if<T: ComponentOrPairId>(self, condition: bool) -> Self {
        let world = self.world;
        if condition {
            self.add::<T>()
        } else {
            let id = T::get_id(world);
            // the compiler will optimize this branch away since it's known at compile time
            if T::IS_PAIR {
                // If second is 0 or if relationship is exclusive, use wildcard for
                // second which will remove all instances of the relationship.
                // Replacing 0 with Wildcard will make it possible to use the second
                // as the condition.
                let first = ecs_first(id);
                let mut second = ecs_second(id);
                if second == 0
                    || unsafe { sys::ecs_has_id(self.world.world_ptr(), *first, ECS_EXCLUSIVE) }
                {
                    second = ECS_WILDCARD.into();
                }
                self.remove_id((first, second))
            } else {
                self.remove_id(id)
            }
        }
    }

    /// Conditionally adds or removes a pair with a specified first element.
    ///
    /// This operation adds the pair if the condition is true, and removes it if false.
    /// The pair consists of the specified first element and second entity.
    ///  # Type Parameters
    ///
    /// * `First`: The component type for the first element of the pair
    ///
    /// # Arguments
    ///
    /// * `second`: The entity to use as the second element of the pair
    /// * `condition`: Whether to add the pair (true) or remove it (false)
    pub fn add_first_if<First: ComponentId>(
        self,
        second: impl Into<Entity>,
        condition: bool,
    ) -> Self {
        let world = self.world;
        self.add_id_if((First::id(world), second.into()), condition)
    }

    /// Conditionally adds or removes a pair with a specified second element.
    ///
    /// This operation adds the pair if the condition is true, and removes it if false.
    /// The pair consists of the specified entity as first element and the component type as second.
    ///
    ///  # Type Parameters
    ///
    /// * `Second`: The component type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The entity to use as the first element of the pair
    /// * `condition`: Whether to add the pair (true) or remove it (false)
    pub fn add_second_if<Second: ComponentId>(
        self,
        first: impl Into<Entity>,
        condition: bool,
    ) -> Self {
        let world = self.world;
        self.add_id_if((first.into(), Second::id(world)), condition)
    }

    /// Conditionally adds or removes an enum value based on a condition.
    ///
    /// This operation adds the enum value if the condition is true, and removes it if false.
    /// Creates a relationship between the enum type and the specific enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type with required trait implementations
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to conditionally add
    /// * `condition`: Whether to add the enum value (true) or remove it (false)
    pub fn add_enum_if<T>(self, enum_value: T, condition: bool) -> Self
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let world = self.world;
        // SAFETY: we know that the enum_value is a valid because of the T::id call
        self.add_id_if(
            (T::id(world), unsafe {
                enum_value.id_variant_unchecked(world)
            }),
            condition,
        )
    }

    /// Removes an ID from an entity.
    ///
    /// This function removes the specified ID (component, pair, tag, or entity)
    /// from the entity.
    ///
    /// # Arguments
    ///
    /// * `id`: The ID to remove from the entity
    pub fn remove_id(self, id: impl IntoId) -> Self {
        unsafe { sys::ecs_remove_id(self.world.world_ptr_mut(), *self.id, *id.into()) }
        self
    }

   /// Removes a component or pair from an entity.
   ///
   /// This is a type-safe way to remove a component or relationship from an entity.
   ///
   /// # Type Parameters
   ///
   /// * `T`: The component or pair type to remove
    pub fn remove<T: ComponentOrPairId>(self) -> Self {
        let world = self.world;

        //this branch will be compiled away in release mode
        if T::IS_ENUM {
            self.remove_id((T::get_id(world), ECS_WILDCARD))
        } else {
            self.remove_id(T::get_id(world))
        }
    }


    /// Removes a pair of tag and enum constant from an entity.
    ///
    /// This function specifically removes a relationship where the first element
    /// is a tag component and the second element is an enum constant.
    ///
     /// # Type Parameters
    ///
    /// * `First`: The tag component type (first element of pair)
    /// * `Second`: The enum component type (second element of pair)
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum constant to remove
    pub fn remove_enum_tag<First, Second>(self, enum_value: Second) -> Self
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let world = self.world;
        self.remove_id((First::id(world), enum_value.id_variant(world)))
    }

    /// Removes a pair with a specified first element from an entity.
    ///
    /// This function removes a relationship where the first element
    /// is the specified component type and the second element is the provided entity.
    ///
    ///  # Type Parameters
    ///
    /// * `First`: Component type for the first element of the pair
    ///
    /// # Arguments
    ///
    /// * `second`: The entity used as the second element of the pair
    pub fn remove_first<First: ComponentId>(self, second: impl Into<Entity>) -> Self {
        let world = self.world;
        self.remove_id((First::id(world), second.into()))
    }

    /// Removes a pair with a specified second element from an entity.
    ///
    /// This function removes a relationship where the first element
    /// is the provided entity and the second element is the specified component type.
    ///
    ///  # Type Parameters
    ///
    /// * `Second`: Component type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The entity used as the first element of the pair
    pub fn remove_second<Second: ComponentId>(self, first: impl Into<Entity>) -> Self {
        let world = self.world;
        self.remove_id((first.into(), Second::id(world)))
    }

    /// Establishes an inheritance relationship with another entity.
    ///
    /// This is a shortcut for adding a pair of (IsA, entity). The entity will inherit
    /// all components and tags from the target entity, similar to how a class inherits
    /// from a parent class.
    ///
    /// # Arguments
    ///
    /// * `second`: The entity to inherit from
    pub fn is_a_id(self, second: impl Into<Entity>) -> Self {
        unsafe { self.add_id_unchecked((ECS_IS_A, second.into())) }
    }

    /// Establishes an inheritance relationship with an entity associated with a component type.
    ///
    /// This is a type-safe shortcut for `is_a_id` where the target entity
    /// is the one associated with the specified component type.
    ///
    ///  # Type Parameters
    ///
    /// * `T`: The component type whose associated entity will be inherited from
    pub fn is_a<T: ComponentId>(self) -> Self {
        let world = self.world;
        self.is_a_id(T::id(world))
    }


    ///  Establishes a parent-child relationship with another entity.
    /// This is a shortcut for adding a pair of (ChildOf, entity) [`add_id((flecs::ChildOf::ID, entity))`]. This relationship
    /// creates a hierarchical structure where the current entity becomes a child
    /// of the specified parent entity.
    ///
    /// # Arguments
    /// * `parent`: The entity to set as parent
    pub fn child_of_id(self, parent: impl Into<Entity>) -> Self {
        unsafe { self.add_id_unchecked((ECS_CHILD_OF, parent.into())) }
    }

    /// Establishes a parent-child relationship with an entity associated with a component type.
    ///
    /// This is a type-safe shortcut for `child_of_id` where the parent entity
    /// is the one associated with the specified component type.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type whose associated entity will be the parent
    pub fn child_of<T: ComponentId>(self) -> Self {
        let world = self.world;
        self.child_of_id(T::id(world))
    }

    /// Establishes a dependency relationship with another entity.
    ///
    /// This is a shortcut for adding a pair of (DependsOn, entity). 
    /// This relationship creates a dependency where the current entity 
    /// depends on the specified entity.
    ///
    /// /// # Arguments
    ///
    /// * `second`: The entity that the current entity depends on
    pub fn depends_on_id(self, second: impl Into<Entity>) -> Self {
        unsafe { self.add_id_unchecked((ECS_DEPENDS_ON, second.into())) }
    }

    ///  Establishes a dependency relationship with an entity associated with a component type.

    /// This is a type-safe shortcut for `depends_on_id [add_id((flecs::Dependency::ID, entity))]` where the dependency
    /// is associated with the specified component type.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type whose associated entity will be depended on
  
    pub fn depends_on<T: ComponentId + ComponentType<Struct>>(self) -> Self {
        let world = self.world;
        self.depends_on_id(T::id(world))
    }

   
    /// Establishes a dependency relationship with an enum value.
    ///
    /// Creates a dependency where the current entity depends on
    /// the entity associated with the specified enum constant.
    ///
    ///  # Type Parameters
    ///
    /// * `T`: The enum type with required trait implementations
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to depend on

    pub fn depends_on_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        self,
        enum_value: T,
    ) -> Self {
        let world = self.world;
        self.depends_on_id(enum_value.id_variant(world))
    }

    /// Establishes a slot relationship with another entity.
    ///
    /// This is a shortcut for adding a pair of (SlotOf, entity).
    /// Slots are used in prefab hierarchies to mark specific places where
    /// entities can be instantiated.
    ///
    ///  # Arguments
    ///
    /// * `second`: The entity to create a slot for
    pub fn slot_of_id(self, second: impl Into<Entity>) -> Self {
        unsafe { self.add_id_unchecked((ECS_SLOT_OF, second.into())) }
    }

   
    /// Establishes a slot relationship with an entity associated with a component type.
    ///
    /// This is a type-safe shortcut for `slot_of_id [add_id((flecs::SlotOf::ID, entity))]` where the entity
    /// is the one associated with the specified component type.
    ///
     /// # Type Parameters
    ///
    /// * `T`: The component type whose associated entity will have this slot
    pub fn slot_of<T: ComponentId>(self) -> Self {
        let world = self.world;
        self.slot_of_id(T::id(world))
    }

    
    /// Creates a slot for the current entity's parent.
    ///
    /// This is a shortcut for `add_id((flecs::SlotOf::ID, target(ChildOf)))`. It automatically
    /// creates a slot relationship with the entity that is the parent of
    /// the current entity via the ChildOf relationship.
    ///
    /// # Panics
    ///
    /// This function panics if the entity does not have a ChildOf relationship.
    pub fn slot(self) -> Self {
        ecs_assert!(
            self.target::<flecs::ChildOf>(0).is_some(),
            FlecsErrorCode::InvalidParameter,
            "add ChildOf pair before using slot()"
        );
        let id = self.target_id(ECS_CHILD_OF, 0);
        self.slot_of_id(id.expect("ChildOf pair not found"))
    }

    /// Marks an ID for auto-overriding in inherited entities.
    ///
    /// When an entity inherits from a base entity (using the `IsA` relationship),
    /// any IDs marked for auto-overriding on the base will be automatically
    /// overridden by the inheriting entity, preventing them from being shared.
    ///
    /// # Arguments
  
    ///
    /// * `id`: The ID to mark for auto-overriding
    pub fn auto_override_id(self, id: impl IntoId) -> Self {
        unsafe { self.add_id_unchecked(ECS_AUTO_OVERRIDE | id.into()) }
    }

     /// Marks a component for auto-overriding in inherited entities.
    ///
    /// This is a type-safe version of `auto_override_id`. When an entity inherits 
    /// from a base entity, components marked for auto-overriding will be 
    /// automatically overridden, preventing them from being shared.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type to mark for auto-overriding
    pub fn auto_override<T: ComponentOrPairId>(self) -> Self {
        let world = self.world;
        self.auto_override_id(T::get_id(world))
    }

    /// Marks a pair for auto-overriding where the first element is a specific component.
    ///
    /// When an entity inherits from a base entity, pairs marked for auto-overriding 
    /// will be automatically overridden by the inheriting entity, preventing them from
    /// being shared.
    ///
    ///  # Type Parameters
    ///
    /// * `First`: The component type for the first element of the pair
    ///
    /// # Arguments
    ///
    /// * `second`: The entity to use as the second element of the pair
    pub fn auto_override_first<First: ComponentId + DataComponent>(
        self,
        second: impl Into<Entity>,
    ) -> Self {
        let world = self.world;
        let pair_id = ecs_pair(First::id(world), *second.into());
        self.auto_override_id(pair_id)
    }

    /// Marks a pair for auto-overriding where the second element is a specific component.  
    ///
    /// When an entity inherits from a base entity, pairs marked for auto-overriding 
    /// will be automatically overridden by the inheriting entity, preventing them from
    /// being shared.
    ///  # Type Parameters
    ///
    /// * `Second`: The component type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The entity to use as the first element of the pair
    pub fn auto_override_second<Second: ComponentId + DataComponent>(
        self,
        first: impl Into<Entity>,
    ) -> Self {
        let world = self.world;
        let pair_id = ecs_pair(*first.into(), Second::id(world));
        self.auto_override_id(pair_id)
    }

    /// Sets a component for an entity and marks it for auto-overriding.
    ///
    /// This function combines setting a component and marking it for 
    /// auto-overriding in a single operation.
    ///
    /// # Arguments
    ///
    /// * `id`: The component ID to set and mark for auto-overriding
    pub fn set_auto_override_id(self, id: impl IntoId) -> Self {
        unsafe {
            sys::ecs_add_id(
                self.world.world_ptr_mut(),
                *self.id,
                ECS_AUTO_OVERRIDE | *id.into(),
            );
        }
        self
    }

     /// Sets a component value and marks it for auto-overriding.
    ///
    /// This function combines setting a component's data and marking it for 
    /// auto-overriding in a single operation. This is useful for prefabs where
    /// you want to provide a default value but ensure each instance gets its own copy.
    ///
    ///  # Type Parameters
    ///
    /// * `T`: The component type to set and mark for auto-overriding
    ///
    /// # Arguments
    ///
    /// * `component`: The component data to set
    pub fn set_auto_override<T: ComponentId + DataComponent + ComponentType<Struct>>(
        self,
        component: T,
    ) -> Self {
        self.auto_override::<T>().set(component)
    }

    /// Sets a pair value and marks it for auto-overriding.
    ///
    /// This function combines setting a pair's data and marking it for 
    /// auto-overriding in a single operation.
    ///
    ///  # Type Parameters
    ///
    /// * `First`: The component type for the first element of the pair
    /// * `Second`: The component type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `data`: The data to set for the pair
    pub fn set_pair_override<First, Second>(
        self,
        data: <(First, Second) as ComponentOrPairId>::CastType,
    ) -> Self
    where
        First: ComponentId,
        Second: ComponentId,
        (First, Second): ComponentOrPairId,
        <(First, Second) as ComponentOrPairId>::CastType: DataComponent,
    {
        let id_pair = <(First, Second) as ComponentOrPairId>::get_id(self.world);
        self.auto_override_id(id_pair).set_id(data, id_pair)
    }

    /// Sets a pair where the first element is a component and marks it for auto-overriding.
    ///
    /// This function sets data for a pair with the first element as the specified component
    /// and the second element as the provided entity, then marks it for auto-overriding.
    ///
    ///  # Safety
    ///
    /// Caller must ensure that `First` and `second` pair id data type match the provided data.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The component type for the first element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The component data to set
    /// * `second`: The entity to use as the second element of the pair
    pub unsafe fn set_auto_override_first<First>(
        self,
        first: First,
        second: impl Into<Entity>,
    ) -> Self
    where
        First: ComponentId + ComponentType<Struct> + DataComponent,
    {
        let second_id = *second.into();
        let first_id = First::id(self.world);
        let pair_id = ecs_pair(first_id, second_id);
        self.auto_override_id(pair_id).set_id(first, pair_id)
    }

    /// Sets a pair where the second element is a component and marks it for auto-overriding.
    ///
    /// This function sets data for a pair with the first element as the provided entity
    /// and the second element as the specified component, then marks it for auto-overriding.
    ///
    ///  # Safety
    ///
    /// Caller must ensure that `first` and `Second` pair id data type match the provided data.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The component type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `second`: The component data to set
    /// * `first`: The entity to use as the first element of the pair
    pub unsafe fn set_auto_override_second<Second>(
        self,
        second: Second,
        first: impl Into<Entity>,
    ) -> Self
    where
        Second: ComponentId + ComponentType<Struct> + DataComponent,
    {
        let first_id = first.into();
        let second_id = Second::id(self.world);
        let pair_id = ecs_pair(*first_id, second_id);
        self.auto_override_id(pair_id).set_id(second, pair_id)
    }

    /// Sets a component value for the entity.
    ///
    /// If the entity does not have the component, it will be added.
    /// If it already has the component, the value will be updated.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type to set
    ///
    /// # Arguments
    ///
    /// * `component`: The component data to set
    pub fn set<T: ComponentId + DataComponent>(self, component: T) -> Self {
        set_helper(
            self.world.world_ptr_mut(),
            *self.id,
            component,
            T::id(self.world),
        );
        self
    }

    /// Sets the data of a component specified by ID.
    ///
    ///  This function allows setting data for components or pairs specified by their ID.
    /// It provides more flexibility than the `set` method for working with runtime IDs.
    
    /// # Safety
    ///
    /// Caller must ensure that `data` is a valid data for the id.
    ///
    /// ```no_run
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct InWorld;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let position = world.component::<Position>();
    /// let in_world = world.component::<InWorld>();
    ///
    /// let entity = world.entity();
    ///
    /// // using a tuple indicates a relationship. It doesn't have to be a relationship.
    /// entity.set_id(Position { x: 10.0, y: 20.0 }, (in_world, position));
    /// // no relationship
    /// entity.set_id(Position { x: 1.0, y: 2.0 }, position);
    ///
    /// entity.get::<&(InWorld, Position)>(|pos| {
    ///     // ...
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`EntityView::add`]
    /// * [`EntityView::add_id`]
    /// * [`EntityView::set`]
    /// * [`EntityView::set_pair`]
    /// 
    
    /// # Type Parameters
    ///
    /// * `T`: The type of data to set
    ///
    /// # Arguments
    ///
    /// * `data`: The component data to set
    /// * `id`: The ID (component or pair) to set the data for
    ///
    /// # Panics
    ///
    /// This function panics if the data type does not match the type associated with the ID.
    /// For pairs, this is the first non-ZST element type.
    pub fn set_id<T>(self, data: T, id: impl IntoId) -> Self
    where
        T: ComponentId + DataComponent,
    {
        let world = self.world.world_ptr_mut();
        let id = *id.into();
        let data_id = T::id(self.world);
        let id_data_id = unsafe { sys::ecs_get_typeid(world, id) };

        if data_id != id_data_id {
            panic!("Data type does not match id type. For pairs this is the first element occurrence that is not a zero-sized type (ZST).");
        }

        set_helper(world, *self.id, data, id);
        self
    }

   /// Sets a pair value for an entity.
    ///
    /// This is a type-safe way to set data for a relationship pair.
    /// If the entity does not have the pair, it will be added.
    /// If it already has the pair, the value will be updated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct InWorld;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity();
    ///
    /// entity.set_pair::<InWorld, _>(Position { x: 1.0, y: 2.0 });
    ///
    /// entity.get::<&(InWorld, Position)>(|pos| {
    ///     // ...
    /// });
    /// ```
    /// # Type Parameters
    ///
    /// * `First`: The component type for the first element of the pair
    /// * `Second`: The component type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `data`: The data to set for the pair
    ///
    /// # Panics
    ///
    /// This function panics if both elements of the pair are tags (zero-sized types).
    /// For tag relationships, use `add::<(Tag1, Tag2)>()` instead.
    pub fn set_pair<First, Second>(
        self,
        data: <(First, Second) as ComponentOrPairId>::CastType,
    ) -> Self
    where
        First: ComponentId,
        Second: ComponentId,
        (First, Second): ComponentOrPairId,
    {
        const {
            assert!(!<(First, Second) as ComponentOrPairId>::IS_TAGS, "setting tag relationships is not possible with `set_pair`. use `add::<(Tag1, Tag2)()` instead.");
        };

        let pair_id = ecs_pair(First::id(self.world), Second::id(self.world));

        ecs_assert!(
            unsafe { sys::ecs_get_typeid(self.world.ptr_mut(), pair_id) } != 0,
            FlecsErrorCode::InvalidOperation,
            "Pair is not a (data) component. Possible cause: PairIsTag trait"
        );

        set_helper(self.world.world_ptr_mut(), *self.id, data, pair_id);
        self
    }

    /// Sets a pair where the first element is a component with data.
    ///
    /// This function sets data for a pair where the first element is the specified component
    /// and the second element is the provided entity.
    ///
    ///
    /// # Type Parameters
    ///
    /// * `First`: The component type for the first element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The component data to set
    /// * `second`: The entity to use as the second element of the pair
    ///
    /// # Panics
    ///
    /// This function panics if the first type does not match the data type of the pair.
    pub fn set_first<First>(self, first: First, second: impl Into<Entity>) -> Self
    where
        First: ComponentId + DataComponent,
    {
        let world_ptr = self.world.world_ptr_mut();
        let first_id = First::id(self.world);
        let second_id = *second.into();
        let pair_id = ecs_pair(first_id, second_id);
        let data_id = unsafe { sys::ecs_get_typeid(world_ptr, pair_id) };

        if data_id != first_id {
            panic!("First type does not match id data type. For pairs this is the first element occurrence that is not a zero-sized type (ZST).");
        }

        set_helper(world_ptr, *self.id, first, pair_id);
        self
    }

    /// Sets a pair where the second element is a component with data.
    ///
    /// This function sets data for a pair where the first element is the provided entity
    /// and the second element is the specified component.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The component type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The entity to use as the first element of the pair
    /// * `second`: The component data to set
    ///
    /// # Panics
    ///
    /// This function panics if the second type does not match the data type of the pair.
    pub fn set_second<Second>(self, first: impl Into<Entity>, second: Second) -> Self
    where
        Second: ComponentId + ComponentType<Struct> + DataComponent,
    {
        let world = self.world.world_ptr_mut();
        let first_id = *first.into();
        let second_id = Second::id(self.world);
        let pair_id = ecs_pair(first_id, second_id);
        // NOTE: we could this safety check optional
        let data_id = unsafe { sys::ecs_get_typeid(world, pair_id) };

        if data_id != second_id {
            panic!("Second type does not match id data type. For pairs this is the first element occurrence that is not a zero-sized type (ZST).");
        }

        set_helper(world, *self.id, second, pair_id);
        self
    }

    /// Sets a pair where the first element is a component and the second is an enum constant.
    ///
    /// This function sets data for a pair where the first element is the specified component
    /// and the second element is the enum constant.
    ///
   /// # Type Parameters
    ///
    /// * `First`: The component type for the first element of the pair
    /// * `Second`: The enum type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `enum_variant`: The enum constant to use as the second element
    /// * `first`: The component data to set
    pub fn set_pair_enum<First, Second>(self, enum_variant: Second, first: First) -> Self
    where
        First: ComponentId + ComponentType<Struct> + DataComponent,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        set_helper(
            self.world.world_ptr_mut(),
            *self.id,
            first,
            ecs_pair(First::id(self.world), **enum_variant.id_variant(self.world)),
        );
        self
    }

    /// Sets a pointer to a component with a given ID and size.
    ///
    /// This is a low-level function that allows setting component data from a raw pointer.
    ///
    /// # Safety
    /// 
    /// Caller must ensure that `ptr` points to data that can be accessed as the type 
    /// associated with `id`, and that the size matches the expected size of that type.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the entity.
    /// * `component_id` - The ID of the component to set the pointer to.
    /// * `size` - The size of the component.
    /// * `ptr` - A pointer to the component.
    ///
    pub unsafe fn set_ptr_w_size(
        self,
        id: impl Into<Entity>,
        size: usize,
        ptr: *const c_void,
    ) -> Self {
        sys::ecs_set_id(self.world.world_ptr_mut(), *self.id, *id.into(), size, ptr);
        self
    }

    /// Sets a pointer to a component with a given ID.
    ///
    /// This is a low-level function that allows setting component data from a raw pointer.
    /// Unlike `set_ptr_w_size`, this function automatically retrieves the correct size
    /// for the component.
    ///
    /// 
    /// # Safety
    /// Caller must ensure that `ptr` points to data that can be accessed as the type 
    /// associated with `id`.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the entity.
    /// * `component_id` - The ID of the component to set the pointer to.
    /// * `ptr` - A pointer to the component.
    ///
    /// # Panics
    ///
    /// This function panics if the provided ID is not a valid component ID.
    pub unsafe fn set_ptr(self, id: impl Into<Entity>, ptr: *const c_void) -> Self {
        let id = id.into();
        let cptr: *const sys::EcsComponent = unsafe {
            sys::ecs_get_id(
                self.world.world_ptr_mut(),
                *id,
                sys::FLECS_IDEcsComponentID_,
            )
        } as *const sys::EcsComponent;

        ecs_assert!(
            !cptr.is_null(),
            FlecsErrorCode::InvalidParameter,
            "invalid component id: {:?}",
            id
        );

        self.set_ptr_w_size(id, unsafe { (*cptr).size } as usize, ptr)
    }

    /// Sets the name of the entity.
    ///
    /// The name can be used to lookup the entity using `world.lookup()`.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to be set.
    ///
    pub fn set_name(self, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        unsafe {
            sys::ecs_set_name(
                self.world.world_ptr_mut(),
                *self.id,
                name.as_ptr() as *const _,
            );
        }
        self
    }

    /// Removes the name of the entity.
    ///
    /// After calling this method, the entity will no longer be accessible 
    /// via name-based lookups.
    ///
    pub fn remove_name(self) -> Self {
        unsafe {
            sys::ecs_set_name(self.world.world_ptr_mut(), *self.id, core::ptr::null());
        }
        self
    }

    /// Sets an alias name for the entity.
    ///
    /// An alias is an alternative name that can be used to look up the entity.
    /// Entities can have both a primary name (set with `set_name`) and multiple aliases.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the alias name to be set.
    ///
    pub fn set_alias(self, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        unsafe {
            sys::ecs_set_alias(
                self.world.world_ptr_mut(),
                *self.id,
                name.as_ptr() as *const _,
            );
        }
        self
    }

    /// Enables the entity.
    ///
    /// Enabled entities are matched with systems and can be found with queries.
    /// By default, entities are enabled when created. This method is useful
    /// to re-enable an entity that was previously disabled.
    ///
    pub fn enable_self(self) -> Self {
        unsafe { sys::ecs_enable(self.world.world_ptr_mut(), *self.id, true) }
        self
    }
    /// Enables a component or pair on the entity.
    ///
    /// This allows for selectively enabling specific components or relationships on an entity.
    /// When a component is disabled, it won't be matched by queries unless they explicitly
    /// include disabled components.
    ///
    /// # Arguments
    ///
    /// * `id`: The ID of the component or pair to enable
    pub fn enable_id(self, id: impl IntoId) -> Self {
        unsafe { sys::ecs_enable_id(self.world.world_ptr_mut(), *self.id, *id.into(), true) }
        self
    }

    /// Enables a component or pair on the entity.
    ///
    /// This is a type-safe version of `enable_id` that works with component or pair types.
    /// When a component is disabled, it won't be matched by queries unless they explicitly
    /// include disabled components.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component or pair type to enable.
    ///
    pub fn enable<T: ComponentOrPairId>(self) -> Self {
        let world = self.world;
        self.enable_id(T::get_id(world))
    }

    /// Enables a pair with a specific component as the first element.
    ///
    /// This allows for selectively enabling a relationship pair where the first element
    /// is a component and the second element is an entity.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `second`: The ID of the second element of the pair.
    ///
    pub fn enable_first<First: ComponentId>(self, second: impl Into<Entity>) -> Self {
        let world = self.world;
        self.enable_id((First::id(world), second.into()))
    }

    /// Enables a pair with a specific component as the second element.
    ///
    /// This allows for selectively enabling a relationship pair where the first element
    /// is an entity and the second element is a component.
    ///
    ///
    /// # Type Parameters
    ///
    /// - `Second`: The component type for the second element of the pair
    ///
    /// # Arguments
    ///
    /// - `first`: The entity to use as the first element of the pair.
    ///
  
    pub fn enable_second<Second: ComponentId>(self, first: impl Into<Entity>) -> Self {
        let world = self.world;
        self.enable_id((first.into(), Second::id(world)))
    }

    /// Disables the entity.
    ///
    /// Disabled entities are not matched with systems and cannot be found with queries
    /// unless explicitly specified in the query. This is useful for temporarily removing
    /// an entity from processing without deleting it.
    ///
  
    pub fn disable_self(self) -> Self {
        unsafe { sys::ecs_enable(self.world.world_ptr_mut(), *self.id, false) }
        self
    }

    /// Disables a component or pair on the entity.
    ///
    /// This allows for selectively disabling specific components or relationships on an entity.
    /// When a component is disabled, it won't be matched by queries unless they explicitly
    // include disabled components.
    ///
    /// # Arguments
    ///
    /// - * `id`: The ID of the component or pair to disable
    ///
  
    pub fn disable_id(self, id: impl IntoId) -> Self {
        unsafe { sys::ecs_enable_id(self.world.world_ptr_mut(), *self.id, *id.into(), false) }
        self
    }

    /// Disables a component or pair on the entity.
    ///
    /// This is a type-safe version of `disable_id` that works with component or pair types.
    /// When a component is disabled, it won't be matched by queries unless they explicitly
    /// include disabled components.
    ///
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component or pair type to disable
    ///

    pub fn disable<T: ComponentOrPairId>(self) -> Self {
        let world = self.world;
        self.disable_id(T::get_id(world))
    }

    /// Disables a pair with a specific component as the first element.
    ///
    /// This allows for selectively disabling a relationship pair where the first element
    /// is a component and the second element is an entity.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The component type for the first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `second`: The entity to use as the second element of the pair.
    ///
    pub fn disable_first<First: ComponentId>(self, second: impl Into<Entity>) -> Self {
        let world = self.world;
        self.disable_id((First::id(world), second.into()))
    }


    /// Sets the entity as the default for new entities created within the provided function.
    ///
    /// Any entities created within the function will automatically have this entity added
    /// to them. This is a thread-safe operation.
    /// 
    /// # Arguments
    ///
    /// - `func`: TThe function to execute with this entity as the default.
    pub fn with(self, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_with(self.world.world_ptr_mut(), *self.id);
            func();
            sys::ecs_set_with(self.world.world_ptr_mut(), prev);
        }
        self
    }

    /// Sets a pair with this entity as the second element as the default for new entities.
    ///
    /// Any entities created within the function will automatically have this pair added
    /// to them, where the first element is the specified entity and the second element
    /// is the current entity. This is a thread-safe operation.
    ///
    /// # Arguments
    ///
    /// * `first`: The entity to use as the first element of the pair
    /// * `func`: The function to execute with this pair as the default

    pub fn with_first_id(self, first: impl Into<Entity>, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_with(
                self.world.world_ptr_mut(),
                ecs_pair(*first.into(), *self.id),
            );
            func();
            sys::ecs_set_with(self.world.world_ptr_mut(), prev);
        }
        self
    }

    /// Sets a pair with this entity as the first element as the default for new entities.
    ///
    /// Any entities created within the function will automatically have this pair added
    /// to them, where the first element is the current entity and the second element
    /// is the specified entity. This is a thread-safe operation.
    ///
    /// # Arguments
    ///
     /// * `second`: The entity to use as the second element of the pair
    /// * `func`: The function to execute with this pair as the default
    ///
    pub fn with_second_id(self, second: impl Into<Entity>, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_with(
                self.world.world_ptr_mut(),
                ecs_pair(*self.id, *second.into()),
            );
            func();
            sys::ecs_set_with(self.world.world_ptr_mut(), prev);
        }
        self
    }

    /// Sets a pair with a component as first element and this entity as second element
    /// as the default for new entities.
    ///
    /// Any entities created within the function will automatically have this pair added
    /// to them. This is a thread-safe operation.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The component type for the first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to execute with this pair as the default.
    ///
    pub fn with_first<First: ComponentId>(self, func: impl FnOnce()) -> Self {
        let world = self.world;
        self.with_first_id(First::id(world), func)
    }

    /// Sets a pair with this entity as first element and a component as second element
    /// as the default for new entities.
    ///
    /// Any entities created within the function will automatically have this pair added
    /// to them. This is a thread-safe operation.
    ///
    /// # Type Parameters
    ///
    /// - `Second`: The component type for the second element of the pair.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to execute with this pair as the default.
    ///
    pub fn with_second<Second: ComponentId>(self, func: impl FnOnce()) -> Self {
        let world = self.world;
        self.with_second_id(Second::id(world), func)
    }

    /// Runs a function with the current entity set as the parent scope.
    ///
    /// This creates a new scope where all entities created within the function
    /// will automatically be children of the current entity (they'll have a ChildOf
    /// relationship with it). This is a thread-safe operation.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to execute with this entity as the scope.
    ///
    pub fn run_in_scope(self, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_scope(self.world.world_ptr_mut(), *self.id);
            func();
            sys::ecs_set_scope(self.world.world_ptr_mut(), prev);
        }
        self
    }

    /// Calls the provided function with a world scoped to the entity.
    ///
    /// This creates a new scope where all entities created within the function
    /// will automatically be children of the current entity. Unlike `run_in_scope`,
    /// this method passes the world to the function.
    ///
    ///  # Arguments
    ///
    /// * `f`: The function to execute with a world scoped to this entity
   
    pub fn scope(self, f: impl FnMut(&World)) -> Self {
        let world = &*self.world;
        world.scope_id(self.id, f);
        self
    }

    /// Notifies the ECS that a component or pair identified by ID was modified.
    ///
    /// This is important for triggering OnSet systems and other observers that 
    /// depend on knowing when component values change. When using `set` methods, 
    /// this is called automatically, but when modifying component data directly
    /// (e.g., through a mutable reference), you should call this manually.
    ///
    ///
    /// # Arguments
    ///
    /// * `id`: The ID of the component or pair that was modified.
    ///
    /// # See also
    ///
    /// * [`EntityView::modified()`]
    /// * [`EntityView::modified_first()`]
    /// * [`World::modified()`]
    pub fn modified_id(self, id: impl IntoId) {
        unsafe { sys::ecs_modified_id(self.world.world_ptr_mut(), *self.id, *id.into()) }
    }

    /// Notifies the ECS that a component was modified.
    ///
    /// This is a type-safe version of `modified_id`. It notifies the ECS that
    /// a component or pair of the specified type was modified on this entity.
    ///
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the component that was modified.
    ///
    /// # Panics
    ///
    /// This function panics if `T` is a zero-sized type, as those can't be modified.
    /// 
    /// # See also
    ///
    /// * [`EntityView::modified_first()`]
    /// * [`EntityView::modified_id()`]
    /// * [`World::modified()`]
    pub fn modified<T: ComponentOrPairId>(&self) {
        const {
            assert!(
                core::mem::size_of::<T::CastType>() != 0,
                "cannot modify zero-sized-type / tag components"
            );
        };

        self.modified_id(T::get_id(self.world));
    }

    /// Notifies the ECS that a pair with a specified first element was modified.
    ///
    /// This notifies the ECS that a pair where the first element is the specified
    /// component type and the second element is the provided entity was modified.
    ///
    ///
    /// # Type Parameters
    ///
    /// * `First` - The component type for the first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second` - The entity used as the second element of the pair.
    ///
    ///  # Panics
    ///
    /// This function panics if `First` is a zero-sized type, as those can't be modified.
    /// 
    /// # See also
    ///
    /// * [`EntityView::modified()`]
    /// * [`EntityView::modified_id()`]
    /// * [`World::modified()`]
    pub fn modified_first<First: ComponentId>(self, second: impl Into<Entity>) {
        ecs_assert!(
            core::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            core::any::type_name::<First>()
        );

        self.modified_id((First::id(self.world), second.into()));
    }

    /// Get reference to component specified by id.
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling 'get' for the same component.
    ///
    /// The method accepts a component id argument, which can be used to create a
    /// ref to a component that is different from the provided type. This allows
    /// for creating a base type ref that points to a derived type:
    ///
    /// # Safety
    ///
    /// If the provided component id is not binary compatible with the specified
    /// type, the behavior is undefined.
    ///
    /// ```no_run
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Base {
    ///     x: i32,
    /// };
    /// #[derive(Component)]
    /// struct Derived {
    ///     x: i32,
    /// };
    ///
    /// let world = World::new();
    ///
    /// let base = world.component::<Base>();
    /// let derived = world.component::<Derived>().is_a_id(base);
    ///
    /// let entity = world.entity().set(Derived { x: 10 });
    ///
    /// let base_ref = entity.get_ref_w_id::<Base>(derived.id());
    /// ```
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component for which to get a reference.
    ///
    /// # Arguments
    ///
    /// * `component` - The component id.
    ///
    /// # Returns
    ///
    /// The reference.
    ///
    /// # See also
    ///
    /// * [`EntityView::get_ref()`]
    /// * [`EntityView::get_ref_first()`]
    /// * [`EntityView::get_ref_second()`]
    pub fn get_ref_w_id<T>(&self, component: impl IntoId) -> CachedRef<'a, T::CastType>
    where
        T: ComponentOrPairId,
        T::CastType: DataComponent,
    {
        CachedRef::<T::CastType>::new(self.world, *self.id, *component.into())
    }

    /// Get a reference to a component or pair.
    ///
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling `get` for the same component.
    ///
    /// - `T`: Component for which to get a reference.
    ///
    /// Returns: The reference component.
    ///
    pub fn get_ref<T>(&self) -> CachedRef<'a, T::CastType>
    where
        T: ComponentOrPairId,
        T::CastType: DataComponent,
    {
        CachedRef::<T::CastType>::new(self.world, *self.id, T::get_id(self.world))
    }

    /// Get a reference to the first component of pair
    ///
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling `get` for the same component.
    ///
    /// # Arguments
    ///
    /// * `second` - The entity associated with the second component in the pair.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The type of the first component in the pair.
    ///
    /// # Returns
    ///
    /// A reference to the first component in the pair.
    pub fn get_ref_first<First: ComponentId + DataComponent>(
        self,
        second: impl Into<Entity>,
    ) -> CachedRef<'a, First> {
        let first = First::id(self.world);
        let second = *second.into();
        let pair = ecs_pair(first, second);
        ecs_assert!(
            !(unsafe { sys::ecs_get_type_info(self.world.world_ptr(), pair,) }.is_null()),
            FlecsErrorCode::InvalidParameter,
            "pair is not a component"
        );
        ecs_assert!(
            unsafe { *sys::ecs_get_type_info(self.world.world_ptr(), pair,) }.component == first,
            FlecsErrorCode::InvalidParameter,
            "type of pair is not First"
        );
        CachedRef::<First>::new(self.world, *self.id, pair)
    }

    /// Get a reference to the second component of pair
    ///
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling `get` for the same component.
    ///
    /// # Arguments
    ///
    /// * `first` - The entity associated with the first component in the pair.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The type of the second component in the pair.
    ///
    /// # Returns
    ///
    /// A reference to the first component in the pair.
    ///
    pub fn get_ref_second<Second: ComponentId + DataComponent>(
        &self,
        first: impl Into<Entity>,
    ) -> CachedRef<Second> {
        let first = *first.into();
        let second = Second::id(self.world);
        let pair = ecs_pair(first, second);
        ecs_assert!(
            !(unsafe { sys::ecs_get_type_info(self.world.world_ptr(), pair,) }.is_null()),
            FlecsErrorCode::InvalidParameter,
            "pair is not a component"
        );
        ecs_assert!(
            unsafe { *sys::ecs_get_type_info(self.world.world_ptr(), pair,) }.component == second,
            FlecsErrorCode::InvalidParameter,
            "type of pair is not Second"
        );

        CachedRef::<Second>::new(self.world, *self.id, pair)
    }

    /// Removes all components from an entity without deleting it.
    ///
    /// This operation leaves the entity ID intact but removes all of its components.
    /// The entity will continue to exist but will have no components associated with it.
    ///
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn clear(&self) {
        unsafe { sys::ecs_clear(self.world.world_ptr_mut(), *self.id) }
    }

    /// Delete an entity.
    ///
    /// This operation permanently removes the entity and all its components
    /// from the world. After calling this method, the entity ID will eventually
    /// be recycled for new entities.
    pub fn destruct(self) {
        unsafe { sys::ecs_delete(self.world.world_ptr_mut(), *self.id) }
    }
}
