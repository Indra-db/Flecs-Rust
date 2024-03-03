use super::c_binding::bindings::*;
use super::c_types::*;
use super::ecs_pair;
use super::ecs_pair_first;
use super::entity::*;
use super::world::World;
use crate::core::ecs_pair_second;
use crate::core::FlecsErrorCode;
use crate::ecs_assert;

/// Class that wraps around a `flecs::id_t`
///
/// A flecs id is an identifier that can be added to entities. Ids can be:
///
/// * entities (including components, tags)
/// * pair ids
/// * entities with id flags set (like `flecs::Override`, `flecs::Toggle`)
///
/// # See also
///
/// * [flecs C++ documentation](https://www.flecs.dev/flecs/structflecs_1_1id.html#details)
/// * [flecs C documentation](https://www.flecs.dev/flecs/group__ids.html)
#[derive(Debug, Clone, Copy, Eq)]
pub struct Id {
    /// World is optional, but guarantees that entity identifiers extracted from the id are valid
    pub(crate) world: *mut WorldT,
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

pub enum IdType {
    Id(IdT),
    Pair(IdT, IdT),
}
impl Id {
    /// Wraps an id or pair
    ///
    /// # Arguments
    ///
    /// * `world` - The optional world to the id belongs to
    /// * `with` - The id or pair to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `Id::Id`
    #[doc(alias = "Id::Id")]
    /// * C API: `ecs_id_t`
    #[doc(alias = "ecs_id_t")]
    pub fn new(world: Option<&World>, with: IdType) -> Self {
        if let Some(world) = world {
            match with {
                IdType::Id(id) => Self::new_from_existing(world.raw_world, id),
                IdType::Pair(id1, id2) => Self::new_world_pair(world.raw_world, id1, id2),
            }
        } else {
            match with {
                IdType::Id(id) => Self::new_id_only(id),
                IdType::Pair(id1, id2) => Self::new_pair_only(id1, id2),
            }
        }
    }

    /// wraps a raw id
    ///
    /// # Arguments
    ///
    /// * `world` - The optional raw world to the id belongs to
    /// * `id` - The id to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `Id::Id`
    #[doc(alias = "Id::Id")]
    /// * C API: `ecs_id_t`
    #[doc(alias = "ecs_id_t")]
    pub(crate) const fn new_from_existing(world: *mut WorldT, id: IdT) -> Self {
        Self { world, raw_id: id }
    }

    /// wraps a raw id without a world
    ///
    /// # Arguments
    ///
    /// * `id` - The id to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `Id::Id`
    #[doc(alias = "Id::Id")]
    /// * C API: `ecs_id_t`
    #[doc(alias = "ecs_id_t")]
    pub(crate) const fn new_id_only(id: IdT) -> Self {
        Self {
            world: std::ptr::null_mut(),
            raw_id: id,
        }
    }

    /// wraps a pair of raw ids without an optional world
    ///
    /// # Arguments
    ///
    /// * `world` - The optional world to the id belongs to
    /// * `first` - The first id to wrap
    /// * `second` - The second id to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `Id::Id`
    #[doc(alias = "Id::Id")]
    /// * C API: `ecs_id_t`
    #[doc(alias = "ecs_id_t")]
    pub(crate) fn new_world_pair(world: *mut WorldT, first: IdT, second: IdT) -> Self {
        Self {
            world,
            raw_id: ecs_pair(first, second),
        }
    }

    /// wraps a pair of raw ids without a world
    ///
    /// # Arguments
    ///
    /// * `first` - The first id to wrap
    /// * `second` - The second id to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `Id::Id`
    #[doc(alias = "Id::Id")]
    /// * C API: `ecs_id_t`
    #[doc(alias = "ecs_id_t")]
    pub(crate) fn new_pair_only(first: IdT, second: IdT) -> Self {
        Self {
            world: std::ptr::null_mut(),
            raw_id: ecs_pair(first, second),
        }
    }

    /// wraps a pair of Ids
    ///
    /// # Arguments
    ///
    /// * `id` - The first id to wrap
    /// * `id2` - The second id to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `Id::Id`
    #[doc(alias = "Id::Id")]
    /// * C API: `ecs_id_t`
    #[doc(alias = "ecs_id_t")]
    pub(crate) fn new_from_ids(id: Id, id2: Id) -> Self {
        Self {
            world: id.world,
            raw_id: ecs_pair(id.raw_id, id2.raw_id),
        }
    }

