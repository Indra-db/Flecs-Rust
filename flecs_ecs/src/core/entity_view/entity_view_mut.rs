use std::os::raw::c_void;

use flecs_ecs::core::*;
use sys::EcsIsA;

use crate::sys;

use self::flecs::FlecsTrait;

// functions in here match most of the functions in the c++ entity and entity_builder class
impl<'a> EntityView<'a> {
    fn check_add_id_validity(world: *mut sys::ecs_world_t, id: u64) {
        let is_valid_id = unsafe { sys::ecs_id_is_valid(world, id) };

        if !is_valid_id {
            panic!("Id is not a valid component, pair or entity.");
        }

        let is_not_tag = unsafe { sys::ecs_get_typeid(world, id) != 0 };

        if is_not_tag {
            let hooks = unsafe { sys::ecs_get_hooks_id(world, id) };
            let is_default_hook = unsafe { (*hooks).ctor.is_some() };
            if !is_default_hook {
                panic!("Id is not a ZST type such as a Tag or Entity or does not implement the Default hook for a non ZST type.");
            }
        }
    }
    /// Add an id to an entity.
    /// This Id can be a component, a pair, a tag or another entity.
    ///
    /// # Panics
    ///
    /// Caller must ensure the id is a non ZST types. Otherwise it could cause you to read uninitialized payload data.
    /// use `set_id` for ZST types.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add_id(self, id: impl IntoId) -> Self {
        let id = *id.into();
        let world = self.world.world_ptr_mut();

        Self::check_add_id_validity(world, id);

