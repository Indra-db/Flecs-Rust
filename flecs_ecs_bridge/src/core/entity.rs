use std::{ops::Deref, os::raw::c_void};

use crate::{
    core::{
        c_binding::bindings::{ecs_get_target, ecs_set_id, ecs_set_name, ECS_OVERRIDE},
        utility::errors::FlecsErrorCode,
    },
    ecs_assert,
};

use super::{
    c_binding::bindings::{
        ecs_add_id, ecs_clear, ecs_delete, ecs_enable, ecs_enable_id, ecs_entity_desc_t,
        ecs_entity_init, ecs_flatten, ecs_flatten_desc_t, ecs_get_id, ecs_get_mut_id,
        ecs_get_world, ecs_has_id, ecs_modified_id, ecs_new_w_id, ecs_remove_id, ecs_set_alias,
        ecs_set_scope, ecs_set_with, EcsChildOf, EcsComponent, EcsDependsOn, EcsExclusive, EcsIsA,
        EcsSlotOf, EcsWildcard, FLECS__EEcsComponent,
    },
    c_types::{EntityT, IdT, WorldT, SEPARATOR},
    component::{CachedComponentData, ComponentType, Enum, Struct},
    component_ref::Ref,
    entity_view::EntityView,
    enum_type::CachedEnumData,
    id::Id,
    utility::functions::ecs_pair,
    utility::macros::*,
};

pub struct Entity {
    entity_view: EntityView,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            entity_view: EntityView::default(),
        }
    }
}

impl Deref for Entity {
    type Target = EntityView;

    fn deref(&self) -> &Self::Target {
        &self.entity_view
    }
}

impl From<Entity> for u64 {
    fn from(entity: Entity) -> Self {
        entity.raw_id
    }
}

