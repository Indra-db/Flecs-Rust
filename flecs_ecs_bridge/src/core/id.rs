use super::c_binding::bindings::*;
use super::c_types::*;
use super::entity::*;
use super::world::World;
use crate::core::utility::{errors::*, functions::*};
use crate::ecs_assert;

#[derive(Debug, Clone, Copy, Eq)]
pub struct Id {
    /// World is optional, but guarantees that entity identifiers extracted from the id are valid
    pub world: *mut WorldT,
    pub raw_id: IdT,
}

impl Default for Id {
    fn default() -> Self {
        Self {
            world: std::ptr::null_mut(),
            raw_id: 0,
        }
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.raw_id == other.raw_id
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.raw_id.cmp(&other.raw_id))
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw_id.cmp(&other.raw_id)
    }
}

pub enum With {
    Id(IdT),
    Pair(IdT, IdT),
}
impl Id {
    pub fn new(world: Option<&World>, with: With) -> Self {
        if let Some(world) = world {
            match with {
                With::Id(id) => Self::new_from_existing(world.raw_world, id),
                With::Pair(id1, id2) => Self::new_world_pair(world.raw_world, id1, id2),
            }
        } else {
            match with {
                With::Id(id) => Self::new_id_only(id),
                With::Pair(id1, id2) => Self::new_pair_only(id1, id2),
            }
        }
    }

    pub(crate) const fn new_from_existing(world: *mut WorldT, id: IdT) -> Self {
        Self {
            world: world,
            raw_id: id,
        }
    }

    pub(crate) const fn new_id_only(id: IdT) -> Self {
        Self {
            world: std::ptr::null_mut(),
            raw_id: id,
        }
    }

    pub(crate) fn new_world_pair(world: *mut WorldT, first: IdT, second: IdT) -> Self {
        Self {
            world,
            raw_id: ecs_pair(first, second),
        }
    }

    pub(crate) fn new_pair_only(first: IdT, second: IdT) -> Self {
        Self {
            world: std::ptr::null_mut(),
            raw_id: ecs_pair(first, second),
        }
    }

    pub(crate) fn new_from_ids(id: Id, id2: Id) -> Self {
        Self {
            world: id.world,
            raw_id: ecs_pair(id.raw_id, id2.raw_id),
        }
    }

    pub fn is_pair(&self) -> bool {
        unsafe { ecs_id_is_pair(self.raw_id) }
    }

    pub fn is_wildcard(&self) -> bool {
        unsafe { ecs_id_is_wildcard(self.raw_id) }
    }

    pub fn is_entity(&self) -> bool {
        self.raw_id & RUST_ECS_ID_FLAGS_MASK == 0
    }

    /// Return id as entity (only allowed when id is valid entity)
    #[inline(always)]
    pub fn entity(&self) -> Entity {
        {
            ecs_assert!(!self.is_pair(), FlecsErrorCode::InvalidOperation);
            ecs_assert!(
                self.flags().id.raw_id == 0,
                FlecsErrorCode::InvalidOperation
            );
        }
        Entity::new_from_existing(self.world, self.raw_id)
    }

    /// Return id with role added
    #[inline(always)]
    pub fn add_flags(&self, flags: IdT) -> Entity {
        Entity::new_from_existing(self.world, self.raw_id | flags)
    }

    /// Return id without role
    #[inline(always)]
    pub fn remove_flags_checked(&self, _flags: IdT) -> Entity {
        ecs_assert!(
            self.raw_id & RUST_ECS_ID_FLAGS_MASK == _flags,
            FlecsErrorCode::InvalidParameter
        );

        Entity::new_from_existing(self.world, self.raw_id & RUST_ECS_COMPONENT_MASK)
    }

    /// Return id without role
    #[inline(always)]
    pub fn remove_flags(&self) -> Entity {
        Entity::new_from_existing(self.world, self.raw_id & RUST_ECS_COMPONENT_MASK)
    }

    /// Return id flags set on id
    #[inline(always)]
    pub fn flags(&self) -> Entity {
        Entity::new_from_existing(self.world, self.raw_id & RUST_ECS_ID_FLAGS_MASK)
    }

    /// Test if id has specified role
    #[inline(always)]
    pub fn has_flags_for(&self, flags: IdT) -> bool {
        self.raw_id & flags == flags
    }