        unsafe { sys::ecs_add_id(world, *self.id, id) }
        self
    }

    pub(crate) unsafe fn add_id_unchecked(self, id: impl IntoId) -> Self {
        let id = *id.into();
        let world = self.world.world_ptr_mut();

        unsafe { sys::ecs_add_id(world, *self.id, id) }
        self
    }

    /// Add a Tag or Tags relationship to an entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add<T>(self) -> Self
    where
        T: IntoComponentId,
    {
        // const {
        //     if !T::IS_TAGS {
        //         if !T::First::IS_TAG && !T::First::IMPLS_DEFAULT {
        //             panic!("Adding an element that is not a Tag / Zero sized type requires to implement Default");
        //         }
        //         if T::IS_PAIR && !T::Second::IS_TAG && !T::Second::IMPLS_DEFAULT {
        //             panic!("Adding an element that is not a Tag / Zero sized type requires to implement Default");
        //         }
        //     }
        // }
        let world = self.world;
        unsafe { self.add_id_unchecked(T::get_id(world)) }
    }

    /// Adds a flecs trait.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add_trait<T>(self) -> Self
    where
        T: IntoComponentId,
        T::First: FlecsTrait,
    {
        let world = self.world;
        unsafe { self.add_id_unchecked(T::get_id(world)) }
    }

    /// Override a component on an entity.
    /// This is useful if you want to override a component that is inherited by a prefab on a per entity basis
    ///
    /// # Panics
    ///
    /// Caller must ensure the entity has the component to override.
    pub fn override_type<T>(self) -> Self
    where
        T: IntoComponentId,
    {
        let id = T::get_id(self.world);
        let world_ptr = self.world.world_ptr_mut();

        if unsafe { sys::ecs_get_target_for_id(world_ptr, *self.id, EcsIsA, id) } == 0 {
            panic!("Entity does not have the component to override");
        }
        unsafe { self.add_id_unchecked(id) }
    }

    /// Adds a pair to the entity
    ///
    /// # Panics
    ///
    /// Caller must ensure the id is a non ZST types. Otherwise it could cause you to read uninitialized payload data.
    /// use `set_first` for ZST types.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add_first<First: ComponentId>(self, second: impl Into<Entity>) -> Self {
        // const {
        //     if !First::IS_TAG && !First::IMPLS_DEFAULT {
        //         panic!("Adding an element that is not a Tag / Zero sized type requires to implement Default");
        //     }
        // }

        let world = self.world;
        let world_ptr = world.world_ptr_mut();

        let second = *second.into();

        let is_valid_id = unsafe { sys::ecs_id_is_valid(world_ptr, second) };

        if !is_valid_id {
            panic!("Id is not a valid component or entity.");
        }

        if First::IS_TAG {
            let is_second_not_tag = unsafe { sys::ecs_get_typeid(world_ptr, second) != 0 };

            if is_second_not_tag {
                let hooks = unsafe { sys::ecs_get_hooks_id(world_ptr, second) };
                let is_default_hook = unsafe { (*hooks).ctor.is_some() };
                if !is_default_hook {
                    panic!("second id is not a ZST type such as a Tag or Entity or does not implement the Default hook for a non ZST type.");
                }
            }
        }

        // SAFETY: we know that the id is a valid because first is a Type and second has been checked
        unsafe { self.add_id_unchecked((First::get_id(world), second)) }
    }

    /// Adds a pair to the entity
    ///
    /// # Safety
    ///
    /// Caller must ensure the id is a non ZST types. Otherwise it could cause you to read uninitialized payload data.
    /// use `set_second` for ZST types.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add_second<Second: ComponentId>(self, first: impl Into<Entity>) -> Self {
        let world = self.world;
        let world_ptr = world.world_ptr_mut();

        let first = *first.into();

        let is_valid = unsafe { sys::ecs_id_is_valid(world_ptr, first) };

        if !is_valid {
            panic!("Id is not a valid component or entity.");
        }

        let is_first_tag = unsafe { sys::ecs_get_typeid(world_ptr, first) == 0 };

        if is_first_tag {
            if !Second::IS_TAG && !Second::IMPLS_DEFAULT {
                panic!("first id is a tag type such as a Tag or Entity, but second id is not a ZST type such as a Tag or Entity or does not implement the Default hook for a non ZST type.");
            }
        } else {
            let hooks = unsafe { sys::ecs_get_hooks_id(world_ptr, first) };
            let is_default_hook = unsafe { (*hooks).ctor.is_some() };
            if !is_default_hook {
                panic!("first id is not a ZST type such as a Tag or Entity and does not implement the Default hook.
                Use `set` id variant.");
            }
        }

        // SAFETY: we know that the id is a valid because first is a Type and second has been checked
        self.add_id((first, Second::get_id(world)))
    }

    /// Adds a pair to the entity composed of a tag and an enum constant.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add_pair_enum<First, Second>(self, enum_value: Second) -> Self
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        // const {
        //     if !First::IS_TAG && First::IMPLS_DEFAULT {
        //         panic!("Adding an element that is not a Tag / Zero sized type requires to implement Default");
        //     }
        // }
        let world = self.world;
        let enum_id = enum_value.get_id_variant(world);
        unsafe { self.add_id_unchecked((First::get_id(world), enum_id)) }
    }

    /// Adds a pair to the entity where the first element is the enumeration type,
    /// and the second element is the enumeration constant.
    ///
    /// This function works with regular (C style) enumerations as well as enum classes.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The enumeration type, which derives from `ComponentId`, `ComponentType<Enum>`, and `CachedEnumData`.
    ///
    /// # Arguments
    ///
    /// - `enum_value`: The enumeration value.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        self,
        enum_value: T,
    ) -> Self {
        let world = self.world;
        let first = T::get_id(world);
        // SAFETY: we know that the enum_value is a valid because of the T::get_id call
        let second = unsafe { enum_value.get_id_variant_unchecked(world) };
        ecs_assert!(
            second != 0,
            FlecsErrorCode::InvalidParameter,
            "Component was not found in reflection data."
        );
        unsafe { self.add_id_unchecked((first, second)) }
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `component`: The component to add.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
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
                    || unsafe { sys::ecs_has_id(self.world.world_ptr_mut(), *first, ECS_EXCLUSIVE) }
                {
                    second = ECS_WILDCARD.into();
                }
                self.remove_id((first, second))
            } else {
                self.remove_id(id)
            }
        }
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component to add.
    ///
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
    pub fn add_if<T: IntoComponentId>(self, condition: bool) -> Self {
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
                    || unsafe { sys::ecs_has_id(self.world.world_ptr_mut(), *first, ECS_EXCLUSIVE) }
                {
                    second = ECS_WILDCARD.into();
                }
                self.remove_id((first, second))
            } else {
                self.remove_id(id)
            }
        }
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair
    ///
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
    pub fn add_first_if<First: ComponentId>(
        self,
        second: impl Into<Entity>,
        condition: bool,
    ) -> Self {
        let world = self.world;
        self.add_id_if((First::get_id(world), second.into()), condition)
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair
    ///
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
    pub fn add_second_if<Second: ComponentId>(
        self,
        first: impl Into<Entity>,
        condition: bool,
    ) -> Self {
        let world = self.world;
        self.add_id_if((first.into(), Second::get_id(world)), condition)
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `T`: enum type
    ///
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `enum_value`: The enumeration constant.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
    pub fn add_enum_if<T>(self, enum_value: T, condition: bool) -> Self
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        // SAFETY: we know that the enum_value is a valid because of the T::get_id call
        self.add_id_if(
            (T::get_id(world), unsafe {
                enum_value.get_id_variant_unchecked(world)
            }),
            condition,
        )
    }

    /// Remove an entity from an entity.
    ///
    /// # Arguments
    ///
    /// * `component_id`: The entity to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove`
    #[doc(alias = "entity_builder::remove")]
    pub fn remove_id(self, id: impl IntoId) -> Self {
        unsafe { sys::ecs_remove_id(self.world.world_ptr_mut(), *self.id, *id.into()) }
        self
    }

    /// Remove a component from an entity.
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type of the component to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove`
    #[doc(alias = "entity_builder::remove")]
    pub fn remove<T: IntoComponentId>(self) -> Self {
        let world = self.world;

        //this branch will be compiled away in release mode
        if T::IS_ENUM {
            self.remove_id((T::get_id(world), ECS_WILDCARD))
        } else {
            self.remove_id(T::get_id(world))
        }
    }

    /// Remove a pair.
    /// This operation removes a pair to the entity.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the first element of the pair.
    /// * `U`: The type of the second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: the enum constant.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove`
    #[doc(alias = "entity_builder::remove")]
    pub fn remove_enum_tag<First, Second>(self, enum_value: Second) -> Self
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        self.remove_id((First::get_id(world), enum_value.get_id_variant(world)))
    }

    /// Removes a pair.
    /// This operation removes a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove_second`
    #[doc(alias = "entity_builder::remove_second")]
    pub fn remove_first<First: ComponentId>(self, second: impl Into<Entity>) -> Self {
        let world = self.world;
        self.remove_id((First::get_id(world), second.into()))
    }

    /// Removes a pair.
    /// This operation removes a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove_second`
    #[doc(alias = "entity_builder::remove_second")]
    pub fn remove_second<Second: ComponentId>(self, first: impl Into<Entity>) -> Self {
        let world = self.world;
        self.remove_id((first.into(), Second::get_id(world)))
    }

    /// Shortcut for add(IsA, id).
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::is_a`
    #[doc(alias = "entity_builder::is_a")]
    pub fn is_a_id(self, second: impl Into<Entity>) -> Self {
        unsafe { self.add_id_unchecked((ECS_IS_A, second.into())) }
    }

    /// Shortcut for add(IsA, entity).
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type associated with the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::is_a`
    #[doc(alias = "entity_builder::is_a")]
    pub fn is_a<T: ComponentId>(self) -> Self {
        let world = self.world;
        self.is_a_id(T::get_id(world))
    }

    /// Shortcut for add(ChildOf, entity).
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::child_of`
    #[doc(alias = "entity_builder::child_of")]
    pub fn child_of_id(self, parent: impl Into<Entity>) -> Self {
        unsafe { self.add_id_unchecked((ECS_CHILD_OF, parent.into())) }
    }

    /// Shortcut for add(ChildOf, entity).
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type associated with the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::child_of`
    #[doc(alias = "entity_builder::child_of")]
    pub fn child_of<T: ComponentId>(self) -> Self {
        let world = self.world;
        self.child_of_id(T::get_id(world))
    }

    /// Shortcut for add(DependsOn, entity).
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::depends_on`
    #[doc(alias = "entity_builder::depends_on")]
    pub fn depends_on_id(self, second: impl Into<Entity>) -> Self {
        unsafe { self.add_id_unchecked((ECS_DEPENDS_ON, second.into())) }
    }

    /// Shortcut for add(DependsOn, entity).
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type associated with the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::depends_on`
    #[doc(alias = "entity_builder::depends_on")]
    pub fn depends_on<T: ComponentId + ComponentType<Struct>>(self) -> Self {
        let world = self.world;
        self.depends_on_id(T::get_id(world))
    }

    /// Shortcut for add(Dependency, entity) for Enums.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum constant.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::depends_on`
    #[doc(alias = "entity_builder::depends_on")]
    pub fn depends_on_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        self,
        enum_value: T,
    ) -> Self {
        let world = self.world;
        self.depends_on_id(enum_value.get_id_variant(world))
    }

    /// Shortcut for add(SlotOf, entity).
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::slot_of`
    #[doc(alias = "entity_builder::slot_of")]
    pub fn slot_of_id(self, second: impl Into<Entity>) -> Self {
        unsafe { self.add_id_unchecked((ECS_SLOT_OF, second.into())) }
    }

    /// Shortcut for add(SlotOf, entity).
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type associated with the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::slot_of`
    #[doc(alias = "entity_builder::slot_of")]
    pub fn slot_of<T: ComponentId>(self) -> Self {
        let world = self.world;
        self.slot_of_id(T::get_id(world))
    }

    /// Shortcut for add(SlotOf, target(ChildOf)).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::slot`
    #[doc(alias = "entity_builder::slot")]
    pub fn slot(self) -> Self {
        ecs_assert!(
            self.target::<flecs::ChildOf>(0) != 0,
            FlecsErrorCode::InvalidParameter,
            "add ChildOf pair before using slot()"
        );
        let id = self.target_id(ECS_CHILD_OF, 0);
        self.slot_of_id(id)
    }

    /// Mark id for auto-overriding.
    ///
    /// When an entity inherits from a base entity (using the `IsA` relationship)
    /// any ids marked for auto-overriding on the base will be overridden
    /// automatically by the entity.
    ///
    /// # Arguments
    ///
    /// * `id`: The id to mark for overriding.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::override`
    #[doc(alias = "entity_builder::override")]
    pub fn auto_override_id(self, id: impl IntoId) -> Self {
        unsafe { self.add_id_unchecked(ECS_AUTO_OVERRIDE | id.into()) }
    }

    /// Mark component for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component to mark for overriding.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::override`
    #[doc(alias = "entity_builder::override")]
    pub fn auto_override<T: IntoComponentId>(self) -> Self {
        let world = self.world;
        self.auto_override_id(T::get_id(world))
    }

    /// Mark pair for auto-overriding with a given second ID.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::override`
    #[doc(alias = "entity_builder::override")]
    pub fn auto_override_first<First: ComponentId + NotEmptyComponent>(
        self,
        second: impl Into<Entity>,
    ) -> Self {
        let world = self.world;
        let pair_id = ecs_pair(First::get_id(world), *second.into());
        self.auto_override_id(pair_id)
    }

    /// Mark pair for auto-overriding with a given first ID.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::override`
    #[doc(alias = "entity_builder::override")]
    pub fn auto_override_second<Second: ComponentId + NotEmptyComponent>(
        self,
        first: impl Into<Entity>,
    ) -> Self {
        let world = self.world;
        let pair_id = ecs_pair(*first.into(), Second::get_id(world));
        self.auto_override_id(pair_id)
    }

    /// Sets a component for an entity and marks it as overridden.
    ///
    /// This function sets a component for an entity and marks the component
    /// as overridden, meaning that it will not be updated by systems that
    /// typically update this component.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_auto_override`
    #[doc(alias = "entity_builder::set_auto_override")]
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

    /// Sets a component mark override for the entity and sets the component data.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_auto_override`
    #[doc(alias = "entity_builder::set_auto_override")]
    pub fn set_auto_override<T: ComponentId + NotEmptyComponent + ComponentType<Struct>>(
        self,
        component: T,
    ) -> Self {
        self.auto_override::<T>().set(component)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_auto_override`
    pub fn set_pair_override<First, Second>(
        self,
        data: <(First, Second) as FlecsCastType>::CastType,
    ) -> Self
    where
        First: ComponentId,
        Second: ComponentId,
        (First, Second): FlecsCastType,
        <(First, Second) as FlecsCastType>::CastType: NotEmptyComponent,
    {
        let id_pair = <(First, Second) as IntoComponentId>::get_id(self.world);
        self.auto_override_id(id_pair).set_id(data, id_pair)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `First` and `second` pair id data type is the one provided.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_auto_override`
    #[doc(alias = "entity_builder::set_auto_override")]
    pub unsafe fn set_auto_override_first<First>(
        self,
        first: First,
        second: impl Into<Entity>,
    ) -> Self
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let second_id = *second.into();
        let first_id = First::get_id(self.world);
        let pair_id = ecs_pair(first_id, second_id);
        self.auto_override_id(pair_id).set_id(first, pair_id)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `Sirst` and `fecond` pair id data type is the one provided.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_auto_override`
    #[doc(alias = "entity_builder::set_auto_override")]
    pub unsafe fn set_auto_override_second<Second>(
        self,
        second: Second,
        first: impl Into<Entity>,
    ) -> Self
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let first_id = first.into();
        let second_id = Second::get_id(self.world);
        let pair_id = ecs_pair(*first_id, second_id);
        self.auto_override_id(pair_id).set_id(second, pair_id)
    }

    /// Sets a component of type `T` on the entity.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to set on the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set`
    #[doc(alias = "entity_builder::set")]
    pub fn set<T: ComponentId + NotEmptyComponent>(self, component: T) -> Self {
        set_helper(
            self.world.world_ptr_mut(),
            *self.id,
            component,
            T::get_id(self.world),
        );
        self
    }

    /// Sets the data of the specified id. Can be a pair or Component.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `data` is a valid data for the id.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set`
    #[doc(alias = "entity_builder::set")]
    pub fn set_id<T>(self, data: T, id: impl IntoId) -> Self
    where
        T: ComponentId + NotEmptyComponent,
    {
        let world = self.world.world_ptr_mut();
        let id = *id.into();
        let data_id = T::get_id(self.world);
        let id_data_id = unsafe { sys::ecs_get_typeid(world, id) };

        if data_id != id_data_id {
            panic!("Data type does not match id type. For pairs this is the first element occurrence that is not a ZST type.");
        }

        set_helper(world, *self.id, data, id);
        self
    }

    /// Set a pair for an entity.
    /// This operation sets the pair value, and uses the first non tag / ZST as type.
    /// If the data is an flecs enum (Repr(C)), it will use the enum variant id.
    ///
    /// If the entity did not yet have the pair, it will be added, otherwise overridden.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set`
    #[doc(alias = "entity_builder::set")]
    pub fn set_pair<First, Second>(self, data: <(First, Second) as FlecsCastType>::CastType) -> Self
    where
        First: ComponentId,
        Second: ComponentId,
        (First, Second): FlecsCastType,
    {
        // const {
        //     assert!(!<(First, Second) as IntoComponentId>::IS_TAGS, "setting tag relationships is not possible with `set_pair`. use `add_pair` instead.");
        // };

        set_helper(
            self.world.world_ptr_mut(),
            *self.id,
            data,
            ecs_pair(First::get_id(self.world), Second::get_id(self.world)),
        );
        self
    }

    /// Set a pair for an entity using the first element type and a second component ID.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set`
    #[doc(alias = "entity_builder::set")]
    pub fn set_first<First>(self, second: impl Into<Entity>, first: First) -> Self
    where
        First: ComponentId + NotEmptyComponent,
    {
        let world_ptr = self.world.world_ptr_mut();
        let first_id = First::get_id(self.world);
        let second_id = *second.into();
        let pair_id = ecs_pair(first_id, second_id);
        let data_id = unsafe { sys::ecs_get_typeid(world_ptr, pair_id) };

        if data_id != first_id {
            panic!("First type does not match id data type. For pairs this is the first element occurrence that is not a ZST type.");
        }

        set_helper(world_ptr, *self.id, first, pair_id);
        self
    }

    /// Set a pair for an entity using the second element type and a first id.
    ///
    /// # Panics
    ///
    /// Caller must ensure that first is a ZST type or entity and not a pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_second`
    #[doc(alias = "entity_builder::set_second")]
    pub fn set_second<Second>(self, first: impl Into<Entity>, second: Second) -> Self
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let world = self.world.world_ptr_mut();
        let first_id = *first.into();
        let second_id = Second::get_id(self.world);
        let pair_id = ecs_pair(first_id, second_id);
        let data_id = unsafe { sys::ecs_get_typeid(world, pair_id) };

        if data_id != second_id {
            panic!("Second type does not match id data type. For pairs this is the first element occurrence that is not a ZST type.");
        }

        set_helper(world, *self.id, second, pair_id);
        self
    }

    /// Set a pair for an entity.
    /// This operation sets the pair value, and uses First as type. If the
    /// entity did not yet have the pair, it will be added.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `constant`: The enum constant.
    /// * `value`: The value to set.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set`
    #[doc(alias = "entity_builder::set")]
    pub fn set_pair_enum<First, Second>(self, enum_variant: Second, first: First) -> Self
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        set_helper(
            self.world.world_ptr_mut(),
            *self.id,
            first,
            ecs_pair(
                First::get_id(self.world),
                **enum_variant.get_id_variant(self.world),
            ),
        );
        self
    }

    /// Sets a pointer to a component of an entity with a given component ID and size.
    ///
    /// # Safety
    /// Caller must ensure that `ptr` points to data that can be accessed as the type associated with `id`
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the entity.
    /// * `component_id` - The ID of the component to set the pointer to.
    /// * `size` - The size of the component.
    /// * `ptr` - A pointer to the component.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_ptr`
    #[doc(alias = "entity_builder::set_ptr")]
    pub unsafe fn set_ptr_w_size(
        self,
        id: impl Into<Entity>,
        size: usize,
        ptr: *const c_void,
    ) -> Self {
        sys::ecs_set_id(self.world.world_ptr_mut(), *self.id, *id.into(), size, ptr);
        self
    }

    /// Sets a pointer to a component of an entity with a given component ID.
    ///
    /// # Safety
    /// Caller must ensure that `ptr` points to data that can be accessed as the type associated with `id`
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the entity.
    /// * `component_id` - The ID of the component to set the pointer to.
    /// * `ptr` - A pointer to the component.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_ptr`
    #[doc(alias = "entity_builder::set_ptr")]
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
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to be set.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_name`
    #[doc(alias = "entity_builder::set_name")]
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
    pub fn remove_name(self) -> Self {
        unsafe {
            sys::ecs_set_name(self.world.world_ptr_mut(), *self.id, std::ptr::null());
        }
        self
    }

    /// Sets the alias name of the entity.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the alias name to be set.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_alias`
    #[doc(alias = "entity_builder::set_alias")]
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

    /// Enables itself (the entity).
    ///
    /// Enabled entities are matched with systems and can be searched with queries.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::enable`
    #[doc(alias = "entity_builder::enable")]
    pub fn enable_self(self) -> Self {
        unsafe { sys::ecs_enable(self.world.world_ptr_mut(), *self.id, true) }
        self
    }
    /// Enables an ID which represents a component or pair.
    ///
    /// This sets the enabled bit for this component. If this is the first time the component is
    /// enabled or disabled, the bitset is added.
    ///
    /// # Arguments
    ///
    /// - `component_id`: The ID to enable.
    /// - `toggle`: True to enable, false to disable (default = true).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::enable`
    #[doc(alias = "entity_builder::enable")]
    pub fn enable_id(self, id: impl IntoId) -> Self {
        unsafe { sys::ecs_enable_id(self.world.world_ptr_mut(), *self.id, *id.into(), true) }
        self
    }

    /// Enables a component or pair.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component to enable.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::enable`
    #[doc(alias = "entity_builder::enable")]
    pub fn enable<T: IntoComponentId>(self) -> Self {
        let world = self.world;
        self.enable_id(T::get_id(world))
    }

    /// Enables a pair with a specific ID for the second element.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `second`: The ID of the second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::enable`
    #[doc(alias = "entity_builder::enable")]
    pub fn enable_second<First: ComponentId>(self, second: impl Into<Entity>) -> Self {
        let world = self.world;
        self.enable_id((First::get_id(world), second.into()))
    }

    /// Disables self (entity).
    ///
    /// Disabled entities are not matched with systems and cannot be searched with queries,
    /// unless explicitly specified in the query expression.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::disable`
    #[doc(alias = "entity_builder::disable")]
    pub fn disable_self(self) -> Self {
        unsafe { sys::ecs_enable(self.world.world_ptr_mut(), *self.id, false) }
        self
    }

    /// Disables an ID which represents a component or pair.
    ///
    /// This sets the enabled bit for this ID. If this is the first time the ID is
    /// enabled or disabled, the bitset is added.
    ///
    /// # Arguments
    ///
    /// - `component_id`: The ID to disable.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::disable`
    #[doc(alias = "entity_builder::disable")]
    pub fn disable_id(self, id: impl IntoId) -> Self {
        unsafe { sys::ecs_enable_id(self.world.world_ptr_mut(), *self.id, *id.into(), false) }
        self
    }

    /// Disables a component or pair.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component to disable.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::disable`
    #[doc(alias = "entity_builder::disable")]
    pub fn disable<T: IntoComponentId>(self) -> Self {
        let world = self.world;
        self.disable_id(T::get_id(world))
    }

    /// Disables a pair with a specific ID for the second element.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `second`: The ID of the second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::disable`
    #[doc(alias = "entity_builder::disable")]
    pub fn disable_first<First: ComponentId>(self, second: impl Into<Entity>) -> Self {
        let world = self.world;
        self.disable_id((First::get_id(world), second.into()))
    }
    /// Entities created in the function will have the current entity.
    /// This operation is thread safe.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to call.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::with`
    #[doc(alias = "entity_builder::with")]
    pub fn with(self, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_with(self.world.world_ptr_mut(), *self.id);
            func();
            sys::ecs_set_with(self.world.world_ptr_mut(), prev);
        }
        self
    }

    /// Entities created in the function will have a pair consisting of a specified ID and the current entity.
    /// This operation is thread safe.
    ///
    /// # Arguments
    ///
    /// - `first`: The first element of the pair.
    /// - `func`: The function to call.///
    /// # See also
    ///
    /// * C++ API: `entity_builder::with`
    #[doc(alias = "entity_builder::with")]
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

    /// Entities created in the function will have a pair consisting of the current entity and a specified ID.
    /// This operation is thread safe.
    ///
    /// # Arguments
    ///
    /// - `second`: The second element of the pair.
    /// - `func`: The function to call.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::with`
    #[doc(alias = "entity_builder::with")]
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

    /// Entities created in the function will have (First, self).
    /// This operation is thread safe.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to call.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::with`
    #[doc(alias = "entity_builder::with")]
    pub fn with_first<First: ComponentId>(self, func: impl FnOnce()) -> Self {
        let world = self.world;
        self.with_first_id(First::get_id(world), func)
    }

    /// Entities created in the function will have (self, Second)
    /// This operation is thread safe.
    ///
    /// # Type Parameters
    ///
    /// - `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to call.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::with`
    #[doc(alias = "entity_builder::with")]
    pub fn with_second<Second: ComponentId>(self, func: impl FnOnce()) -> Self {
        let world = self.world;
        self.with_second_id(Second::get_id(world), func)
    }

    /// The function will be ran with the scope set to the current entity.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to call.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::scope`
    #[doc(alias = "entity_builder::scope")]
    pub fn run_in_scope(self, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_scope(self.world.world_ptr_mut(), *self.id);
            func();
            sys::ecs_set_scope(self.world.world_ptr_mut(), prev);
        }
        self
    }

    /// Calls the provided function with a world scoped to entity
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::scope`
    #[doc(alias = "entity_builder::scope")]
    pub fn scope(self, f: impl FnMut(&World)) -> Self {
        let world = &*self.world;
        world.scope_id(self.id, f);
        self
    }

    /// Gets mut component.
    ///
    /// This operation returns a mutable reference to the component. If the entity
    /// did not yet have the component, it will be added. If a base entity had
    /// the component, it will be overridden, and the value of the base component
    /// will be copied to the entity before this function returns.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component to get.
    ///
    /// # Returns
    ///
    /// A mutable ref to the component value.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::ensure`
    #[doc(alias = "entity::ensure")]
    #[allow(clippy::mut_from_ref)]
    pub fn ensure_mut<T: ComponentId + NotEmptyComponent + ComponentType<Struct>>(
        self,
    ) -> &'a mut T::UnderlyingType {
        let component_id = T::get_id(self.world);

        ecs_assert!(
            std::mem::size_of::<T>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<T>()
        );

        unsafe {
            &mut *(sys::ecs_ensure_id(self.world.world_ptr_mut(), *self.id, component_id)
                as *mut T::UnderlyingType)
        }
    }

    pub fn ensure_callback_mut<T: ComponentId + NotEmptyComponent + ComponentType<Struct>>(
        self,
        callback: impl FnOnce(&mut T::UnderlyingType),
    ) {
        let comp = self.ensure_mut::<T>();
        callback(comp);
    }

    /// Get mutable component value or pair (untyped).
    /// This operation returns a mutable pointer to the component. If the entity
    /// did not yet have the component, it will be added. If a base entity had
    /// the component, it will be overridden, and the value of the base component
    /// will be copied to the entity before this function returns.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the id is valid, not an enum, and not a tag, zero sized type.
    ///
    /// # Arguments
    ///
    /// * `comp`: The component to get.
    ///
    /// # Returns
    ///
    /// Pointer to the component value.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::ensure`
    #[doc(alias = "entity::ensure")]
    pub unsafe fn ensure_untyped_mut(self, id: impl IntoId) -> *mut c_void {
        unsafe {
            sys::ecs_ensure_id(self.world.world_ptr_mut(), *self.id, *id.into()) as *mut c_void
        }
    }

    /// Get a mutable reference for the first element of a pair
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::ensure`
    #[doc(alias = "entity::ensure")]
    pub fn ensure_first_id_mut<First>(self, second: impl Into<Entity>) -> &'a mut First
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let component_id = First::get_id(self.world);

        ecs_assert!(
            std::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<First>()
        );

        // SAFETY: The pointer is valid because sys::ecs_ensure_id adds the component if not present, so
        // it is guaranteed to be valid
        unsafe {
            &mut *(sys::ecs_ensure_id(
                self.world.world_ptr_mut(),
                *self.id,
                ecs_pair(component_id, *second.into()),
            ) as *mut First)
        }
    }

    /// Get a mutable reference for the first element of a pair
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    /// * `Second`: The second part of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::ensure`
    #[doc(alias = "entity::ensure")]
    pub fn ensure_first_mut<First, Second>(&mut self) -> &'a mut First
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId + ComponentType<Struct>,
    {
        self.ensure_first_id_mut::<First>(Second::get_id(self.world))
    }

    /// Get a mutable reference for the second element of a pair.
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::ensure`
    #[doc(alias = "entity::ensure")]
    pub fn ensure_second_id_mut<Second>(self, first: impl Into<Entity>) -> &'a mut Second
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let component_id = Second::get_id(self.world);

        ecs_assert!(
            std::mem::size_of::<Second>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<Second>()
        );

        // SAFETY: The pointer is valid because sys::ecs_ensure_id adds the component if not present, so
        // it is guaranteed to be valid
        unsafe {
            &mut *(sys::ecs_ensure_id(
                self.world.world_ptr_mut(),
                *self.id,
                ecs_pair(*first.into(), component_id),
            ) as *mut Second)
        }
    }

    /// Get a mutable reference for the second element of a pair.
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::ensure`
    #[doc(alias = "entity::ensure")]
    pub fn ensure_second_mut<First, Second>(&mut self) -> &'a mut Second
    where
        First: ComponentId + ComponentType<Struct> + EmptyComponent,
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        self.ensure_second_id_mut::<Second>(First::get_id(self.world))
    }

    /// Signal that component or pair was modified.
    ///
    /// # Arguments
    ///
    /// * `comp` - The component that was modified.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::modified`
    #[doc(alias = "entity::modified")]
    pub fn modified_id(self, id: impl IntoId) {
        unsafe { sys::ecs_modified_id(self.world.world_ptr_mut(), *self.id, *id.into()) }
    }

    /// Signal that component was modified.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the component that was modified.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::modified`
    #[doc(alias = "entity::modified")]
    pub fn modified<T: IntoComponentId>(&self) {
        ecs_assert!(
            std::mem::size_of::<T>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            T::name(),
        );
        self.modified_id(T::get_id(self.world));
    }

    /// Signal that the first part of a pair was modified.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first part of the pair.
    ///
    /// # Arguments
    ///
    /// * `second` - The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::modified`
    #[doc(alias = "entity::modified")]
    pub fn modified_first<First: ComponentId>(self, second: impl Into<Entity>) {
        ecs_assert!(
            std::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<First>()
        );

        self.modified_id((First::get_id(self.world), second.into()));
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
    /// # See also
    ///
    /// * C++ API: `entity::get_ref`
    #[doc(alias = "entity::get_ref")]
    pub fn get_ref<T>(&self) -> CachedRef<'a, T::UnderlyingType>
    where
        T: ComponentId + NotEmptyComponent,
        T::UnderlyingType: NotEmptyComponent,
    {
        CachedRef::<T::UnderlyingType>::new(self.world, *self.id, T::get_id(self.world))
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
    ///
    /// # See also
    ///
    /// * C++ API: `entity::get_ref`
    #[doc(alias = "entity::get_ref")]
    pub fn get_ref_first<First: ComponentId + NotEmptyComponent>(
        self,
        second: impl Into<Entity>,
    ) -> CachedRef<'a, First> {
        CachedRef::<First>::new(
            self.world,
            *self.id,
            ecs_pair(First::get_id(self.world), *second.into()),
        )
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
    /// # See also
    ///
    /// * C++ API: `entity::get_ref`
    #[doc(alias = "entity::get_ref")]
    pub fn get_ref_second<Second: ComponentId + NotEmptyComponent>(
        &self,
        first: impl Into<Entity>,
    ) -> CachedRef<Second> {
        CachedRef::<Second>::new(
            self.world,
            *self.id,
            ecs_pair(*first.into(), Second::get_id(self.world)),
        )
    }

    /// Clear an entity.
    ///
    /// This operation removes all components from an entity without recycling
    /// the entity id.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::clear`
    #[doc(alias = "entity::clear")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn clear(&self) {
        unsafe { sys::ecs_clear(self.world.world_ptr_mut(), *self.id) }
    }

    /// Delete an entity.
    ///
    /// Entities have to be deleted explicitly, and are not deleted when the
    /// entity object goes out of scope.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::destruct`
    #[doc(alias = "entity::destruct")]
    pub fn destruct(self) {
        unsafe { sys::ecs_delete(self.world.world_ptr_mut(), *self.id) }
    }
}