// functions in here match most of the functions in the c++ entity and entity_builder class
impl Entity {
    /// Create new entity.
    /// ### Safety
    /// This function is unsafe because it assumes that the world is not null.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(world: *mut WorldT) -> Self {
        Self {
            entity_view: EntityView::new_from_existing(world, unsafe { ecs_new_w_id(world, 0) }),
        }
    }

    /// Wrap an existing entity id.
    /// # Arguments
    /// * `world` - The world the entity belongs to.
    /// * `id` - The entity id.
    pub fn new_from_existing(world: *mut WorldT, id: IdT) -> Self {
        Self {
            entity_view: EntityView::new_from_existing(world, id),
        }
    }

    // Explicit conversion from flecs::entity_t to Entity
    pub const fn new_only_id(id: EntityT) -> Self {
        Self {
            entity_view: EntityView::new_only_id(id),
        }
    }

    /// Create a named entity.
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "Foo::Bar". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Parameters
    ///
    /// - `world`: The world in which to create the entity.
    /// - `name`: The entity name.
    pub fn new_named(world: *mut WorldT, name: &str) -> Self {
        let c_name = std::ffi::CString::new(name).expect("Failed to convert to CString");

        let desc = ecs_entity_desc_t {
            name: c_name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            symbol: std::ptr::null(),
            use_low_id: false,
            add: [0; 32],
            add_expr: std::ptr::null(),
        };
        let id = unsafe { ecs_entity_init(world, &desc) };
        Self {
            entity_view: EntityView::new_from_existing(world, id),
        }
    }

    /// Add an entity to an entity.
    ///
    /// Add an entity to the entity. This is typically used for tagging.
    ///
    /// # Parameters
    ///
    /// - `component_id`: The component to add.
    pub fn add_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_add_id(self.world, self.raw_id, component_id) }
        self
    }

    /// Add a component to an entity.
    ///
    /// To ensure the component is initialized, it should have a constructor.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to add.
    pub fn add_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.add_component_id(T::get_id(world))
    }

    /// Add a pair to an entity.
    ///
    /// This operation adds a pair to the entity.
    ///
    /// # Parameters
    ///
    /// - `first`: The first element of the pair.
    /// - `second`: The second element of the pair.
    pub fn add_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        let world = self.world;
        self.add_component_id(ecs_pair(first, second))
    }

    /// Add a pair.
    /// This operation adds a pair to the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair
    /// * `Second`: The second element of the pair
    pub fn add_pair<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let world = self.world;
        self.add_pair_ids(First::get_id(world), Second::get_id(world))
    }

    /// Adds a pair to the entity
    ///
    /// # Type Parameters
    ///
    /// * `Second` - the second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first` - the first element of the pair
    pub fn add_pair_first_id<Second: CachedComponentData>(self, first: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(first, Second::get_id(world))
    }

    /// Adds a pair to the entity
    ///
    /// # Type Parameters
    ///
    /// * `First` - the first element of the pair
    ///
    /// # Arguments
    ///
    /// * `second` - the second element of the pair
    pub fn add_pair_second_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(First::get_id(world), second)
    }

    /// Adds a pair to the entity composed of a tag and an enum constant.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The tag (first element of the pair).
    /// - `U`: The enum constant (second element of the pair).
    ///
    /// # Parameters
    ///
    /// - `enum_value`: The enum constant.
    pub fn add_enum_tag<First, Second>(self, enum_value: Second) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        self.add_pair_ids(
            First::get_id(world),
            enum_value.get_entity_id_from_enum_field(world),
        )
    }

    /// Adds a pair to the entity where the first element is the enumeration type,
    /// and the second element is the enumeration constant.
    ///
    /// This function works with regular (C style) enumerations as well as enum classes.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The enumeration type, which derives from `CachedComponentData`, `ComponentType<Enum>`, and `CachedEnumData`.
    ///
    /// # Parameters
    ///
    /// - `enum_value`: The enumeration value.
    pub fn add_enum_constant<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        self,
        enum_value: T,
    ) -> Self {
        let world = self.world;
        self.add_pair_ids(
            T::get_id(world),
            enum_value.get_entity_id_from_enum_field(world),
        )
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Parameters
    ///
    /// * `condition`: The condition to evaluate.
    /// * `component`: The component to add.
    pub fn add_component_id_if(self, component_id: IdT, condition: bool) -> Self {
        if condition {
            let world = self.world;
            return self.add_component_id(component_id);
        }

        self
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component to add.
    ///
    /// # Parameters
    ///
    /// * `condition`: The condition to evaluate.
    pub fn add_component_if<T: CachedComponentData>(self, condition: bool) -> Self {
        let world = self.world;
        self.add_component_id_if(T::get_id(world), condition)
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Parameters
    ///
    /// * `condition`: The condition to evaluate.
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    pub fn add_pair_ids_if(self, first: EntityT, mut second: EntityT, condition: bool) -> Self {
        let world = self.world;
        if condition {
            self.add_pair_ids(first, second)
        } else {
            // If second is 0 or if relationship is exclusive, use wildcard for
            // second which will remove all instances of the relationship.
            // Replacing 0 with Wildcard will make it possible to use the second
            // as the condition.
            if second == 0 || unsafe { ecs_has_id(self.world, first, EcsExclusive) } {
                second = unsafe { EcsWildcard }
            }
            self.remove_pair_ids(first, second)
        }
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair
    /// * `Second`: The second element of the pair
    ///
    /// # Parameters
    ///
    /// * `condition`: The condition to evaluate.
    pub fn add_pair_if<First, Second>(self, condition: bool) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let world = self.world;
        self.add_pair_ids_if(First::get_id(world), Second::get_id(world), condition)
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair
    ///
    /// # Parameters
    ///
    /// * `condition`: The condition to evaluate.
    /// * `first`: The first element of the pair.
    pub fn add_pair_first_id_if<Second: CachedComponentData>(
        self,
        first: EntityT,
        condition: bool,
    ) -> Self {
        let world = self.world;
        self.add_pair_ids_if(first, Second::get_id(world), condition)
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair
    ///
    /// # Parameters
    ///
    /// * `condition`: The condition to evaluate.
    /// * `second`: The second element of the pair.
    pub fn add_pair_second_id_if<First: CachedComponentData>(
        self,
        second: EntityT,
        condition: bool,
    ) -> Self {
        let world = self.world;
        self.add_pair_ids_if(First::get_id(world), second, condition)
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Type Parameters
    ///
    /// * `T`: enum type
    ///
    /// # Parameters
    ///
    /// * `condition`: The condition to evaluate.
    /// * `enum_value`: The enumeration constant.
    pub fn add_enum_tag_if<T>(self, enum_value: T, condition: bool) -> Self
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        self.add_pair_ids_if(
            T::get_id(world),
            enum_value.get_entity_id_from_enum_field(world),
            condition,
        )
    }

    /// Remove an entity from an entity.
    ///
    /// # Parameters
    ///
    /// * `component_id`: The entity to remove.
    pub fn remove_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_remove_id(self.world, self.raw_id, component_id) }
        self
    }

    /// Remove a component from an entity.
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type of the component to remove.
    pub fn remove_component<T: CachedComponentData + ComponentType<Struct>>(self) -> Self {
        let world = self.world;
        self.remove_component_id(T::get_id(world))
    }

    /// Remove pair for enum
    /// This operation will remove any (Enum, *) pair from the entity.
    ///
    /// # Type parameters
    /// * `T` - The enum type.
    pub fn remove_component_enum<T: CachedComponentData + ComponentType<Enum>>(self) -> Self {
        let world = self.world;
        self.remove_pair_ids(T::get_id(world), unsafe { EcsWildcard })
    }

    /// Remove a pair.
    /// This operation removes a pair from the entity.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    pub fn remove_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        let world = self.world;
        self.remove_component_id(ecs_pair(first, second))
    }

    /// Removes a pair.
    /// This operation removes a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The type of the first element of the pair.
    /// * `Second`: The type of the second element of the pair.
    pub fn remove_pair<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let world = self.world;
        self.remove_pair_ids(First::get_id(world), Second::get_id(world))
    }

    /// Remove a pair.
    /// This operation removes a pair to the entity.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the first element of the pair.
    /// * `U`: The type of the second element of the pair.
    ///
    /// # Parameters
    ///
    /// * `enum_value`: the enum constant.
    pub fn remove_enum_tag<First, Second>(self, enum_value: Second) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        self.remove_pair_ids(
            First::get_id(world),
            enum_value.get_entity_id_from_enum_field(world),
        )
    }

    /// Removes a pair.
    /// This operation removes a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    pub fn remove_pair_first_id<Second: CachedComponentData>(self, first: EntityT) -> Self {
        let world = self.world;
        self.remove_pair_ids(first, Second::get_id(world))
    }

    /// Shortcut for add(IsA, entity).
    ///
    /// # Parameters
    ///
    /// * `second`: The second element of the pair.
    pub fn is_a_id(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(unsafe { EcsIsA }, second)
    }

    /// Shortcut for add(IsA, entity).
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type associated with the entity.
    pub fn is_a<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.is_a_id(T::get_id(world))
    }

    /// Shortcut for add(ChildOf, entity).
    ///
    /// # Parameters
    ///
    /// * `second`: The second element of the pair.
    pub fn child_of_id(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(unsafe { EcsChildOf }, second)
    }

    /// Shortcut for add(ChildOf, entity).
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type associated with the entity.
    pub fn child_of<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.child_of_id(T::get_id(world))
    }

    /// Shortcut for add(DependsOn, entity).
    ///
    /// # Parameters
    ///
    /// * `second`: The second element of the pair.
    pub fn depends_on_id(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(unsafe { EcsDependsOn }, second)
    }

    /// Shortcut for add(DependsOn, entity).
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type associated with the entity.
    pub fn depends_on<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.depends_on_id(T::get_id(world))
    }

    /// Shortcut for add(SlotOf, entity).
    ///
    /// # Parameters
    ///
    /// * `second`: The second element of the pair.
    pub fn slot_of_id(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(unsafe { EcsSlotOf }, second)
    }

    /// Shortcut for add(SlotOf, entity).
    ///
    /// # Type Parameters
    ///
    /// * `T`: the type associated with the entity.
    pub fn slot_of<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.slot_of_id(T::get_id(world))
    }

    /// Shortcut for add(SlotOf, target(ChildOf)).
    pub fn slot_child(self) -> Self {
        ecs_assert!(
            unsafe { ecs_get_target(self.world, self.raw_id, EcsChildOf, 0) } != 0,
            FlecsErrorCode::InvalidParameter,
            "add ChildOf pair before using slot()"
        );
        let id: u64 = self.get_target_from_entity(unsafe { EcsChildOf }, 0).raw_id;
        self.slot_of_id(id)
    }

    /// Mark id for auto-overriding.
    ///
    /// When an entity inherits from a base entity (using the IsA relationship)
    /// any ids marked for auto-overriding on the base will be overridden
    /// automatically by the entity.
    ///
    /// # Parameters
    ///
    /// * `id`: The id to mark for overriding.
    pub fn mark_component_id_for_override(self, id: IdT) -> Self {
        let world = self.world;
        self.add_component_id(unsafe { ECS_OVERRIDE | id })
    }

    /// Mark component for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component to mark for overriding.
    pub fn mark_component_for_override<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.mark_component_id_for_override(T::get_id(world))
    }

    /// Mark pair for auto-overriding.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    pub fn mark_pair_ids_for_override(self, first: EntityT, second: EntityT) -> Self {
        let world = self.world;
        self.mark_component_id_for_override(ecs_pair(first, second))
    }

    /// Mark pair for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    pub fn mark_pair_for_override<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        let world = self.world;
        self.mark_pair_ids_for_override(First::get_id(world), Second::get_id(world))
    }

    /// Mark pair for auto-overriding with a given first ID.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    pub fn mark_pair_for_override_with_first_id<Second: CachedComponentData>(
        self,
        first: EntityT,
    ) -> Self {
        let world = self.world;
        self.mark_pair_ids_for_override(first, Second::get_id(world))
    }

    /// Mark pair for auto-overriding with a given second ID.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Parameters
    ///
    /// * `second`: The second element of the pair.
    pub fn mark_pair_for_override_with_second_id<First: CachedComponentData>(
        self,
        second: EntityT,
    ) -> Self {
        let world = self.world;
        self.mark_pair_ids_for_override(First::get_id(world), second)
    }

    /// Sets a component of type `T` on the entity.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to set on the entity.
    pub fn set_component<T: CachedComponentData>(self, component: T) -> Self {
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(raw_id, component, T::get_id(world))
    }

    /// Set a pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair to be set.
    pub fn set_pair_first<First, Second>(self, first: First) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(
            raw_id,
            first,
            ecs_pair(First::get_id(world), Second::get_id(world)),
        )
    }

    /// Set a pair for an entity using the first element type and a second id.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair. Must implement `CachedComponentData`.
    ///
    /// # Parameters
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    pub fn set_pair_second_id<First: CachedComponentData>(
        self,
        first: First,
        second: EntityT,
    ) -> Self {
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(raw_id, first, ecs_pair(First::get_id(world), second))
    }

    /// Set a pair for an entity.
    /// This operation sets the pair value, and uses Second as type. If the
    /// entity did not yet have the pair, it will be added.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    /// * `value`: The value to set.
    pub fn set_pair_second<First, Second>(self, second: Second) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(
            raw_id,
            second,
            ecs_pair(First::get_id(world), Second::get_id(world)),
        )
    }

    /// Set a pair for an entity using the second element type and a first component ID.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair. Must implement `CachedComponentData`.
    ///
    /// # Parameters
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    pub fn set_pair_first_id<Second: CachedComponentData>(
        self,
        first: EntityT,
        second: Second,
    ) -> Self {
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(raw_id, second, ecs_pair(first, Second::get_id(world)))
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
    /// # Parameters
    ///
    /// * `constant`: The enum constant.
    /// * `value`: The value to set.
    pub fn set_enum_pair_first<First, Second>(self, first: First, constant: Second) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        //not sure if this is correct
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(
            raw_id,
            first,
            ecs_pair(
                First::get_id(world),
                constant.get_entity_id_from_enum_field(world),
            ),
        )
    }

    /// Internal helper function to set a component for an entity.
    ///
    /// This function sets the given value for an entity in the ECS world, ensuring
    /// that the type of the component is valid.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the component data. Must implement `CachedComponentData`.
    ///
    /// # Parameters
    ///
    /// * `entity`: The ID of the entity.
    /// * `value`: The value to set for the component.
    /// * `id`: The ID of the component type.
    fn set_helper<T: CachedComponentData>(self, entity: EntityT, value: T, id: IdT) -> Self {
        ecs_assert!(
            T::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            T::get_symbol_name()
        );

        let comp = unsafe { ecs_get_mut_id(self.world, self.raw_id, id) as *mut T };
        unsafe {
            *comp = value;
            ecs_modified_id(self.world, entity, id)
        };
        self
    }

    /// Sets a component for an entity and marks it as overridden.
    ///
    /// This function sets a component for an entity and marks the component
    /// as overridden, meaning that it will not be updated by systems that
    /// typically update this component.
    ///
    /// # Parameters
    ///
    /// * `component_id`: The ID of the component to set and mark as overridden.
    pub fn set_component_id_mark_override(self, component_id: IdT) -> Self {
        unsafe { ecs_add_id(self.world, self.raw_id, ECS_OVERRIDE | component_id) }
        self
    }

    /// Sets a component mark override for the entity and sets the component data.
    ///
    /// # Arguments
    ///
    /// * `component` - The component data to set.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the component data.
    ///
    /// # Returns
    ///
    /// The modified entity.
    pub fn set_component_mark_override<T: CachedComponentData>(self, component: T) -> Self {
        self.mark_component_for_override::<T>()
            .set_component(component)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The type of the first element of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The ID of the second element of the pair.
    pub fn set_pair_first_override<First: CachedComponentData + ComponentType<Struct>>(
        self,
        first: First,
        second: EntityT,
    ) -> Self {
        let world = self.world;
        self.mark_pair_for_override_with_second_id::<First>(second)
            .set_pair_second_id(first, second)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The type of the second element of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The ID of the second element of the pair.
    /// * `second`: The first element of the pair.
    pub fn set_pair_second_override<Second: CachedComponentData + ComponentType<Struct>>(
        self,
        second: Second,
        first: EntityT,
    ) -> Self {
        let world = self.world;
        self.mark_pair_for_override_with_first_id::<Second>(first)
            .set_pair_first_id(first, second)
    }

    /// Sets a pointer to a component of an entity with a given component ID and size.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the entity.
    /// * `component_id` - The ID of the component to set the pointer to.
    /// * `size` - The size of the component.
    /// * `ptr` - A pointer to the component.
    pub fn set_ptr_w_size(self, component_id: EntityT, size: usize, ptr: *const c_void) -> Self {
        unsafe { ecs_set_id(self.world, self.raw_id, component_id, size, ptr) };
        self
    }

    /// Sets a pointer to a component of an entity with a given component ID.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the entity.
    /// * `component_id` - The ID of the component to set the pointer to.
    /// * `ptr` - A pointer to the component.
    pub fn set_ptr(self, component_id: EntityT, ptr: *const c_void) -> Self {
        let cptr: *const EcsComponent =
            unsafe { ecs_get_id(self.world, component_id, FLECS__EEcsComponent) }
                as *const EcsComponent;

        ecs_assert!(
            !cptr.is_null(),
            FlecsErrorCode::InvalidParameter,
            "invalid component id: {:?}",
            component_id
        );

        self.set_ptr_w_size(component_id, unsafe { (*cptr).size } as usize, ptr)
    }

    /// Sets the name of the entity.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to be set.
    pub fn set_name(self, name: &str) -> Self {
        let c_name = std::ffi::CString::new(name).expect("Failed to convert to CString");
        unsafe {
            ecs_set_name(self.world, self.raw_id, c_name.as_ptr());
        }
        self
    }

    /// Sets the alias name of the entity.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the alias name to be set.
    pub fn set_alias_name(self, name: &str) -> Self {
        let c_name = std::ffi::CString::new(name).expect("Failed to convert to CString");
        unsafe {
            ecs_set_alias(self.world, self.raw_id, c_name.as_ptr());
        }
        self
    }

    /// Enables an entity.
    ///
    /// Enabled entities are matched with systems and can be searched with queries.
    pub fn enable(self) -> Self {
        unsafe { ecs_enable(self.world, self.raw_id, true) }
        self
    }
    /// Enables an ID.
    ///
    /// This sets the enabled bit for this component. If this is the first time the component is
    /// enabled or disabled, the bitset is added.
    ///
    /// # Parameters
    ///
    /// - `component_id`: The ID to enable.
    /// - `toggle`: True to enable, false to disable (default = true).
    pub fn enable_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_enable_id(self.world, self.raw_id, component_id, true) }
        self
    }

    /// Enables a component.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component to enable.
    pub fn enable_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.enable_component_id(T::get_id(world))
    }

    /// Enables a pair using IDs.
    ///
    /// # Parameters
    ///
    /// - `first`: The first element of the pair.
    /// - `second`: The second element of the pair.
    pub fn enable_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        self.enable_component_id(ecs_pair(first, second))
    }

    /// Enables a pair.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The first element of the pair.
    /// - `U`: The second element of the pair.
    pub fn enable_pair<First, Second>(self) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData,
    {
        let world = self.world;
        self.enable_pair_ids(T::get_id(world), U::get_id(world))
    }

    /// Enables a pair with a specific ID for the second element.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The first element of the pair.
    ///
    /// # Parameters
    ///
    /// - `second`: The ID of the second element of the pair.
    pub fn enable_pair_with_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.enable_pair_ids(First::get_id(world), second)
    }

    /// Disables an entity.
    ///
    /// Disabled entities are not matched with systems and cannot be searched with queries,
    /// unless explicitly specified in the query expression.
    pub fn disable(self) -> Self {
        unsafe { ecs_enable(self.world, self.raw_id, false) }
        self
    }

    /// Disables an ID.
    ///
    /// This sets the enabled bit for this ID. If this is the first time the ID is
    /// enabled or disabled, the bitset is added.
    ///
    /// # Parameters
    ///
    /// - `component_id`: The ID to disable.
    pub fn disable_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_enable_id(self.world, self.raw_id, component_id, false) }
        self
    }

    /// Disables a component.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component to disable.
    pub fn disable_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.disable_component_id(T::get_id(world))
    }

    /// Disables a pair using IDs.
    ///
    /// # Parameters
    ///
    /// - `first`: The first element of the pair.
    /// - `second`: The second element of the pair.
    pub fn disable_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        self.disable_component_id(ecs_pair(first, second))
    }

    /// Disables a pair.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The first element of the pair.
    /// - `U`: The second element of the pair.
    pub fn disable_pair<First, Second>(self) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData,
    {
        let world = self.world;
        self.disable_pair_ids(T::get_id(world), U::get_id(world))
    }

    /// Disables a pair with a specific ID for the second element.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The first element of the pair.
    ///
    /// # Parameters
    ///
    /// - `second`: The ID of the second element of the pair.
    pub fn disable_pair_with_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.disable_pair_ids(First::get_id(world), second)
    }
    /// Entities created in the function will have the current entity.
    ///
    /// # Parameters
    ///
    /// - `func`: The function to call.
    pub fn with<F>(&self, func: F) -> &Self
    where
        F: FnOnce(),
    {
        unsafe {
            let prev = ecs_set_with(self.world, self.raw_id);
            func();
            ecs_set_with(self.world, prev);
        }
        self
    }

    /// Entities created in the function will have a pair consisting of a specified ID and the current entity.
    ///
    /// # Parameters
    ///
    /// - `first`: The first element of the pair.
    /// - `func`: The function to call.
    pub fn with_pair_first_id<F>(&self, first: EntityT, func: F) -> &Self
    where
        F: FnOnce(),
    {
        unsafe {
            let prev = ecs_set_with(self.world, ecs_pair(first, self.raw_id));
            func();
            ecs_set_with(self.world, prev);
        }
        self
    }

    /// Entities created in the function will have a pair consisting of the current entity and a specified ID.
    ///
    /// # Parameters
    ///
    /// - `second`: The second element of the pair.
    /// - `func`: The function to call.
    pub fn with_pair_second_id<F>(&self, second: EntityT, func: F) -> &Self
    where
        F: FnOnce(),
    {
        unsafe {
            let prev = ecs_set_with(self.world, ecs_pair(self.raw_id, second));
            func();
            ecs_set_with(self.world, prev);
        }
        self
    }

    /// Entities created in the function will have a pair consisting of a specified component and the current entity.
    ///
    /// # Type Parameters
    ///
    /// - `First`: The first element of the pair.
    ///
    /// # Parameters
    ///
    /// - `func`: The function to call.
    pub fn with_pair_first<First: CachedComponentData, F>(&self, func: F) -> &Self
    where
        F: FnOnce(),
    {
        let world = self.world;
        self.with_pair_first_id(First::get_id(world), func)
    }

    /// Entities created in the function will have a pair consisting of the current entity and a specified component.
    ///
    /// # Type Parameters
    ///
    /// - `Second`: The second element of the pair.
    ///
    /// # Parameters
    ///
    /// - `func`: The function to call.
    pub fn with_pair_second<Second: CachedComponentData, F>(&self, func: F) -> &Self
    where
        F: FnOnce(),
    {
        let world = self.world;
        self.with_pair_second_id(Second::get_id(world), func)
    }

    /// The function will be ran with the scope set to the current entity.
    ///
    /// # Parameters
    ///
    /// - `func`: The function to call.
    pub fn scope<F>(&self, func: F) -> &Self
    where
        F: FnOnce(),
    {
        unsafe {
            let prev = ecs_set_scope(self.world, self.raw_id);
            func();
            ecs_set_scope(self.world, prev);
        }
        self
    }

    /// Gets a mutable pointer to a component value.
    ///
    /// This operation returns a mutable pointer to the component. If the entity
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
    /// A mutable pointer to the component value.
    pub fn get_component_mut<T: CachedComponentData + ComponentType<Struct>>(&self) -> *mut T {
        let component_id = T::get_id(self.world);
        ecs_assert!(
            T::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            T::get_symbol_name()
        );
        unsafe { ecs_get_mut_id(self.world, self.raw_id, component_id) as *mut T }
    }

    /// Get mut enum constant
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The enum component type which to get the constant
    ///
    /// ### Returns
    ///
    /// * `*mut T` - The enum component, nullptr if the entity does not have the component
    pub fn get_enum_component_mut<T: CachedComponentData + ComponentType<Enum>>(&self) -> *mut T {
        let component_id: IdT = T::get_id(self.world);
        let target: IdT = unsafe { ecs_get_target(self.world, self.raw_id, component_id, 0) };

        if target == 0 {
            // if there is no matching pair for (r,*), try just r
            unsafe { ecs_get_mut_id(self.world, self.raw_id, component_id) as *mut T }
        } else {
            // get constant value from constant entity
            let constant_value =
                unsafe { ecs_get_mut_id(self.world, target, component_id) as *mut T };

            ecs_assert!(
                !constant_value.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );

            constant_value
        }
    }

    /// Get mutable component value (untyped).
    /// This operation returns a mutable pointer to the component. If the entity
    /// did not yet have the component, it will be added. If a base entity had
    /// the component, it will be overridden, and the value of the base component
    /// will be copied to the entity before this function returns.
    ///
    /// # Parameters
    ///
    /// * `comp`: The component to get.
    ///
    /// # Returns
    ///
    /// Pointer to the component value.
    pub fn get_component_by_id_mut(&self, component_id: EntityT) -> *mut c_void {
        unsafe { ecs_get_mut_id(self.world, self.raw_id, component_id) as *mut c_void }
    }

    /// Get mutable pointer for a pair (untyped).
    /// This operation gets the value for a pair from the entity. If neither the
    /// first nor second element of the pair is a component, the operation will
    /// fail.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    pub fn get_pair_ids_mut(&self, first: EntityT, second: EntityT) -> *mut c_void {
        unsafe { ecs_get_mut_id(self.world, self.raw_id, ecs_pair(first, second)) as *mut c_void }
    }

    /// Get mutable pointer for the first element of a pair
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// # Parameters
    ///
    /// * `second`: The second element of the pair.
    pub fn get_pair_first_mut<First: CachedComponentData>(&self, second: EntityT) -> *mut First {
        let component_id = First::get_id(self.world);
        ecs_assert!(
            First::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            First::get_symbol_name()
        );
        unsafe {
            ecs_get_mut_id(self.world, self.raw_id, ecs_pair(component_id, second)) as *mut First
        }
    }

    /// Get mutable pointer for the second element of a pair.
    /// This operation gets the value for a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    pub fn get_pair_second_mut<Second: CachedComponentData>(&self, first: EntityT) -> *mut Second {
        let component_id = Second::get_id(self.world);
        ecs_assert!(
            Second::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            Second::get_symbol_name()
        );
        unsafe {
            ecs_get_mut_id(self.world, self.raw_id, ecs_pair(first, component_id)) as *mut Second
        }
    }

    /// Signal that component was modified.
    ///
    /// # Arguments
    ///
    /// * `comp` - The component that was modified.
    ///
    pub fn mark_component_id_modified(&self, component_id: IdT) {
        unsafe { ecs_modified_id(self.world, self.raw_id, component_id) }
    }

    /// Signal that component was modified.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the component that was modified.
    ///
    pub fn mark_component_modified<T: CachedComponentData>(&self) {
        ecs_assert!(
            T::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            T::get_symbol_name(),
        );
        self.mark_component_id_modified(T::get_id(self.world));
    }

    /// Signal that a pair has been modified (untyped).
    /// If neither the first nor the second element of the pair are a component, the
    /// operation will fail.
    ///
    /// # Parameters
    ///
    /// * `first` - The first element of the pair.
    /// * `second` - The second element of the pair.
    pub fn mark_pair_ids_modified(&self, first: EntityT, second: EntityT) {
        self.mark_component_id_modified(ecs_pair(first, second));
    }

    /// Signal that the first element of a pair was modified.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first part of the pair.
    /// * `Second` - The second part of the pair.
    pub fn mark_pair_modified<First, Second>(&self)
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        self.mark_pair_ids_modified(First::get_id(self.world), Second::get_id(self.world))
    }

    /// Signal that the first part of a pair was modified.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first part of the pair.
    ///
    /// # Parameters
    ///
    /// * `second` - The second element of the pair.
    pub fn mark_pair_first_modified<First: CachedComponentData>(&self, second: EntityT) {
        ecs_assert!(
            First::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            First::get_symbol_name(),
        );
        self.mark_pair_ids_modified(First::get_id(self.world), second)
    }

    /// Get a reference to a component.
    ///
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling `get` for the same component.
    ///
    /// - `T`: Component for which to get a reference.
    ///
    /// Returns: The reference.
    pub fn get_ref<T: CachedComponentData>(&self) -> Ref<T> {
        Ref::<T>::new(self.world, self.raw_id, T::get_id(self.world))
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
    pub fn get_ref_pair_first<First: CachedComponentData>(&self, second: EntityT) -> Ref<First> {
        Ref::<First>::new(
            self.world,
            self.raw_id,
            ecs_pair(First::get_id(self.world), second),
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
    pub fn get_ref_pair_second<Second: CachedComponentData>(&self, first: EntityT) -> Ref<Second> {
        Ref::<Second>::new(
            self.world,
            self.raw_id,
            ecs_pair(first, Second::get_id(self.world)),
        )
    }

    /// Recursively flatten relationship.
    ///
    /// # Parameters
    ///
    /// * `relationship`: The relationship to flatten.
    pub fn flatten(&self, relationship: EntityT) {
        unsafe {
            ecs_flatten(
                self.world,
                ecs_pair(relationship, self.raw_id),
                std::ptr::null_mut(),
            )
        }
    }

    /// Recursively flatten relationship with desc.
    ///
    /// # Parameters
    ///
    /// * `relationship`: The relationship to flatten.
    /// * `desc`: The flatten desc.
    pub fn flatten_w_desc(&self, relationship: EntityT, desc: *const ecs_flatten_desc_t) {
        unsafe { ecs_flatten(self.world, ecs_pair(relationship, self.raw_id), desc) }
    }

    /// Clear an entity.
    ///
    /// This operation removes all components from an entity without recycling
    /// the entity id.
    pub fn clear(&self) {
        unsafe { ecs_clear(self.world, self.raw_id) }
    }

    /// Delete an entity.
    ///
    /// Entities have to be deleted explicitly, and are not deleted when the
    /// entity object goes out of scope.
    pub fn destruct(self) {
        unsafe { ecs_delete(self.world, self.raw_id) }
    }

    /// Return entity as entity_view.
    /// This returns an entity_view instance for the entity which is a readonly
    /// version of the entity class.
    pub fn get_view(&self) -> EntityView {
        self.entity_view
    }

    /// Entity id 0.
    /// This function is useful when the API must provide an entity that
    /// belongs to a world, but the entity id is 0.
    pub fn null_w_world(world: *const WorldT) -> Entity {
        Entity::new(world as *mut WorldT)
    }

    /// Entity id 0.
    /// returns the default entity, which is 0 id and nullptr world
    pub fn null() -> Entity {
        Entity::default()
    }
}