    /// checks if the id is a pair
    ///
    /// # See also
    ///
    /// * C++ API: `id::is_pair`
    #[doc(alias = "id::is_pair")]
    /// * C API: `ecs_id_is_pair`
    #[doc(alias = "ecs_id_is_pair")]
    pub fn is_pair(&self) -> bool {
        unsafe { ecs_id_is_pair(self.raw_id) }
    }

    /// checks if the id is a wildcard
    ///
    /// # See also
    ///
    /// * C++ API: `id::is_wildcard`
    #[doc(alias = "id::is_wildcard")]
    /// * C API: `ecs_id_is_wildcard`
    #[doc(alias = "ecs_id_is_wildcard")]
    pub fn is_wildcard(&self) -> bool {
        unsafe { ecs_id_is_wildcard(self.raw_id) }
    }

    /// checks if the id is a entity
    ///
    /// # See also
    ///
    /// * C++ API: `id::is_entity`
    #[doc(alias = "id::is_entity")]
    pub fn is_entity(&self) -> bool {
        self.raw_id & RUST_ECS_ID_FLAGS_MASK == 0
    }

    /// Return id as entity (only allowed when id is valid entity)
    ///
    /// # See also
    ///
    /// * C++ API: `id::entity`
    #[doc(alias = "id::entity")]
    #[inline(always)]
    pub fn entity(&self) -> Entity {
        {
            ecs_assert!(!self.is_pair(), FlecsErrorCode::InvalidOperation);
            ecs_assert!(
                self.flags().id.raw_id == 0,
                FlecsErrorCode::InvalidOperation
            );
        }
        Entity::new_from_existing_raw(self.world, self.raw_id)
    }

    /// Return id with role added
    ///
    /// # See also
    ///
    /// * C++ API: `id::add_flags`
    #[doc(alias = "id::add_flags")]
    #[inline(always)]
    pub fn add_flags(&self, flags: IdT) -> Entity {
        Entity::new_from_existing_raw(self.world, self.raw_id | flags)
    }

    /// Return id with role removed.
    /// This function checks if the id has the specified role, and if it does not, the function will assert.
    ///
    /// # See also
    ///
    /// * C++ API: `id::remove_flags`
    #[doc(alias = "id::remove_flags")]
    #[inline(always)]
    pub fn remove_flags_checked(&self, _flags: IdT) -> Entity {
        ecs_assert!(
            self.raw_id & RUST_ECS_ID_FLAGS_MASK == _flags,
            FlecsErrorCode::InvalidParameter
        );

        Entity::new_from_existing_raw(self.world, self.raw_id & RUST_ECS_COMPONENT_MASK)
    }

    /// Return id with role removed
    ///
    /// # See also
    ///
    /// * C++ API: `id::remove_flags`
    #[doc(alias = "id::remove_flags")]
    #[inline(always)]
    pub fn remove_flags(&self) -> Entity {
        Entity::new_from_existing_raw(self.world, self.raw_id & RUST_ECS_COMPONENT_MASK)
    }

    /// Get flags associated with id
    ///
    /// # Returns
    ///
    /// The flags associated with the id or 0 Entity if the id is not in use
    ///
    /// # See also
    ///
    /// * C++ API: `id::flags`
    #[doc(alias = "id::flags")]
    #[inline(always)]
    pub fn flags(&self) -> Entity {
        Entity::new_from_existing_raw(self.world, self.raw_id & RUST_ECS_ID_FLAGS_MASK)
    }

    /// Test if id has specified role
    ///
    /// # See also
    ///
    /// * C++ API: `id::has_flags`
    #[doc(alias = "id::has_flags")]
    #[inline(always)]
    pub fn has_flags_for(&self, flags: IdT) -> bool {
        self.raw_id & flags == flags
    }

    /// Test if id has any role
    ///
    /// # See also
    ///
    /// * C++ API: `id::has_flags`
    #[doc(alias = "id::has_flags")]
    #[inline(always)]
    pub fn has_any_flags(&self) -> bool {
        self.raw_id & RUST_ECS_ID_FLAGS_MASK != 0
    }

    /// Return id without role
    ///
    /// # See also
    ///
    /// * C++ API: `id::remove_flags`
    #[doc(alias = "id::remove_flags")]
    #[inline(always)]
    pub fn remove_generation(&self) -> Entity {
        Entity::new_from_existing_raw(self.world, self.raw_id as u32 as u64)
    }

