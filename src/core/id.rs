use super::c_binding::bindings::*;
use super::c_types::*;
use super::entity::*;
use crate::core::utility::functions::*;

pub struct Id {
    pub world: *mut WorldT,
    pub id: IdT,
}

impl Id {
    pub const fn new(world: *mut WorldT, id: IdT) -> Self {
        Self { world, id }
    }

    pub const fn new_only_id(id: IdT) -> Self {
        Self {
            world: std::ptr::null_mut(),
            id,
        }
    }

    pub const fn new_only_world(world: *mut WorldT) -> Self {
        Self { world, id: 0 }
    }

    pub fn new_world_pair(world: *mut WorldT, first: IdT, second: IdT) -> Self {
        Self {
            world,
            id: ecs_pair(first, second),
        }
    }

    pub fn new_pair_only(first: IdT, second: IdT) -> Self {
        Self {
            world: std::ptr::null_mut(),
            id: ecs_pair(first, second),
        }
    }

    pub fn new_from_ids(id: Id, id2: Id) -> Self {
        Self {
            world: id.world,
            id: ecs_pair(id.id, id2.id),
        }
    }

    pub const fn default() -> Self {
        Self {
            world: std::ptr::null_mut(),
            id: 0,
        }
    }

    pub fn is_pair(&self) -> bool {
        unsafe { ecs_id_is_pair(self.id) }
    }

    pub fn is_wildcard(&self) -> bool {
        unsafe { ecs_id_is_wildcard(self.id) }
    }

    pub fn is_entity(&self) -> bool {
        self.id & RUST_ECS_ID_FLAGS_MASK == 0
    }

    /// Return id as entity (only allowed when id is valid entity)
    #[inline(always)]
    pub fn entity(&self) -> Entity {
        #[cfg(feature = "enable_core_asserts")]
        {
            assert!(!self.is_pair());
            //TODO
            //assert!(!self.flags());
        }
        Entity::new(self.world, self.id)
    }

    /// Return id with role added
    #[inline(always)]
    pub fn add_flags(&self, flags: IdT) -> Entity {
        Entity::new(self.world, self.id | flags)
    }

    /// Return id without role
    #[inline(always)]
    pub fn remove_flags_check(&self, _flags: IdT) -> Entity {
        #[cfg(feature = "enable_core_asserts")]
        {
            assert!(self.id & RUST_ECS_ID_FLAGS_MASK == _flags);
        }
        Entity::new(self.world, self.id & RUST_ECS_COMPONENT_MASK)
    }

    /// Return id without role
    #[inline(always)]
    pub fn remove_flags(&self) -> Entity {
        Entity::new(self.world, self.id & RUST_ECS_COMPONENT_MASK)
    }

    /// Return id flags set on id
    #[inline(always)]
    pub fn flags(&self) -> Entity {
        Entity::new(self.world, self.id & RUST_ECS_ID_FLAGS_MASK)
    }

    /// Test if id has specified role
    #[inline(always)]
    pub fn has_flags_for_role(&self, flags: IdT) -> bool {
        self.id & flags == flags
    }

    /// Test if id has any role
    #[inline(always)]
    pub fn has_flags_any_role(&self) -> bool {
        self.id & RUST_ECS_ID_FLAGS_MASK != 0
    }

    /// Return id without role
    #[inline(always)]
    pub fn remove_generation(&self) -> Entity {
        Entity::new(self.world, self.id as u32 as u64)
    }

    /// Return component type of id
    #[inline(always)]
    pub fn type_id(&self) -> Entity {
        Entity::new(self.world, unsafe { ecs_get_typeid(self.world, self.id) })
    }

    /// Test if id has specified first
    #[inline(always)]
    pub fn has_relationship(&self, first: IdT) -> bool {
        if !self.is_pair() {
            return false;
        }
        false
        //ecs_entity_t_hi
    }
}