    /// Test if id has any role
    #[inline(always)]
    pub fn has_flags_any(&self) -> bool {
        self.raw_id & RUST_ECS_ID_FLAGS_MASK != 0
    }

    /// Return id without role
    #[inline(always)]
    pub fn remove_generation(&self) -> Entity {
        Entity::new_from_existing(self.world, self.raw_id as u32 as u64)
    }

    /// Return component type of id
    #[inline(always)]
    pub fn type_id(&self) -> Entity {
        Entity::new_from_existing(self.world, unsafe {
            ecs_get_typeid(self.world, self.raw_id)
        })
    }

    /// Test if id has specified first
    #[inline(always)]
    pub fn has_relationship(&self, first: IdT) -> bool {
        if !self.is_pair() {
            return false;
        }

        ecs_pair_first(self.raw_id) == first
    }

    /// Get first element from a pair.
    ///
    /// If the id is not a pair, this operation will fail. When the id has a
    /// world, the operation will ensure that the returned id has the correct generation count.
    #[inline(always)]
    pub fn first(&self) -> Entity {
        ecs_assert!(self.is_pair(), FlecsErrorCode::InvalidOperation);

        let entity = ecs_pair_first(self.raw_id);

        if self.world.is_null() {
            Entity::new_id_only(entity)
        } else {
            Entity::new_from_existing(self.world, unsafe { ecs_get_alive(self.world, entity) })
        }
    }

    /// Get second element from a pair.
    ///
    /// If the id is not a pair, this operation will fail. When the id has a
    /// world, the operation will ensure that the returned id has the correct generation count.
    pub fn second(&self) -> Entity {
        //TODO add the assert to cpp flecs
        ecs_assert!(self.is_pair(), FlecsErrorCode::InvalidOperation);

        let entity = ecs_pair_second(self.raw_id);

        if self.world.is_null() {
            Entity::new_id_only(entity)
        } else {
            Entity::new_from_existing(self.world, unsafe { ecs_get_alive(self.world, entity) })
        }
    }

    /// Convert id to string
    #[inline(always)]
    pub fn to_str(&self) -> &'static str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        unsafe { std::ffi::CStr::from_ptr(ecs_id_str(self.world, self.raw_id)) }
            .to_str()
            .unwrap_or_else(|_| {
                ecs_assert!(
                    false,
                    FlecsErrorCode::UnwrapFailed,
                    "Failed to convert id to string (id: {})",
                    self.raw_id
                );
                "invalid_str_from_id"
            })
    }

    /// Convert id to string
    /// ### Safety
    /// safe version : 'to_str'
    /// This function is unsafe because it assumes that the id is valid.
    #[inline(always)]
    pub unsafe fn to_str_unchecked(&self) -> &'static str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        let c_str_ptr = unsafe { ecs_id_str(self.world, self.raw_id) };

        // SAFETY: We assume the C string is valid UTF-8. This is risky if not certain.
        unsafe { std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(c_str_ptr).to_bytes()) }
    }

    /// Convert role of id to string.
    #[inline(always)]
    pub fn to_flags_str(&self) -> &'static str {
        // SAFETY: We assume that `ecs_role_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_role_str never returns null, so we don't need to check for that.
        unsafe { std::ffi::CStr::from_ptr(ecs_id_flag_str(self.raw_id & RUST_ECS_ID_FLAGS_MASK)) }
            .to_str()
            .unwrap_or_else(|_| {
                ecs_assert!(
                    false,
                    FlecsErrorCode::UnwrapFailed,
                    "Failed to convert id to string (id: {})",
                    self.raw_id
                );
                "invalid_str_from_id"
            })
    }

    /// Convert role of id to string.
    /// ### Safety
    /// safe version : 'to_flags_str'
    /// This function is unsafe because it assumes that the id is valid.
    #[inline(always)]
    pub unsafe fn to_flags_str_unchecked(&self) -> &'static str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        let c_str_ptr = unsafe { ecs_id_flag_str(self.raw_id & RUST_ECS_ID_FLAGS_MASK) };

        // SAFETY: We assume the C string is valid UTF-8. This is risky if not certain.
        unsafe { std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(c_str_ptr).to_bytes()) }
    }

    pub fn get_as_world(&self) -> World {
        World {
            raw_world: self.world,
            is_owned: false,
        }
    }
}