    /// Get the component type for the id.
    ///
    /// This operation returns the component id for an id,
    /// if the id is associated with a type. For a regular component with a non-zero size
    /// (an entity with the `EcsComponent` component) the operation will return the entity itself.
    /// For an entity that does not have the `EcsComponent` component, or with an `EcsComponent`
    /// value with size 0, the operation will return an Entity wrapping 0
    ///
    /// For a pair id the operation will return the type associated with the pair, by applying the following rules in order:
    ///
    /// * The first pair element is returned if it is a component
    /// * Entity wrapping 0 is returned if the relationship entity has the Tag property
    /// * The second pair element is returned if it is a component
    /// * Entity wrapping 0 is returned
    ///
    /// # Returns
    ///
    /// The type id of the id
    ///
    /// # See also
    ///
    /// * C++ API: `id::type_id`
    #[doc(alias = "id::type_id")]
    /// * C API: `ecs_get_typeid`
    #[doc(alias = "ecs_get_typeid")]
    #[inline(always)]
    pub fn type_id(&self) -> Entity {
        Entity::new_from_existing_raw(self.world, unsafe {
            ecs_get_typeid(self.world, self.raw_id)
        })
    }

    /// Test if id has specified first
    ///
    /// # Arguments
    ///
    /// * `first` - The first id to test
    ///
    /// # See also
    ///
    /// * C++ API: `id::has_relationship`
    #[doc(alias = "id::has_relationship")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `id::first`
    #[doc(alias = "id::first")]
    #[inline(always)]
    pub fn first(&self) -> Entity {
        ecs_assert!(self.is_pair(), FlecsErrorCode::InvalidOperation);

        let entity = ecs_pair_first(self.raw_id);

        if self.world.is_null() {
            Entity::new_id_only(entity)
        } else {
            Entity::new_from_existing_raw(self.world, unsafe { ecs_get_alive(self.world, entity) })
        }
    }

    /// Get second element from a pair.
    ///
    /// If the id is not a pair, this operation will fail. When the id has a
    /// world, the operation will ensure that the returned id has the correct generation count.
    ///
    /// # See also
    ///
    /// * C++ API: `id::second`
    #[doc(alias = "id::second")]
    pub fn second(&self) -> Entity {
        //TODO add the assert to cpp flecs
        ecs_assert!(self.is_pair(), FlecsErrorCode::InvalidOperation);

        let entity = ecs_pair_second(self.raw_id);

        if self.world.is_null() {
            Entity::new_id_only(entity)
        } else {
            Entity::new_from_existing_raw(self.world, unsafe { ecs_get_alive(self.world, entity) })
        }
    }

    /// Convert id to string
    ///
    /// # See also
    ///
    /// * C++ API: `id::str`
    #[doc(alias = "id::str")]
    /// * C API: `ecs_id_str`
    #[doc(alias = "ecs_id_str")]
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
    ///
    /// # Safety
    /// safe version : '`to_str`'
    /// This function is unsafe because it assumes that the id is valid.
    ///
    /// # See also
    ///
    /// * C++ API: `id::str`
    #[doc(alias = "id::str")]
    /// * C API: `ecs_id_str`
    #[doc(alias = "ecs_id_str")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `id::flag_str`
    #[doc(alias = "id::flag_str")]
    /// * C API: `ecs_id_flag_str`
    #[doc(alias = "ecs_id_flag_str")]
    #[inline(always)]
    pub fn flags_str(&self) -> &'static str {
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
    /// # Safety
    /// safe version : '`to_flags_str`'
    /// This function is unsafe because it assumes that the id is valid.
    ///
    /// # See also
    ///
    /// * C++ API: `id::flag_str`
    #[doc(alias = "id::flag_str")]
    /// * C API: `ecs_id_flag_str`
    #[doc(alias = "ecs_id_flag_str")]
    #[inline(always)]
    pub unsafe fn to_flags_str_unchecked(&self) -> &'static str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        let c_str_ptr = unsafe { ecs_id_flag_str(self.raw_id & RUST_ECS_ID_FLAGS_MASK) };

        // SAFETY: We assume the C string is valid UTF-8. This is risky if not certain.
        unsafe { std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(c_str_ptr).to_bytes()) }
    }

    pub fn get_world(&self) -> World {
        World {
            raw_world: self.world,
            is_owned: false,
        }
    }

    pub(crate) fn get_world_raw(&self) -> *mut WorldT {
        self.world
    }
}
