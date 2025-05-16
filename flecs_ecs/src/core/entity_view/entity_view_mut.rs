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
    #[allow(clippy::should_implement_trait)]
    pub fn add<T: IntoId>(self, id: T) -> Self {
        let id = *id.into_id(self.world);
        let world = self.world.world_ptr_mut();

        if !T::IS_PAIR {
            if !T::IS_TYPED {
                check_add_id_validity(world, id);
            } else if !T::IF_ID_IS_DEFAULT && !<T as IntoId>::IS_TYPE_TAG {
                panic!("Default hook not implemented for non ZST type");
            }
        } else if T::IS_TYPED {
            if !T::IF_ID_IS_DEFAULT {
                if T::IS_TYPED_SECOND {
                    if !T::IF_ID_IS_DEFAULT_SECOND && !<T as IntoId>::IS_TYPE_TAG {
                        //for some reason const panic doesn't work here
                        panic!(
                            "none implement default, use `set_pair` instead to ensure valid data"
                        )
                    }
                } else {
                    check_add_id_validity(world, id);
                }
            }
        } else if T::IS_TYPED_SECOND {
            if !T::IF_ID_IS_DEFAULT_SECOND {
                check_add_id_validity(world, id);
            }
        } else {
            check_add_id_validity(world, id);
        }

        unsafe { sys::ecs_add_id(world, *self.id, id) }
        self
    }

    /// Adds an ID to the entity unchecked. Useful for run-time components.
    ///
    /// The provided `id` can represent various types, including a component, a pair, a tag, or another entity.
    ///
    /// # Safety
    /// Caller must ensure the `id` is a valid type.
    /// This function is unsafe because it does not check if the `id` is a valid type nor if
    /// the `id` implements a constructor hook if it is not a zero-sized type (ZST).
    /// if the id is a type without a constructor hook, it could cause you to read uninitialized data.
    /// the caller must ensure to initialize the component data before using it.
    ///
    /// # See Also
    ///
    /// * [`set_id`](Self::set_id)
    pub unsafe fn add_id_unchecked(self, id: impl IntoId) -> Self {
        let id = *id.into_id(self.world);
        let world = self.world.world_ptr_mut();

        unsafe { sys::ecs_add_id(world, *self.id, id) }
        self
    }

    /// Adds a flecs trait.
    pub fn add_trait<T>(self) -> Self
    where
        T: ComponentOrPairId,
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
        T: ComponentOrPairId,
    {
        let id = T::get_id(self.world);
        let world_ptr = self.world.world_ptr_mut();

        if unsafe { sys::ecs_get_target_for_id(world_ptr, *self.id, EcsIsA, id) } == 0 {
            panic!("Entity does not have the component to override");
        }
        unsafe { self.add_id_unchecked(id) }
    }

    /// Adds a pair to the entity composed of a tag and an (C) flecs enum constant.
    pub fn add_pair_enum<First, Second>(self, enum_value: Second) -> Self
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        const {
            if !First::IS_TAG && !First::IMPLS_DEFAULT {
                panic!(
                    "Adding an element that is not a Tag / Zero sized type requires to implement Default"
                );
            }
        }
        let world = self.world;
        let enum_id = enum_value.id_variant(world);
        unsafe { self.add_id_unchecked((First::id(world), enum_id)) }
    }

    /// Adds a pair to the entity where the first element is the enumeration type,
    /// and the second element is the enumeration constant.
    ///
    /// This function works only with regular (C style) enumerations.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The enumeration type, which derives from `ComponentId`, `ComponentType<Enum>`, and `EnumComponentInfo`.
    ///
    /// # Arguments
    ///
    /// - `enum_value`: The enumeration value.
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

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `component`: The component to add.
    pub fn add_if<T: IntoId>(self, id: T, condition: bool) -> Self {
        if condition {
            self.add(id)
        } else {
            // the compiler will optimize this branch away since it's known at compile time
            if T::IS_PAIR {
                // If second is 0 or if relationship is exclusive, use wildcard for
                // second which will remove all instances of the relationship.
                // Replacing 0 with Wildcard will make it possible to use the second
                // as the condition.
                let id = id.into_id(self.world);
                let first = id.get_id_first(self.world);
                let mut second = id.get_id_second(self.world);
                if second == 0
                    || unsafe { sys::ecs_has_id(self.world.world_ptr(), *first, ECS_EXCLUSIVE) }
                {
                    second = ECS_WILDCARD.into();
                }
                self.remove((first, second))
            } else {
                self.remove(id)
            }
        }
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
    pub fn add_enum_if<T>(self, enum_value: T, condition: bool) -> Self
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let world = self.world;
        // SAFETY: we know that the enum_value is a valid because of the T::id call
        self.add_if(
            (T::id(world), unsafe {
                enum_value.id_variant_unchecked(world)
            }),
            condition,
        )
    }

    /// Remove an entity from an entity.
    ///
    /// # Arguments
    ///
    /// * `component_id`: The entity to remove.
    pub fn remove<T: IntoId>(self, id: T) -> Self {
        let id = *id.into_id(self.world);
        let id = if <T as IntoId>::IS_ENUM {
            ecs_pair(id, ECS_WILDCARD)
        } else {
            id
        };

        unsafe { sys::ecs_remove_id(self.world.world_ptr_mut(), *self.id, id) }

        self
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
    pub fn remove_enum_tag<First, Second>(self, enum_value: Second) -> Self
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let world = self.world;
        self.remove((First::id(world), enum_value.id_variant(world)))
    }

    /// Shortcut for `add((flecs::IsA, id))`.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    pub fn is_a(self, second: impl IntoEntity) -> Self {
        unsafe { self.add_id_unchecked((ECS_IS_A, second.into_entity(self.world))) }
    }

    /// Shortcut for `add_id((flecs::ChildOf::ID, entity))`.
    ///
    /// # Arguments
    ///
    /// * `parent`: The parent entity to establish the relationship with.
    pub fn child_of(self, parent: impl IntoEntity) -> Self {
        unsafe { self.add_id_unchecked((ECS_CHILD_OF, parent.into_entity(self.world))) }
    }

    /// Shortcut for `add_id((flecs::DependsOn::ID, entity))`.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    pub fn depends_on(self, second: impl IntoEntity) -> Self {
        unsafe { self.add_id_unchecked((ECS_DEPENDS_ON, second.into_entity(self.world))) }
    }

    /// Shortcut for `add_id((flecs::Dependency::ID, entity))`for Enums.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum constant.
    pub fn depends_on_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        self,
        enum_value: T,
    ) -> Self {
        let world = self.world;
        self.depends_on(enum_value.id_variant(world))
    }

    /// Shortcut for `add_id((flecs::SlotOf::ID, entity))`.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    pub fn slot_of(self, second: impl IntoEntity) -> Self {
        unsafe { self.add_id_unchecked((ECS_SLOT_OF, second.into_entity(self.world))) }
    }

    /// Shortcut for `add_id((flecs::SlotOf::ID, target(ChildOf)))`.
    pub fn slot(self) -> Self {
        ecs_assert!(
            self.target(flecs::ChildOf::ID, 0).is_some(),
            FlecsErrorCode::InvalidParameter,
            "add ChildOf pair before using slot()"
        );
        let id = self.target(ECS_CHILD_OF, 0);
        self.slot_of(id.expect("ChildOf pair not found"))
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
    pub fn auto_override(self, id: impl IntoId) -> Self {
        unsafe { self.add_id_unchecked(ECS_AUTO_OVERRIDE | id.into_id(self.world)) }
    }

    /// Sets a component mark override for the entity and sets the component data.
    pub fn set_auto_override<T: ComponentId + DataComponent + ComponentType<Struct>>(
        self,
        component: T,
    ) -> Self {
        self.auto_override(id::<T>()).set(component)
    }

    /// Sets a pair, mark component for auto-overriding.
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
        self.auto_override(id_pair).set_id(data, id_pair)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `First` and `second` pair id data type is the one provided.
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
        self.auto_override(pair_id).set_id(first, pair_id)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `Sirst` and `fecond` pair id data type is the one provided.
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
        self.auto_override(pair_id).set_id(second, pair_id)
    }

    /// Sets a component of type `T` on the entity.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to set on the entity.
    pub fn set<T: ComponentId + DataComponent>(self, component: T) -> Self {
        set_helper(
            self.world.world_ptr_mut(),
            *self.id,
            component,
            T::id(self.world),
        );
        self
    }

    /// Sets the data of the specified id. Can be a pair or Component.
    ///
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
    /// * [`EntityView::set`]
    /// * [`EntityView::set_pair`]
    pub fn set_id<T>(self, data: T, id: impl IntoId) -> Self
    where
        T: ComponentId + DataComponent,
    {
        let world = self.world.world_ptr_mut();
        let id = *id.into_id(self.world);
        let data_id = T::id(self.world);
        let id_data_id = unsafe { sys::ecs_get_typeid(world, id) };

        if data_id != id_data_id {
            panic!(
                "Data type does not match id type. For pairs this is the first element occurrence that is not a zero-sized type (ZST)."
            );
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
            assert!(
                !<(First, Second) as ComponentOrPairId>::IS_TAGS,
                "setting tag relationships is not possible with `set_pair`. use `add::<(Tag1, Tag2)()` instead."
            );
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

    /// Set a pair for an entity using the first element type and a second component ID.
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
            panic!(
                "First type does not match id data type. For pairs this is the first element occurrence that is not a zero-sized type (ZST)."
            );
        }

        set_helper(world_ptr, *self.id, first, pair_id);
        self
    }

    /// Set a pair for an entity using the second element type and a first id.
    ///
    /// # Panics
    ///
    /// Caller must ensure that first is a zero-sized type (ZST) or entity and not a pair.
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
            panic!(
                "Second type does not match id data type. For pairs this is the first element occurrence that is not a zero-sized type (ZST)."
            );
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
    pub unsafe fn set_ptr_w_size(
        self,
        id: impl Into<Entity>,
        size: usize,
        ptr: *const c_void,
    ) -> Self {
        unsafe {
            sys::ecs_set_id(self.world.world_ptr_mut(), *self.id, *id.into(), size, ptr);
            self
        }
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
    pub unsafe fn set_ptr(self, id: impl Into<Entity>, ptr: *const c_void) -> Self {
        unsafe {
            let id = id.into();
            let cptr: *const sys::EcsComponent = sys::ecs_get_id(
                self.world.world_ptr_mut(),
                *id,
                sys::FLECS_IDEcsComponentID_,
            ) as *const sys::EcsComponent;

            ecs_assert!(
                !cptr.is_null(),
                FlecsErrorCode::InvalidParameter,
                "invalid component id: {:?}",
                id
            );

            self.set_ptr_w_size(id, (*cptr).size as usize, ptr)
        }
    }

    /// Sets the name of the entity.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to be set.
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
            sys::ecs_set_name(self.world.world_ptr_mut(), *self.id, core::ptr::null());
        }
        self
    }

    /// Sets the alias name of the entity.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the alias name to be set.
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
    pub fn enable(self, id: impl IntoId) -> Self {
        unsafe {
            sys::ecs_enable_id(
                self.world.world_ptr_mut(),
                *self.id,
                *id.into_id(self.world),
                true,
            );
        }
        self
    }

    /// Disables self (entity).
    ///
    /// Disabled entities are not matched with systems and cannot be searched with queries,
    /// unless explicitly specified in the query expression.
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
    pub fn disable(self, id: impl IntoId) -> Self {
        unsafe {
            sys::ecs_enable_id(
                self.world.world_ptr_mut(),
                *self.id,
                *id.into_id(self.world),
                false,
            );
        }
        self
    }

    /// Entities created in the function will have the current entity.
    /// This operation is thread safe.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to call.
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
    pub fn with_first(self, first: impl IntoEntity, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_with(
                self.world.world_ptr_mut(),
                ecs_pair(*first.into_entity(self.world), *self.id),
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
    pub fn with_second(self, second: impl IntoEntity, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_with(
                self.world.world_ptr_mut(),
                ecs_pair(*self.id, *second.into_entity(self.world)),
            );
            func();
            sys::ecs_set_with(self.world.world_ptr_mut(), prev);
        }
        self
    }

    /// The function will be ran with the scope set to the current entity.
    ///
    /// # Arguments
    ///
    /// - `func`: The function to call.
    pub fn run_in_scope(self, func: impl FnOnce()) -> Self {
        unsafe {
            let prev = sys::ecs_set_scope(self.world.world_ptr_mut(), *self.id);
            func();
            sys::ecs_set_scope(self.world.world_ptr_mut(), prev);
        }
        self
    }

    /// Calls the provided function with a world scoped to entity
    pub fn scope(self, f: impl FnMut(&World)) -> Self {
        let world = &*self.world;
        world.scope(self.id, f);
        self
    }

    /// Signal that component or pair was modified.
    ///
    /// # Arguments
    ///
    /// * `comp` - The component that was modified.
    ///
    /// # See also
    ///
    /// * [`EntityView::modified()`]
    /// * [`World::modified()`]
    pub fn modified<T: IntoId>(self, id: T) {
        const {
            if <T as IntoId>::IS_TYPE_TAG {
                panic!("Cannot modify tag component");
            }
        }

        unsafe {
            sys::ecs_modified_id(
                self.world.world_ptr_mut(),
                *self.id,
                *id.into_id(self.world),
            );
        }
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
    /// let derived = world.component::<Derived>().is_a(base);
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
    //TODO: can this be shrunk to just one function like with add,add_id
    pub fn get_ref_w_id<T>(&self, component: impl IntoId) -> CachedRef<'a, T::CastType>
    where
        T: ComponentOrPairId,
        T::CastType: DataComponent,
    {
        CachedRef::<T::CastType>::new(self.world, *self.id, *component.into_id(self.world))
    }

    /// Get a reference to a component or pair.
    ///
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling `get` for the same component.
    ///
    /// - `T`: Component for which to get a reference.
    ///
    /// Returns: The reference component.
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

    /// Clear an entity.
    ///
    /// This operation removes all components from an entity without recycling
    /// the entity id.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn clear(&self) {
        unsafe { sys::ecs_clear(self.world.world_ptr_mut(), *self.id) }
    }

    /// Delete an entity.
    ///
    /// Entities have to be deleted explicitly, and are not deleted when the
    /// entity object goes out of scope.
    pub fn destruct(self) {
        unsafe { sys::ecs_delete(self.world.world_ptr_mut(), *self.id) }
    }
}
