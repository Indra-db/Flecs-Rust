use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

use crate::{
    core::{
        c_binding::{
            bindings::{ecs_get_target, ecs_set_id, ecs_set_name},
            FLECS_IDEcsComponentID_,
        },
        FlecsErrorCode,
    },
    ecs_assert,
};

use super::{
    c_binding::bindings::{
        ecs_add_id, ecs_clear, ecs_delete, ecs_enable, ecs_enable_id, ecs_entity_desc_t,
        ecs_entity_init, ecs_flatten, ecs_flatten_desc_t, ecs_get_id, ecs_get_mut_id, ecs_has_id,
        ecs_modified_id, ecs_new_id, ecs_remove_id, ecs_set_alias, ecs_set_scope, ecs_set_with,
        EcsChildOf, EcsComponent, EcsDependsOn, EcsExclusive, EcsIsA, EcsSlotOf, EcsWildcard,
    },
    c_types::{EntityT, IdT, WorldT, SEPARATOR},
    component_ref::Ref,
    component_registration::{CachedComponentData, ComponentType, Enum, Struct},
    ecs_pair,
    enum_type::CachedEnumData,
    set_helper,
    world::World,
    EntityView, ScopedWorld, ECS_OVERRIDE,
};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity {
    pub entity_view: EntityView,
}

impl Deref for Entity {
    type Target = EntityView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity_view
    }
}

impl DerefMut for Entity {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity_view
    }
}

impl From<Entity> for IdT {
    fn from(entity: Entity) -> Self {
        entity.entity_view.id.raw_id
    }
}

