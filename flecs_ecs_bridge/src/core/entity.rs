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
        ecs_entity_init, ecs_get_id, ecs_get_mut_id, ecs_get_world, ecs_has_id, ecs_modified_id,
        ecs_new_w_id, ecs_remove_id, ecs_set_alias, ecs_set_scope, ecs_set_with, EcsChildOf,
        EcsComponent, EcsDependsOn, EcsExclusive, EcsIsA, EcsSlotOf, EcsWildcard,
        FLECS__EEcsComponent,
    },
    c_types::{EntityT, IdT, WorldT, SEPARATOR},
    component::{CachedComponentData, ComponentType, Enum, Struct},
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

    pub fn add_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_add_id(self.world, self.raw_id, component_id) }
        self
    }

    pub fn add_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.add_component_id(T::get_id(world))
    }

    pub fn add_pair_ids(self, id: EntityT, id2: EntityT) -> Self {
        let world = self.world;
        self.add_component_id(ecs_pair(id, id2))
    }

    pub fn add_pair<T, U>(self) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData + ComponentType<Struct>,
    {
        let world = self.world;
        self.add_pair_ids(T::get_id(world), U::get_id(world))
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
    ///
    /// # Returns
    ///
    /// Returns the updated entity.
    pub fn add_enum_tag<T, U>(self, enum_value: U) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        self.add_pair_ids(
            T::get_id(world),
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
    ///
    /// # Returns
    ///
    /// Returns the updated entity.
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

    pub fn add_pair_second<Second: CachedComponentData>(self, first: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(first, Second::get_id(world))
    }

    pub fn add_component_id_if(self, component_id: IdT, condition: bool) -> Self {
        if condition {
            let world = self.world;
            return self.add_component_id(component_id);
        }

        self
    }

    pub fn add_component_if<T: CachedComponentData>(self, condition: bool) -> Self {
        let world = self.world;
        self.add_component_id_if(T::get_id(world), condition)
    }

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

    pub fn add_pair_if<T, U>(self, condition: bool) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData + ComponentType<Struct>,
    {
        let world = self.world;
        self.add_pair_ids_if(T::get_id(world), U::get_id(world), condition)
    }

    pub fn add_enum_tag_if<T, U>(self, enum_value: U, condition: bool) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        self.add_pair_ids_if(
            T::get_id(world),
            enum_value.get_entity_id_from_enum_field(world),
            condition,
        )
    }

    pub fn remove_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_remove_id(self.world, self.raw_id, component_id) }
        self
    }

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

    pub fn remove_pair_ids(self, id: EntityT, id2: EntityT) -> Self {
        let world = self.world;
        self.remove_component_id(ecs_pair(id, id2))
    }

    pub fn remove_pair<T, U>(self) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData + ComponentType<Struct>,
    {
        let world = self.world;
        self.remove_pair_ids(T::get_id(world), U::get_id(world))
    }

    pub fn remove_enum_tag<T, U>(self, enum_value: U) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        let world = self.world;
        self.remove_pair_ids(
            T::get_id(world),
            enum_value.get_entity_id_from_enum_field(world),
        )
    }

    pub fn remove_pair_first_id<Second: CachedComponentData>(self, first: EntityT) -> Self {
        let world = self.world;
        self.remove_pair_ids(first, Second::get_id(world))
    }

    pub fn is_a_id(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(unsafe { EcsIsA }, second)
    }

    pub fn is_a<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.is_a_id(T::get_id(world))
    }

    pub fn child_of_id(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(unsafe { EcsChildOf }, second)
    }

    pub fn child_of<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.child_of_id(T::get_id(world))
    }

    pub fn depends_on_id(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(unsafe { EcsDependsOn }, second)
    }

    pub fn depends_on<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.depends_on_id(T::get_id(world))
    }

    pub fn slot_of_id(self, second: EntityT) -> Self {
        let world = self.world;
        self.add_pair_ids(unsafe { EcsSlotOf }, second)
    }

    pub fn slot_of<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.slot_of_id(T::get_id(world))
    }

    pub fn slot(self) -> Self {
        ecs_assert!(
            unsafe { ecs_get_target(self.world, self.raw_id, EcsChildOf, 0) } != 0,
            FlecsErrorCode::InvalidParameter,
            "add ChildOf pair before using slot()"
        );
        let id: u64 = self.get_target_from_entity(unsafe { EcsChildOf }, 0).raw_id;
        self.slot_of_id(id)
    }

    pub fn mark_component_id_for_override(self, id: IdT) -> Self {
        let world = self.world;
        self.add_component_id(unsafe { ECS_OVERRIDE | id })
    }

    pub fn mark_component_for_override<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.mark_component_id_for_override(T::get_id(world))
    }

    pub fn mark_pair_ids_for_override(self, id: EntityT, id2: EntityT) -> Self {
        let world = self.world;
        self.mark_component_id_for_override(ecs_pair(id, id2))
    }

    pub fn mark_pair_for_override<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        let world = self.world;
        self.mark_pair_ids_for_override(First::get_id(world), Second::get_id(world))
    }

    pub fn mark_pair_for_override_with_first_id<Second: CachedComponentData>(
        self,
        first: EntityT,
    ) -> Self {
        let world = self.world;
        self.mark_pair_ids_for_override(first, Second::get_id(world))
    }

    pub fn mark_pair_for_override_with_second_id<First: CachedComponentData>(
        self,
        second: EntityT,
    ) -> Self {
        let world = self.world;
        self.mark_pair_ids_for_override(First::get_id(world), second)
    }

    pub fn set_component<T: CachedComponentData>(self, component: T) -> Self {
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(raw_id, component, T::get_id(world))
    }

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

    pub fn set_pair_first_id<First: CachedComponentData>(
        self,
        first: First,
        second: EntityT,
    ) -> Self {
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(raw_id, first, ecs_pair(First::get_id(world), second))
    }

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

    pub fn set_pair_second_id<Second: CachedComponentData>(
        self,
        first: EntityT,
        second: Second,
    ) -> Self {
        let raw_id = self.raw_id;
        let world = self.world;
        self.set_helper(raw_id, second, ecs_pair(first, Second::get_id(world)))
    }

    //not sure if this is correct
    pub fn set_enum_pair_first<First, Second>(self, first: First, constant: Second) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
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

    pub fn set_pair_first_override<First: CachedComponentData + ComponentType<Struct>>(
        self,
        first: First,
        second: EntityT,
    ) -> Self {
        let world = self.world;
        self.mark_pair_for_override_with_first_id::<First>(second)
            .set_pair_first_id(first, second)
    }

    pub fn set_ptr_w_size(self, component_id: EntityT, size: usize, ptr: *const c_void) -> Self {
        unsafe { ecs_set_id(self.world, self.raw_id, component_id, size, ptr) };
        self
    }

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

    pub fn set_name(self, name: &str) -> Self {
        let c_name = std::ffi::CString::new(name).expect("Failed to convert to CString");
        unsafe {
            ecs_set_name(self.world, self.raw_id, c_name.as_ptr());
        }
        self
    }

    pub fn set_alias_name(self, name: &str) -> Self {
        let c_name = std::ffi::CString::new(name).expect("Failed to convert to CString");
        unsafe {
            ecs_set_alias(self.world, self.raw_id, c_name.as_ptr());
        }
        self
    }

    pub fn enable(self) -> Self {
        unsafe { ecs_enable(self.world, self.raw_id, true) }
        self
    }

    pub fn enable_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_enable_id(self.world, self.raw_id, component_id, true) }
        self
    }

    pub fn enable_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.enable_component_id(T::get_id(world))
    }

    pub fn enable_pair_ids(self, id: EntityT, id2: EntityT) -> Self {
        self.enable_component_id(ecs_pair(id, id2))
    }

    pub fn enable_pair<T, U>(self) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData,
    {
        let world = self.world;
        self.enable_pair_ids(T::get_id(world), U::get_id(world))
    }

    pub fn enable_pair_with_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.enable_pair_ids(First::get_id(world), second)
    }

    pub fn disable(self) -> Self {
        unsafe { ecs_enable(self.world, self.raw_id, false) }
        self
    }

    pub fn disable_component_id(self, component_id: IdT) -> Self {
        unsafe { ecs_enable_id(self.world, self.raw_id, component_id, false) }
        self
    }

    pub fn disable_component<T: CachedComponentData>(self) -> Self {
        let world = self.world;
        self.disable_component_id(T::get_id(world))
    }

    pub fn disable_pair_ids(self, id: EntityT, id2: EntityT) -> Self {
        self.disable_component_id(ecs_pair(id, id2))
    }

    pub fn disable_pair<T, U>(self) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData,
    {
        let world = self.world;
        self.disable_pair_ids(T::get_id(world), U::get_id(world))
    }

    pub fn disable_pair_with_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        let world = self.world;
        self.disable_pair_ids(First::get_id(world), second)
    }

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

    pub fn with_pair_first<First: CachedComponentData, F>(&self, func: F) -> &Self
    where
        F: FnOnce(),
    {
        let world = self.world;
        self.with_pair_first_id(First::get_id(world), func)
    }

    pub fn with_pair_second<Second: CachedComponentData, F>(&self, func: F) -> &Self
    where
        F: FnOnce(),
    {
        let world = self.world;
        self.with_pair_second_id(Second::get_id(world), func)
    }

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
    pub fn get_pair_ids_mut(&self, id: EntityT, id2: EntityT) -> *mut c_void {
        unsafe { ecs_get_mut_id(self.world, self.raw_id, ecs_pair(id, id2)) as *mut c_void }
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
    ///
    pub fn mark_pair_ids_modified(&self, id: EntityT, id2: EntityT) {
        self.mark_component_id_modified(ecs_pair(id, id2));
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

    pub fn destruct(self) {
        unsafe { ecs_delete(self.world, self.raw_id) }
    }

    pub fn clear(&self) {
        unsafe { ecs_clear(self.world, self.raw_id) }
    }
}
