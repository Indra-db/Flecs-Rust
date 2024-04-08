use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

use super::{
    c_types::{IdT, SEPARATOR},
    component_ref::Ref,
    component_registration::{ComponentId, ComponentType, Enum, Struct},
    ecs_pair, ecs_pair_first, ecs_pair_second, set_helper,
    world::World,
    CachedEnumData, EmptyComponent, EntityView, IntoComponentId, IntoEntityId, IntoEntityIdExt,
    IntoWorld, NotEmptyComponent, ScopedWorld, ECS_DEPENDS_ON, ECS_EXCLUSIVE, ECS_IS_A,
    ECS_OVERRIDE, ECS_SLOT_OF, ECS_WILDCARD,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;
use crate::{
    core::ECS_CHILD_OF,
    ecs_assert,
    sys::{
        ecs_add_id, ecs_clear, ecs_delete, ecs_enable, ecs_enable_id, ecs_entity_desc_t,
        ecs_entity_init, ecs_flatten, ecs_flatten_desc_t, ecs_get_id, ecs_get_mut_id,
        ecs_get_target, ecs_has_id, ecs_modified_id, ecs_new_id, ecs_remove_id, ecs_set_alias,
        ecs_set_id, ecs_set_name, ecs_set_scope, ecs_set_with, EcsComponent,
        FLECS_IDEcsComponentID_,
    },
};

#[derive(Default, Copy, Clone)]
pub struct Entity<'a> {
    pub entity_view: EntityView<'a>,
}

impl<'a, T> PartialEq<T> for Entity<'a>
where
    T: IntoEntityIdExt,
{
    fn eq(&self, other: &T) -> bool {
        self.raw_id == other.get_id()
    }
}

impl<'a> Eq for Entity<'a> {}

impl<'a, T> PartialOrd<T> for Entity<'a>
where
    T: IntoEntityIdExt,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        Some(self.raw_id.cmp(&other.get_id()))
    }
}

impl<'a> Ord for Entity<'a> {
    fn cmp(&self, other: &Entity) -> std::cmp::Ordering {
        self.raw_id.cmp(&other.raw_id)
    }
}

// Additionally, to allow comparison in the other direction (i32 with MyStruct)
impl<'a> PartialEq<Entity<'a>> for u64 {
    fn eq(&self, other: &Entity) -> bool {
        *self == other.raw_id
    }
}

impl<'a> Deref for Entity<'a> {
    type Target = EntityView<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity_view
    }
}

impl<'a> DerefMut for Entity<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity_view
    }
}

impl<'a> From<Entity<'a>> for IdT {
    fn from(entity: Entity) -> Self {
        entity.entity_view.id.raw_id
    }
}

impl<'a> From<&Entity<'a>> for IdT {
    fn from(entity: &Entity) -> Self {
        entity.entity_view.id.raw_id
    }
}

impl<'a> From<&mut Entity<'a>> for IdT {
    fn from(entity: &mut Entity) -> Self {
        entity.entity_view.id.raw_id
    }
}

// TODO: Unsafe static lifetime injected here
impl<'a> From<IdT> for Entity<'a> {
    fn from(value: IdT) -> Self {
        Entity::new_id_only(value)
    }
}

impl<'a> std::fmt::Display for Entity<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name_optional() {
            write!(f, "{}", name)
        } else {
            write!(f, "{}", self.raw_id)
        }
    }
}

impl<'a> std::fmt::Debug for Entity<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();
        let id = self.raw_id;
        let archetype_str = if let Some(s) = self.archetype().to_string() {
            s
        } else {
            "empty".to_string()
        };
        write!(
            f,
            "Entity name: {} -- id: {} -- archetype: {}",
            name, id, archetype_str
        )
    }
}