// functions in here match most of the functions in the c++ entity and entity_builder class
impl Entity {
    /// Create new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(world: &World) -> Self {
        Self {
            entity_view: EntityView::new_from_existing(world.raw_world, unsafe {
                ecs_new_id(world.raw_world)
            }),
        }
    }

    /// Creates a wrapper around an existing entity / id.
    ///
    /// # Arguments
    ///
    /// * `world` - The world the entity belongs to. If strictly only a storage is needed, this can be None.
    /// * `id` - The entity id.
    ///
    /// # Safety
    ///
    /// The world must be not be None if you want to do operations on the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    pub fn new_from_existing(world: Option<&World>, id: IdT) -> Self {
        if let Some(world) = world {
            Self {
                entity_view: EntityView::new_from_existing(world.raw_world, id),
            }
        } else {
            Self {
                entity_view: EntityView::new_id_only(id),
            }
        }
    }

    /// Create a named entity.
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Arguments
    ///
    /// - `world`: The world in which to create the entity.
    /// - `name`: The entity name.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_named(world: &World, name: &CStr) -> Self {
        let desc = ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            symbol: std::ptr::null(),
            use_low_id: false,
            add: [0; 32],
            add_expr: std::ptr::null(),
        };
        let id = unsafe { ecs_entity_init(world.raw_world, &desc) };
        Self {
            entity_view: EntityView::new_from_existing(world.raw_world, id),
        }
    }

    /// Wrap an existing entity id.
    /// # Arguments
    /// * `world` - The world the entity belongs to.
    /// * `id` - The entity id.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    pub(crate) fn new_from_existing_raw(world: *mut WorldT, id: IdT) -> Self {
        Self {
            entity_view: EntityView::new_from_existing(world, id),
        }
    }

    // Explicit conversion from flecs::entity_t to Entity
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    pub(crate) const fn new_id_only(id: EntityT) -> Self {
        Self {
            entity_view: EntityView::new_id_only(id),
        }
    }

    /// Add an entity to an entity.
    ///
    /// Add an entity to the entity. This is typically used for tagging.
    ///
    /// # Arguments
    ///
    /// - `component_id`: The component to add.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add_id(self, component_id: IdT) -> Self {
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
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.add_id(T::get_id(world))
    }

    /// Add a pair to an entity.
    ///
    /// This operation adds a pair to the entity.
    ///
    /// # Arguments
    ///
    /// - `first`: The first element of the pair.
    /// - `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        self.add_id(ecs_pair(first, second))
    }

    /// Add a pair.
    /// This operation adds a pair to the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair
    /// * `Second`: The second element of the pair
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
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
    /// # Arguments
    ///
    /// - `enum_value`: The enum constant.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
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
    /// # Arguments
    ///
    /// - `enum_value`: The enumeration value.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
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
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `component`: The component to add.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
    pub fn add_id_if(self, component_id: IdT, condition: bool) -> Self {
        if condition {
            return self.add_id(component_id);
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
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
    pub fn add_if<T: CachedComponentData>(self, condition: bool) -> Self {
        let world = self.world;
        self.add_id_if(T::get_id(world), condition)
    }

    /// Conditional add.
    /// This operation adds if condition is true, removes if condition is false.
    ///
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
    pub fn add_pair_ids_if(self, first: EntityT, mut second: EntityT, condition: bool) -> Self {
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
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
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
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
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
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
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
    /// # Arguments
    ///
    /// * `condition`: The condition to evaluate.
    /// * `enum_value`: The enumeration constant.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add_if`
    #[doc(alias = "entity_builder::add_if")]
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
    /// # Arguments
    ///
    /// * `component_id`: The entity to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove`
    #[doc(alias = "entity_builder::remove")]
    pub fn remove_id(self, component_id: IdT) -> Self {
        unsafe { ecs_remove_id(self.world, self.raw_id, component_id) }
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
    pub fn remove<T: CachedComponentData + ComponentType<Struct>>(self) -> Self {
        let world = self.world;
        self.remove_id(T::get_id(world))
    }

    /// Remove pair for enum
    /// This operation will remove any (Enum, *) pair from the entity.
    ///
    /// # Type Parameters
    /// * `T` - The enum type.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove`
    #[doc(alias = "entity_builder::remove")]
    pub fn remove_enum<T: CachedComponentData + ComponentType<Enum>>(self) -> Self {
        let world = self.world;
        self.remove_pair_ids(T::get_id(world), unsafe { EcsWildcard })
    }

    /// Remove a pair.
    /// This operation removes a pair from the entity.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove`
    #[doc(alias = "entity_builder::remove")]
    pub fn remove_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        self.remove_id(ecs_pair(first, second))
    }

    /// Removes a pair.
    /// This operation removes a pair from the entity.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The type of the first element of the pair.
    /// * `Second`: The type of the second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove`
    #[doc(alias = "entity_builder::remove")]
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
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::remove_second`
    #[doc(alias = "entity_builder::remove_second")]
    pub fn remove_pair_first_id<Second: CachedComponentData>(self, first: EntityT) -> Self {
        let world = self.world;
        self.remove_pair_ids(first, Second::get_id(world))
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
    pub fn remove_pair_second_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.remove_pair_ids(First::get_id(world), second)
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
    pub fn is_a_id(self, second: EntityT) -> Self {
        self.add_pair_ids(unsafe { EcsIsA }, second)
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
    pub fn is_a_type<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.is_a_id(T::get_id(world))
    }

    /// Shortcut for add(IsA, entity).
    ///
    /// # Arguments
    ///
    /// * `parent`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::is_a`
    #[doc(alias = "entity_builder::is_a")]
    pub fn is_a(self, parent: &Entity) -> Self {
        self.is_a_id(parent.raw_id)
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
    pub fn child_of_id(self, second: EntityT) -> Self {
        self.add_pair_ids(unsafe { EcsChildOf }, second)
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
    pub fn child_of_type<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.child_of_id(T::get_id(world))
    }

    /// Shortcut for add(ChildOf, entity).
    ///
    /// # Arguments
    ///
    /// * `parent`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::child_of`
    #[doc(alias = "entity_builder::child_of")]
    pub fn child_of(self, parent: &Entity) -> Self {
        self.child_of_id(parent.raw_id)
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
    pub fn depends_on_id(self, second: EntityT) -> Self {
        self.add_pair_ids(unsafe { EcsDependsOn }, second)
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
    pub fn depends_on<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.depends_on_id(T::get_id(world))
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
    pub fn slot_of_id(self, second: EntityT) -> Self {
        self.add_pair_ids(unsafe { EcsSlotOf }, second)
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
    pub fn slot_of<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.slot_of_id(T::get_id(world))
    }

    /// Shortcut for add(SlotOf, target(ChildOf)).
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::slot`
    #[doc(alias = "entity_builder::slot")]
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
    pub fn mark_override_component_id(self, id: IdT) -> Self {
        self.add_id(ECS_OVERRIDE | id)
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
    pub fn mark_override_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.mark_override_component_id(T::get_id(world))
    }

    /// Mark pair for auto-overriding.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::override`
    #[doc(alias = "entity_builder::override")]
    pub fn mark_override_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        self.mark_override_component_id(ecs_pair(first, second))
    }

    /// Mark pair for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::override`
    #[doc(alias = "entity_builder::override")]
    pub fn mark_override_pair<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        let world = self.world;
        self.mark_override_pair_ids(First::get_id(world), Second::get_id(world))
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
    pub fn mark_override_pair_first<Second: CachedComponentData>(self, first: EntityT) -> Self {
        let world = self.world;
        self.mark_override_pair_ids(first, Second::get_id(world))
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
    pub fn mark_override_pair_second_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.mark_override_pair_ids(First::get_id(world), second)
    }

    /// Sets a component for an entity and marks it as overridden.
    ///
    /// This function sets a component for an entity and marks the component
    /// as overridden, meaning that it will not be updated by systems that
    /// typically update this component.
    ///
    /// # Arguments
    ///
    /// * `component_id`: The ID of the component to set and mark as overridden.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_override`
    #[doc(alias = "entity_builder::set_override")]
    pub fn set_mark_override_component_id(self, component_id: IdT) -> Self {
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
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_override`
    #[doc(alias = "entity_builder::set_override")]
    pub fn set_mark_override_component<T: CachedComponentData>(self, component: T) -> Self {
        self.mark_override_component::<T>().set(component)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The type of the first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The ID of the second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_override`
    #[doc(alias = "entity_builder::set_override")]
    pub fn set_mark_override_pair_first<First: CachedComponentData + ComponentType<Struct>>(
        self,
        second: EntityT,
        first: First,
    ) -> Self {
        self.mark_override_pair_second_id::<First>(second)
            .set_pair_first_id(second, first)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The type of the second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The ID of the second element of the pair.
    /// * `second`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_override`
    #[doc(alias = "entity_builder::set_override")]
    pub fn set_mark_override_pair_second<Second: CachedComponentData + ComponentType<Struct>>(
        self,
        second: Second,
        first: EntityT,
    ) -> Self {
        self.mark_override_pair_first::<Second>(first)
            .set_pair_first_id(first, second)
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
    pub fn set<T: CachedComponentData>(self, component: T) -> Self {
        set_helper(self.world, self.raw_id, component, T::get_id(self.world));
        self
    }

    /// Set a pair for an entity using the first element type and a second component ID.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set`
    #[doc(alias = "entity_builder::set")]
    pub fn set_pair_first_id<First: CachedComponentData>(
        self,
        second: EntityT,
        first: First,
    ) -> Self {
        set_helper(
            self.world,
            self.raw_id,
            first,
            ecs_pair(First::get_id(self.world), second),
        );
        self
    }

    /// Set a pair for an entity.
    /// This operation sets the pair value, and uses First as type. If the
    /// entity did not yet have the pair, it will be added.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair
    /// * `Second`: The second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The value to set for first component.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set`
    #[doc(alias = "entity_builder::set")]
    pub fn set_pair_first<First, Second>(self, first: First) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        set_helper(
            self.world,
            self.raw_id,
            first,
            ecs_pair(First::get_id(self.world), Second::get_id(self.world)),
        );
        self
    }

    /// Set a pair for an entity using the second element type and a first id.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_second`
    #[doc(alias = "entity_builder::set_second")]
    pub fn set_pair_second_id<Second: CachedComponentData>(
        self,
        first: EntityT,
        second: Second,
    ) -> Self {
        set_helper(
            self.world,
            self.raw_id,
            second,
            ecs_pair(first, Second::get_id(self.world)),
        );
        self
    }

    /// Set a pair for an entity.
    /// This operation sets the pair value, and uses Second as type. If the
    /// entity did not yet have the pair, it will be added.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `value`: The value to set.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_second`
    #[doc(alias = "entity_builder::set_second")]
    pub fn set_pair_second<First, Second>(self, second: Second) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        set_helper(
            self.world,
            self.raw_id,
            second,
            ecs_pair(First::get_id(self.world), Second::get_id(self.world)),
        );
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
    pub fn set_enum_pair_first<First, Second>(self, first: First, constant: Second) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        //not sure if this is correct
        set_helper(
            self.world,
            self.raw_id,
            first,
            ecs_pair(
                First::get_id(self.world),
                constant.get_entity_id_from_enum_field(self.world),
            ),
        );
        self
    }

    /// Sets a pointer to a component of an entity with a given component ID and size.
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
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
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
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_ptr`
    #[doc(alias = "entity_builder::set_ptr")]
    pub fn set_ptr(self, component_id: EntityT, ptr: *const c_void) -> Self {
        let cptr: *const EcsComponent =
            unsafe { ecs_get_id(self.world, component_id, FLECS_IDEcsComponentID_) }
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
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_name`
    #[doc(alias = "entity_builder::set_name")]
    pub fn set_name(self, name: &CStr) -> Self {
        unsafe {
            ecs_set_name(self.world, self.raw_id, name.as_ptr());
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
    pub fn set_alias_name(self, name: &CStr) -> Self {
        unsafe {
            ecs_set_alias(self.world, self.raw_id, name.as_ptr());
        }
        self
    }

    /// Enables an entity.
    ///
    /// Enabled entities are matched with systems and can be searched with queries.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::enable`
    #[doc(alias = "entity_builder::enable")]
    pub fn enable(self) -> Self {
        unsafe { ecs_enable(self.world, self.raw_id, true) }
        self
    }
    /// Enables an ID.
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
    pub fn enable_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_enable_id(self.world, self.raw_id, component_id, true) }
        self
    }

    /// Enables a component.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component to enable.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::enable`
    #[doc(alias = "entity_builder::enable")]
    pub fn enable_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.enable_component_id(T::get_id(world))
    }

    /// Enables a pair using IDs.
    ///
    /// # Arguments
    ///
    /// - `first`: The first element of the pair.
    /// - `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::enable`
    #[doc(alias = "entity_builder::enable")]
    pub fn enable_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        self.enable_component_id(ecs_pair(first, second))
    }

    /// Enables a pair.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The first element of the pair.
    /// - `U`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::enable`
    #[doc(alias = "entity_builder::enable")]
    pub fn enable_pair<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        let world = self.world;
        self.enable_pair_ids(First::get_id(world), Second::get_id(world))
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
    pub fn enable_pair_second<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.enable_pair_ids(First::get_id(world), second)
    }

    /// Disables an entity.
    ///
    /// Disabled entities are not matched with systems and cannot be searched with queries,
    /// unless explicitly specified in the query expression.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::disable`
    #[doc(alias = "entity_builder::disable")]
    pub fn disable(self) -> Self {
        unsafe { ecs_enable(self.world, self.raw_id, false) }
        self
    }

    /// Disables an ID.
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
    pub fn disable_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_enable_id(self.world, self.raw_id, component_id, false) }
        self
    }

    /// Disables a component.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component to disable.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::disable`
    #[doc(alias = "entity_builder::disable")]
    pub fn disable_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.disable_component_id(T::get_id(world))
    }

    /// Disables a pair using IDs.
    ///
    /// # Arguments
    ///
    /// - `first`: The first element of the pair.
    /// - `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::disable`
    #[doc(alias = "entity_builder::disable")]
    pub fn disable_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        self.disable_component_id(ecs_pair(first, second))
    }

    /// Disables a pair.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The first element of the pair.
    /// - `U`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::disable`
    #[doc(alias = "entity_builder::disable")]
    pub fn disable_pair<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        let world = self.world;
        self.disable_pair_ids(First::get_id(world), Second::get_id(world))
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
    pub fn disable_pair_second<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.disable_pair_ids(First::get_id(world), second)
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
    pub fn with_pair_first<First: CachedComponentData, F>(&self, func: F) -> &Self
    where
        F: FnOnce(),
    {
        let world = self.world;
        self.with_pair_first_id(First::get_id(world), func)
    }

    /// Entities created in the function will have a pair consisting of the current entity and a specified component.
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
    pub fn with_pair_second<Second: CachedComponentData, F>(&self, func: F) -> &Self
    where
        F: FnOnce(),
    {
        let world = self.world;
        self.with_pair_second_id(Second::get_id(world), func)
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
    pub fn run_in_scope<F>(&self, func: F) -> &Self
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

    /// Return world scoped to entity
    ///
    /// # Returns
    ///
    /// A world scoped to the entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::get_world`
    #[doc(alias = "entity_builder::get_world")]
    pub fn scope(&self) -> ScopedWorld {
        ScopedWorld::new(&World::new_wrap_raw_world(self.world), self.raw_id)
    }

    /// Gets mut component.
    /// Use `.unwrap()` or `.unwrap_unchecked()` or `.get_unchecked_mut` if you're sure the entity has the component
    ///
    /// This operation returns a mutable optional reference to the component. If the entity
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
    /// A mutable option ref to the component value.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_mut<T: CachedComponentData + ComponentType<Struct>>(
        &self,
    ) -> Option<&mut T::UnderlyingType> {
        let component_id = T::get_id(self.world);
        ecs_assert!(
            T::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            T::get_symbol_name()
        );
        unsafe {
            (ecs_get_mut_id(self.world, self.raw_id, component_id) as *mut T::UnderlyingType)
                .as_mut()
        }
    }

    /// Gets mut component unchecked
    ///
    /// This operation returns a mutable reference to the component. If the entity
    /// did not yet have the component, it will be added. If a base entity had
    /// the component, it will be overridden, and the value of the base component
    /// will be copied to the entity before this function returns.
    ///
    /// # Safety
    ///
    /// If the entity does not have the component, this will cause a panic
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub unsafe fn get_unchecked_mut<T: CachedComponentData + ComponentType<Struct>>(
        &mut self,
    ) -> &mut T::UnderlyingType {
        let component_id = T::get_id(self.world);
        ecs_assert!(
            T::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            T::get_symbol_name()
        );
        let ptr = ecs_get_mut_id(self.world, self.raw_id, component_id) as *mut T::UnderlyingType;
        ecs_assert!(
            !ptr.is_null(),
            FlecsErrorCode::InternalError,
            "missing component {}",
            T::get_symbol_name()
        );

        &mut *ptr
    }

    /// Get mut enum constant.
    /// Use `.unwrap()` or `.unwrap_unchecked()` or `.get_enum_unchecked_mut` if you're sure the entity has the component
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum component type which to get the constant
    ///
    /// # Returns
    ///
    /// * `*mut T` - The enum component, nullptr if the entity does not have the component
    ///
    /// # See also
    ///
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_enum_mut<T: CachedComponentData + ComponentType<Enum>>(
        &self,
    ) -> Option<&mut T::UnderlyingType> {
        let component_id: IdT = T::get_id(self.world);
        let target: IdT = unsafe { ecs_get_target(self.world, self.raw_id, component_id, 0) };

        if target == 0 {
            // if there is no matching pair for (r,*), try just r
            unsafe {
                (ecs_get_mut_id(self.world, self.raw_id, component_id) as *mut T::UnderlyingType)
                    .as_mut()
            }
        } else {
            // get constant value from constant entity
            let constant_value = unsafe {
                ecs_get_mut_id(self.world, target, component_id) as *mut T::UnderlyingType
            };

            ecs_assert!(
                !constant_value.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );

            unsafe { constant_value.as_mut() }
        }
    }

    /// Get mut enum constant unchecked
    ///
    /// # Safety
    ///
    /// If the entity does not have the component, this will cause a panic
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum component type which to get the constant
    ///
    /// # Returns
    ///
    /// * `*mut T` - The enum component, nullptr if the entity does not have the component
    ///
    /// # See also
    ///
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub unsafe fn get_enum_unchecked_mut<T: CachedComponentData + ComponentType<Enum>>(
        &mut self,
    ) -> &mut T::UnderlyingType {
        let component_id: IdT = T::get_id(self.world);
        let target: IdT = ecs_get_target(self.world, self.raw_id, component_id, 0);

        if target == 0 {
            // if there is no matching pair for (r,*), try just r
            let ptr =
                ecs_get_mut_id(self.world, self.raw_id, component_id) as *mut T::UnderlyingType;
            ecs_assert!(
                !ptr.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );

            &mut *ptr
        } else {
            // get constant value from constant entity
            let constant_value =
                ecs_get_mut_id(self.world, target, component_id) as *mut T::UnderlyingType;
            ecs_assert!(
                !constant_value.is_null(),
                FlecsErrorCode::InternalError,
                "missing enum constant value {}",
                T::get_symbol_name()
            );

            &mut *constant_value
        }
    }

    /// Get mutable component value (untyped).
    /// This operation returns a mutable pointer to the component. If the entity
    /// did not yet have the component, it will be added. If a base entity had
    /// the component, it will be overridden, and the value of the base component
    /// will be copied to the entity before this function returns.
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_untyped_mut(&self, component_id: EntityT) -> *mut c_void {
        unsafe { ecs_get_mut_id(self.world, self.raw_id, component_id) as *mut c_void }
    }

    /// Get mutable pointer for a pair (untyped).
    /// This operation gets the value for a pair from the entity. If neither the
    /// first nor second element of the pair is a component, the operation will
    /// fail.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_untyped_pair_mut(&self, first: EntityT, second: EntityT) -> *mut c_void {
        unsafe { ecs_get_mut_id(self.world, self.raw_id, ecs_pair(first, second)) as *mut c_void }
    }

    /// Get const pointer for the first element of a pair
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_pair_first<First: CachedComponentData>(&self, second: EntityT) -> *const First {
        let component_id = First::get_id(self.world);
        ecs_assert!(
            First::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            First::get_symbol_name()
        );
        unsafe {
            ecs_get_mut_id(self.world, self.raw_id, ecs_pair(component_id, second)) as *const First
        }
    }

    /// Get mutable pointer for the first element of a pair
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
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

    /// Get const pointer for the second element of a pair.
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_pair_second<Second: CachedComponentData>(&self, first: EntityT) -> *const Second {
        let component_id = Second::get_id(self.world);
        ecs_assert!(
            Second::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            Second::get_symbol_name()
        );
        unsafe {
            ecs_get_mut_id(self.world, self.raw_id, ecs_pair(first, component_id)) as *const Second
        }
    }

    /// Get mutable pointer for the second element of a pair.
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
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
    /// # See also
    ///
    /// * C++ API: `entity::modified`
    #[doc(alias = "entity::modified")]
    pub fn mark_modified_component_id(&self, component_id: IdT) {
        unsafe { ecs_modified_id(self.world, self.raw_id, component_id) }
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
    pub fn mark_modified_component<T: CachedComponentData>(&self) {
        ecs_assert!(
            T::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            T::get_symbol_name(),
        );
        self.mark_modified_component_id(T::get_id(self.world));
    }

    /// Signal that a pair has been modified (untyped).
    /// If neither the first nor the second element of the pair are a component, the
    /// operation will fail.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair.
    /// * `second` - The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::modified`
    #[doc(alias = "entity::modified")]
    pub fn mark_modified_pair_ids(&self, first: EntityT, second: EntityT) {
        self.mark_modified_component_id(ecs_pair(first, second));
    }

    /// Signal that the first element of a pair was modified.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first part of the pair.
    /// * `Second` - The second part of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::modified`
    #[doc(alias = "entity::modified")]
    pub fn mark_modified_pair<First, Second>(&self)
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        self.mark_modified_pair_ids(First::get_id(self.world), Second::get_id(self.world));
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
    pub fn mark_modified_pair_first<First: CachedComponentData>(&self, second: EntityT) {
        ecs_assert!(
            First::get_size(self.world) != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            First::get_symbol_name(),
        );
        self.mark_modified_pair_ids(First::get_id(self.world), second);
    }

    /// Get a reference to a component.
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
    pub fn get_ref_component<T: CachedComponentData>(&self) -> Ref<T::UnderlyingType> {
        Ref::<T::UnderlyingType>::new(self.world, self.raw_id, T::get_id(self.world))
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
    ///
    /// # See also
    ///
    /// * C++ API: `entity::get_ref`
    #[doc(alias = "entity::get_ref")]
    pub fn get_ref_pair_second<Second: CachedComponentData>(&self, first: EntityT) -> Ref<Second> {
        Ref::<Second>::new(
            self.world,
            self.raw_id,
            ecs_pair(first, Second::get_id(self.world)),
        )
    }

    /// Recursively flatten relationship.
    ///
    /// # Arguments
    ///
    /// * `relationship`: The relationship to flatten.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::flatten`
    #[doc(alias = "entity::flatten")]
    pub fn flatten(&self, relationship: EntityT) {
        unsafe {
            ecs_flatten(
                self.world,
                ecs_pair(relationship, self.raw_id),
                std::ptr::null_mut(),
            );
        }
    }

    /// Recursively flatten relationship with desc.
    ///
    /// # Arguments
    ///
    /// * `relationship`: The relationship to flatten.
    /// * `desc`: The flatten desc.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::flatten`
    #[doc(alias = "entity::flatten")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn flatten_w_desc(&self, relationship: EntityT, desc: *const ecs_flatten_desc_t) {
        unsafe { ecs_flatten(self.world, ecs_pair(relationship, self.raw_id), desc) }
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
        unsafe { ecs_clear(self.world, self.raw_id) }
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
        unsafe { ecs_delete(self.world, self.raw_id) }
    }

    /// Return entity as `entity_view`.
    /// This returns an `entity_view` instance for the entity which is a readonly
    /// version of the entity class.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::view`
    #[doc(alias = "entity::view")]
    pub fn get_view(&self) -> EntityView {
        self.entity_view
    }

    /// Entity id 0.
    /// This function is useful when the API must provide an entity that
    /// belongs to a world, but the entity id is 0.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::null`
    #[doc(alias = "entity::null")]
    pub fn null_w_world(world: *const WorldT) -> Entity {
        Entity::new_from_existing_raw(world as *mut _, 0)
    }

    /// Entity id 0.
    /// returns the default entity, which is 0 id and nullptr world
    ///
    /// # See also
    ///
    /// * C++ API: `entity::null`
    #[doc(alias = "entity::null")]
    pub fn null() -> Entity {
        Entity::default()
    }
}
