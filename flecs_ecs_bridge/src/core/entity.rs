use std::{ops::Deref, os::raw::c_void};

use super::{
    c_binding::bindings::{ecs_add_id, ecs_clear, ecs_delete, ecs_get_world, ecs_new_w_id},
    c_types::{EntityT, IdT, WorldT},
    component::{CachedComponentData, ComponentType, Enum, Struct},
    entity_view::EntityView,
    enum_type::CachedEnumData,
    id::Id,
    utility::functions::ecs_pair,
    utility::macros::*,
};

macro_rules! add_pair {
    ($self:expr, $id:expr, $id2:expr) => {
        unsafe { ecs_add_id($self.world, $self.raw_id, ecs_pair($id, $id2)) }
    };
}

macro_rules! add_id {
    ($self:expr, $id:expr) => {
        unsafe { ecs_add_id($self.world, $self.raw_id, $id) }
    };
}

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

    pub fn add_component<T: CachedComponentData>(self) -> Self {
        add_id!(self, T::get_id(self.world));
        self
    }

    pub fn add_component_with_id(self, component_id: IdT) -> Self {
        add_id!(self, component_id);
        self
    }

    pub fn add_pair_from_ids(self, id: EntityT, id2: EntityT) -> Self {
        add_pair!(self, id, id2);
        self
    }

    pub fn add_pair<T, U>(self) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData + ComponentType<Struct>,
    {
        add_pair!(self, T::get_id(self.world), U::get_id(self.world));
        self
    }

    pub fn add_enum_tag<T, U>(self, enum_value: U) -> Self
    where
        T: CachedComponentData,
        U: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        add_pair!(
            self,
            T::get_id(self.world),
            enum_value.get_entity_id_from_enum_field(self.world)
        );
        self
    }

    pub fn add_pair_second<Second: CachedComponentData>(self, first: EntityT) -> Self {
        add_pair!(self, first, Second::get_id(self.world));
        self
    }

    pub fn add_component_with_id_if(self, component_id: IdT, condition: bool) -> Self {
        if condition {
            add_id!(self, component_id);
        }
        todo!("remove");

        self
    }

    //
    //
    //
    //
    //
    pub fn destruct(self) {
        unsafe { ecs_delete(self.world, self.raw_id) }
    }

    pub fn clear(&self) {
        unsafe { ecs_clear(self.world, self.raw_id) }
    }
}