// functions in here match most of the functions in the c++ entity and entity_builder class
impl<'a> Entity<'a> {
    /// Create new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(world: &'a World) -> Self {
        let id = unsafe { ecs_new_id(world.world_ptr_mut()) };
        Self {
            entity_view: EntityView::new(Some(world), id),
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
    pub fn new_from_existing(world: impl IntoWorld<'a>, id: impl IntoEntityIdExt) -> Self {
        Self {
            entity_view: EntityView::new(world, id),
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
    pub fn new_named(world: impl IntoWorld<'a>, name: &CStr) -> Self {
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
        let id = unsafe { ecs_entity_init(world.world_ptr_mut(), &desc) };
        Self {
            entity_view: EntityView::new(world, id),
        }
    }

    // Explicit conversion from flecs::entity_t to Entity
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    pub(crate) fn new_id_only(id: impl IntoEntityIdExt) -> Self {
        Self {
            entity_view: EntityView::new_id_only(id),
        }
    }

    /// Entity id 0.
    /// This function is useful when the API must provide an entity that
    /// belongs to a world, but the entity id is 0.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::null`
    #[doc(alias = "entity::null")]
    pub fn new_null_w_world(world: &'a World) -> Entity<'a> {
        Entity::new_from_existing(Some(world), 0)
    }

    /// Entity id 0.
    /// returns the default entity, which is 0 id and nullptr world
    ///
    /// # See also
    ///
    /// * C++ API: `entity::null`
    #[doc(alias = "entity::null")]
    pub const fn new_null() -> Entity<'static> {
        Entity {
            entity_view: EntityView {
                id: super::Id {
                    raw_id: 0,
                    world: None,
                },
            },
        }
    }

    /// Add an id to an entity.
    /// This Id can be a component, a pair, a tag or another entity.
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
    pub fn add_id(self, id: impl IntoEntityIdExt) -> Self {
        unsafe { ecs_add_id(self.world.world_ptr_mut(), self.raw_id, id.get_id()) }
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
    /// # SAFETY
    ///
    /// This function is unsafe, but not marked unsafe. This is because the function does not initialize the component
    /// When it's Trivial. This usually means anything that does not store any heap data, will be uninitialized.
    /// Prefer Set for no risk of Undefined behavior.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::add`
    #[doc(alias = "entity_builder::add")]
    pub fn add<T>(self) -> Self
    where
        T: IntoComponentId,
    {
        let world = self.world;
        self.add_id(T::get_id(world))
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
    pub fn add_pair_second<Second: ComponentId>(self, first: impl IntoEntityId) -> Self {
        let world = self.world;
        self.add_id((first, Second::get_id(world)))
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
    pub fn add_pair_first<First: ComponentId>(self, second: impl IntoEntityId) -> Self {
        let world = self.world;
        self.add_id((First::get_id(world), second))
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
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        self.add_id((First::get_id(world), enum_value.get_id_variant(world)))
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
        let id = T::get_id(world);
        // SAFETY: we know that the enum_value is a valid because of the T::get_id call
        self.add_id((id, unsafe { enum_value.get_id_variant_unchecked(world) }))
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
        T: IntoEntityIdExt,
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
                let first = ecs_pair_first(id.get_id());
                let mut second = ecs_pair_second(id.get_id());
                if second == 0
                    || unsafe { ecs_has_id(self.world.world_ptr_mut(), first, ECS_EXCLUSIVE) }
                {
                    second = ECS_WILDCARD;
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
        self.add_id_if(T::get_id(world), condition)
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
    pub fn add_pair_first_if<First: ComponentId>(
        self,
        second: impl IntoEntityId,
        condition: bool,
    ) -> Self {
        let world = self.world;
        self.add_id_if((First::get_id(world), second), condition)
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
    pub fn add_pair_second_if<Second: ComponentId>(
        self,
        first: impl IntoEntityId,
        condition: bool,
    ) -> Self {
        let world = self.world;
        self.add_id_if((first, Second::get_id(world)), condition)
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
    pub fn remove_id(self, id: impl IntoEntityIdExt) -> Self {
        unsafe { ecs_remove_id(self.world.world_ptr_mut(), self.raw_id, id.get_id()) }
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
    pub fn remove_pair_first<First: ComponentId>(self, second: impl IntoEntityId) -> Self {
        let world = self.world;
        self.remove_id((First::get_id(world), second))
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
    pub fn remove_pair_second<Second: ComponentId>(self, first: impl IntoEntityId) -> Self {
        let world = self.world;
        self.remove_id((first, Second::get_id(world)))
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
    pub fn is_a_id(self, second: impl IntoEntityId) -> Self {
        self.add_id((ECS_IS_A, second))
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
    pub fn child_of_id(self, parent: impl IntoEntityId) -> Self {
        self.add_id((ECS_CHILD_OF, parent))
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
    pub fn depends_on_id(self, second: impl IntoEntityId) -> Self {
        self.add_id((ECS_DEPENDS_ON, second))
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
    pub fn depends_on<T: ComponentId>(self) -> Self {
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
    pub fn slot_of_id(self, second: impl IntoEntityId) -> Self {
        self.add_id((ECS_SLOT_OF, second))
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
    pub fn slot_child(self) -> Self {
        ecs_assert!(
            unsafe { ecs_get_target(self.world.world_ptr_mut(), self.raw_id, ECS_CHILD_OF, 0) }
                != 0,
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
    pub fn override_id(self, id: impl IntoEntityIdExt) -> Self {
        self.add_id(ECS_OVERRIDE | id.get_id())
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
    pub fn override_type<T: IntoComponentId>(self) -> Self {
        let world = self.world;
        self.override_id(T::get_id(world))
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
    pub fn override_pair_first<First: ComponentId>(self, second: impl IntoEntityId) -> Self {
        let world = self.world;
        self.override_id((First::get_id(world), second))
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
    pub fn override_pair_second<Second: ComponentId>(self, first: impl IntoEntityId) -> Self {
        let world = self.world;
        self.override_id((first, Second::get_id(world)))
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
    pub fn set_override_id(self, id: impl IntoEntityIdExt) -> Self {
        unsafe {
            ecs_add_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                ECS_OVERRIDE | id.get_id(),
            )
        }
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
    pub fn set_override<T: ComponentId>(self, component: T) -> Self {
        self.override_type::<T>().set(component)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The type of the first element of the pair.
    /// * `Second`: The type of the second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_override`
    pub fn set_override_pair_first<First, Second>(self, first: First) -> Self
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId + ComponentType<Struct>,
    {
        let second_id = Second::get_id(self.world);
        self.override_pair_first::<First>(second_id)
            .set_pair_first_id(first, second_id)
    }

    /// Sets a pair, mark component for auto-overriding.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The type of the first element of the pair.
    /// * `Second`: The type of the second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_override`
    pub fn set_override_pair_second<First, Second>(self, second: Second) -> Self
    where
        First: ComponentId + ComponentType<Struct>,
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let first_id = First::get_id(self.world);
        self.override_pair_second::<Second>(first_id)
            .set_pair_second_id(second, first_id)
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
    pub fn set_override_pair_first_id<First>(self, first: First, second: impl IntoEntityId) -> Self
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        self.override_pair_first::<First>(&second)
            .set_pair_first_id(first, second)
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
    pub fn set_override_pair_second_id<Second>(
        self,
        second: Second,
        first: impl IntoEntityId,
    ) -> Self
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let first = first.get_id();
        self.override_pair_second::<Second>(first)
            .set_pair_second_id(second, first)
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
    pub fn set<T: ComponentId>(self, component: T) -> Self {
        set_helper(
            self.world.world_ptr_mut(),
            self.raw_id,
            component,
            T::get_id(self.world),
        );
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
    pub fn set_pair_first_id<First>(self, first: First, second: impl IntoEntityId) -> Self
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        set_helper(
            self.world.world_ptr_mut(),
            self.raw_id,
            first,
            (First::get_id(self.world), second),
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
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId + ComponentType<Struct>,
    {
        set_helper(
            self.world.world_ptr_mut(),
            self.raw_id,
            first,
            (First::get_id(self.world), Second::get_id(self.world)),
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
    pub fn set_pair_second_id<Second>(self, second: Second, first: impl IntoEntityId) -> Self
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        set_helper(
            self.world.world_ptr_mut(),
            self.raw_id,
            second,
            (first, Second::get_id(self.world)),
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
        First: ComponentId + ComponentType<Struct> + EmptyComponent,
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        set_helper(
            self.world.world_ptr_mut(),
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
        First: ComponentId + ComponentType<Struct>,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        set_helper(
            self.world.world_ptr_mut(),
            self.raw_id,
            first,
            ecs_pair(
                First::get_id(self.world),
                constant.get_id_variant(self.world),
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
    pub fn set_ptr_w_size(self, id: impl IntoEntityId, size: usize, ptr: *const c_void) -> Self {
        unsafe {
            ecs_set_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                id.get_id(),
                size,
                ptr,
            )
        };
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
    pub fn set_ptr(self, id: impl IntoEntityId, ptr: *const c_void) -> Self {
        let id = id.get_id();
        let cptr: *const EcsComponent =
            unsafe { ecs_get_id(self.world.world_ptr_mut(), id, FLECS_IDEcsComponentID_) }
                as *const EcsComponent;

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
    pub fn set_name(self, name: &CStr) -> Self {
        unsafe {
            ecs_set_name(self.world.world_ptr_mut(), self.raw_id, name.as_ptr());
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
            ecs_set_alias(self.world.world_ptr_mut(), self.raw_id, name.as_ptr());
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
        unsafe { ecs_enable(self.world.world_ptr_mut(), self.raw_id, true) }
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
    pub fn enable_id(self, id: impl IntoEntityIdExt) -> Self {
        unsafe { ecs_enable_id(self.world.world_ptr_mut(), self.raw_id, id.get_id(), true) }
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
    pub fn enable_pair_second<First: ComponentId>(self, second: impl IntoEntityId) -> Self {
        let world = self.world;
        self.enable_id((First::get_id(world), second))
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
        unsafe { ecs_enable(self.world.world_ptr_mut(), self.raw_id, false) }
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
    pub fn disable_id(self, id: impl IntoEntityIdExt) -> Self {
        unsafe { ecs_enable_id(self.world.world_ptr_mut(), self.raw_id, id.get_id(), false) }
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
    pub fn disable_pair_first<First: ComponentId>(self, second: impl IntoEntityId) -> Self {
        let world = self.world;
        self.disable_id((First::get_id(world), second))
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
    pub fn with<F>(self, func: F) -> Self
    where
        F: FnOnce(),
    {
        unsafe {
            let prev = ecs_set_with(self.world.world_ptr_mut(), self.raw_id);
            func();
            ecs_set_with(self.world.world_ptr_mut(), prev);
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
    pub fn with_pair_first_id<F>(self, first: impl IntoEntityId, func: F) -> Self
    where
        F: FnOnce(),
    {
        unsafe {
            let prev = ecs_set_with(
                self.world.world_ptr_mut(),
                ecs_pair(first.get_id(), self.raw_id),
            );
            func();
            ecs_set_with(self.world.world_ptr_mut(), prev);
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
    pub fn with_pair_second_id<F>(self, second: impl IntoEntityId, func: F) -> Self
    where
        F: FnOnce(),
    {
        unsafe {
            let prev = ecs_set_with(
                self.world.world_ptr_mut(),
                ecs_pair(self.raw_id, second.get_id()),
            );
            func();
            ecs_set_with(self.world.world_ptr_mut(), prev);
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
    pub fn with_pair_first<First: ComponentId, F>(self, func: F) -> Self
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
    pub fn with_pair_second<Second: ComponentId, F>(self, func: F) -> Self
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
    pub fn run_in_scope<F>(self, func: F) -> Self
    where
        F: FnOnce(),
    {
        unsafe {
            let prev = ecs_set_scope(self.world.world_ptr_mut(), self.raw_id);
            func();
            ecs_set_scope(self.world.world_ptr_mut(), prev);
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
        ScopedWorld::new(self.world(), self.raw_id)
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    #[allow(clippy::mut_from_ref)]
    pub fn get_mut<T: ComponentId>(self) -> &'static mut T::UnderlyingType {
        // This branch will be removed in release mode since this can be determined at compile time.
        if !T::IS_ENUM {
            let component_id = T::get_id(self.world);

            ecs_assert!(
                std::mem::size_of::<T>() != 0,
                FlecsErrorCode::InvalidParameter,
                "invalid type: {}",
                std::any::type_name::<T>()
            );

            unsafe {
                &mut *(ecs_get_mut_id(self.world.world_ptr_mut(), self.raw_id, component_id)
                    as *mut T::UnderlyingType)
            }
        } else {
            let component_id: IdT = T::get_id(self.world);
            let target: IdT =
                unsafe { ecs_get_target(self.world.world_ptr_mut(), self.raw_id, component_id, 0) };

            if target == 0 {
                // if there is no matching pair for (r,*), try just r
                unsafe {
                    &mut *(ecs_get_mut_id(self.world.world_ptr_mut(), self.raw_id, component_id)
                        as *mut T::UnderlyingType)
                }
            } else {
                // get constant value from constant entity
                let constant_value = unsafe {
                    ecs_get_mut_id(self.world.world_ptr_mut(), target, component_id)
                        as *mut T::UnderlyingType
                };

                ecs_assert!(
                    !constant_value.is_null(),
                    FlecsErrorCode::InternalError,
                    "missing enum constant value {}",
                    std::any::type_name::<T>()
                );

                unsafe { &mut *constant_value }
            }
        }
    }

    pub fn get_callback_mut<T: ComponentId + 'static>(
        self,
        callback: impl FnOnce(&'static mut T::UnderlyingType),
    ) -> bool {
        if self.has::<T>() {
            let comp = self.get_mut::<T>();
            callback(comp);
            true
        } else {
            false
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
    #[doc(hidden)] // flecs 3.2.12 yet to be released
    #[doc(alias = "entity::ensure")]
    pub unsafe fn ensure_mut_unchecked<T: ComponentId + ComponentType<Struct>>(
        &mut self,
    ) -> &mut T::UnderlyingType {
        let component_id = T::get_id(self.world);

        ecs_assert!(
            std::mem::size_of::<T>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<T>()
        );

        let ptr = ecs_get_mut_id(self.world.world_ptr_mut(), self.raw_id, component_id)
            as *mut T::UnderlyingType;
        ecs_assert!(
            !ptr.is_null(),
            FlecsErrorCode::InternalError,
            "missing component {}",
            std::any::type_name::<T>()
        );

        &mut *ptr
    }

    /// Get mutable component value or pair (untyped).
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
    pub fn get_untyped_mut(self, id: impl IntoEntityIdExt) -> *mut c_void {
        unsafe {
            ecs_get_mut_id(self.world.world_ptr_mut(), self.raw_id, id.get_id()) as *mut c_void
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_pair_first_id_mut<First>(self, second: impl IntoEntityId) -> &'static mut First
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

        // SAFETY: The pointer is valid because ecs_get_mut_id adds the component if not present, so
        // it is guaranteed to be valid
        unsafe {
            &mut *(ecs_get_mut_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                ecs_pair(component_id, second.get_id()),
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_pair_first_mut<First, Second>(&mut self) -> &'static mut First
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId + ComponentType<Struct>,
    {
        self.get_pair_first_id_mut::<First>(Second::get_id(self.world))
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_pair_second_id_mut<Second>(self, first: impl IntoEntityId) -> &'static mut Second
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

        // SAFETY: The pointer is valid because ecs_get_mut_id adds the component if not present, so
        // it is guaranteed to be valid
        unsafe {
            &mut *(ecs_get_mut_id(
                self.world.world_ptr_mut(),
                self.raw_id,
                ecs_pair(first.get_id(), component_id),
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
    /// * C++ API: `entity::get_mut`
    #[doc(alias = "entity::get_mut")]
    pub fn get_pair_second_mut<First, Second>(&mut self) -> &'static mut Second
    where
        First: ComponentId + ComponentType<Struct> + EmptyComponent,
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        self.get_pair_second_id_mut::<Second>(First::get_id(self.world))
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
    pub fn modified_id(self, id: impl IntoEntityIdExt) {
        unsafe { ecs_modified_id(self.world.world_ptr_mut(), self.raw_id, id.get_id()) }
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
    pub fn modified_pair_first<First: ComponentId>(self, second: impl IntoEntityId) {
        ecs_assert!(
            std::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<First>()
        );

        self.modified_id((First::get_id(self.world), second));
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
    pub fn get_ref<T: ComponentId>(&self) -> Ref<'a, T::UnderlyingType> {
        Ref::<T::UnderlyingType>::new(Some(self.world), self.raw_id, T::get_id(self.world))
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
    pub fn get_ref_pair_first<First: ComponentId>(
        self,
        second: impl IntoEntityId,
    ) -> Ref<'a, First> {
        Ref::<First>::new(
            Some(self.world),
            self.raw_id,
            ecs_pair(First::get_id(self.world), second.get_id()),
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
    pub fn get_ref_pair_second<Second: ComponentId>(
        &self,
        first: impl IntoEntityId,
    ) -> Ref<Second> {
        Ref::<Second>::new(
            Some(self.world),
            self.raw_id,
            ecs_pair(first.get_id(), Second::get_id(self.world)),
        )
    }

    /// Recursively flatten relationship (relationship, self)
    ///
    /// # Arguments
    ///
    /// * `relationship`: The relationship to flatten.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::flatten`
    #[doc(alias = "entity::flatten")]
    pub fn flatten(self, relationship: impl IntoEntityId) {
        unsafe {
            ecs_flatten(
                self.world.world_ptr_mut(),
                ecs_pair(relationship.get_id(), self.raw_id),
                std::ptr::null_mut(),
            );
        }
    }

    /// Recursively flatten relationship (relationship, self) with desc
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
    pub fn flatten_w_desc(self, relationship: impl IntoEntityId, desc: *const ecs_flatten_desc_t) {
        unsafe {
            ecs_flatten(
                self.world.world_ptr_mut(),
                ecs_pair(relationship.get_id(), self.raw_id),
                desc,
            );
        }
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
        unsafe { ecs_clear(self.world.world_ptr_mut(), self.raw_id) }
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
        unsafe { ecs_delete(self.world.world_ptr_mut(), self.raw_id) }
    }

    /// Return entity as `entity_view`.
    /// This returns an `entity_view` instance for the entity which is a readonly
    /// version of the entity class.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::view`
    #[doc(alias = "entity::view")]
    pub fn as_view(&self) -> EntityView {
        self.entity_view
    }
}
