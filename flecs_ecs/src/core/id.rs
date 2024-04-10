//! Class for working with entity, component, tag and pair ids.

use super::{
    c_types::{IdT, RUST_ecs_id_FLAGS_MASK, RUST_ECS_COMPONENT_MASK},
    ecs_pair_first,
    entity::Entity,
    IntoEntityId, IntoEntityIdExt, IntoWorld, WorldRef,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;
use crate::{
    core::ecs_pair_second,
    ecs_assert,
    sys::{ecs_get_typeid, ecs_id_flag_str, ecs_id_is_pair, ecs_id_is_wildcard, ecs_id_str},
};

/// Class for working with entity, component, tag and pair ids.
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
pub struct Id<'a> {
    pub(crate) world: WorldRef<'a>,
    pub raw_id: IdT,
}

impl<'a> PartialEq for Id<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_id == other.raw_id
    }
}

impl<'a> PartialOrd for Id<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.raw_id.cmp(&other.raw_id))
    }
}

impl<'a> Ord for Id<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw_id.cmp(&other.raw_id)
    }
}

impl<'a> std::ops::Deref for Id<'a> {
    type Target = u64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.raw_id
    }
}

impl<'a> Id<'a> {
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
    pub fn new(world: impl IntoWorld<'a>, id: impl IntoEntityIdExt) -> Self {
        Self {
            world: world.world(),
            raw_id: id.get_id(),
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
    pub fn is_pair(self) -> bool {
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
    pub fn is_wildcard(self) -> bool {
        unsafe { ecs_id_is_wildcard(self.raw_id) }
    }

    /// checks if the id is a entity
    ///
    /// # See also
    ///
    /// * C++ API: `id::is_entity`
    #[doc(alias = "id::is_entity")]
    pub fn is_entity(self) -> bool {
        self.raw_id & RUST_ecs_id_FLAGS_MASK == 0
    }

    /// Return id as entity (only allowed when id is valid entity)
    ///
    /// # See also
    ///
    /// * C++ API: `id::entity`
    #[doc(alias = "id::entity")]
    #[inline(always)]
    pub fn to_entity(self) -> Entity<'a> {
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
    ///
    /// # See also
    ///
    /// * C++ API: `id::add_flags`
    #[doc(alias = "id::add_flags")]
    #[inline(always)]
    pub fn add_flags(self, flags: IdT) -> Entity<'a> {
        Entity::new_from_existing(self.world, self.raw_id | flags)
    }

    /// Return id with role removed.
    /// This function checks if the id has the specified role, and if it does not, the function will assert.
    ///
    /// # See also
    ///
    /// * C++ API: `id::remove_flags`
    #[doc(alias = "id::remove_flags")]
    #[inline(always)]
    pub fn remove_flags_checked(self, _flags: IdT) -> Entity<'a> {
        ecs_assert!(
            self.raw_id & RUST_ecs_id_FLAGS_MASK == _flags,
            FlecsErrorCode::InvalidParameter
        );

        Entity::new_from_existing(self.world, self.raw_id & RUST_ECS_COMPONENT_MASK)
    }

    /// Return id with role removed
    ///
    /// # See also
    ///
    /// * C++ API: `id::remove_flags`
    #[doc(alias = "id::remove_flags")]
    #[inline(always)]
    pub fn remove_flags(self) -> Entity<'a> {
        Entity::new_from_existing(self.world, self.raw_id & RUST_ECS_COMPONENT_MASK)
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
    pub fn flags(self) -> Entity<'a> {
        Entity::new_from_existing(self.world, self.raw_id & RUST_ecs_id_FLAGS_MASK)
    }

    /// Test if id has specified role
    ///
    /// # See also
    ///
    /// * C++ API: `id::has_flags`
    #[doc(alias = "id::has_flags")]
    #[inline(always)]
    pub fn has_flags_for(self, flags: IdT) -> bool {
        self.raw_id & flags == flags
    }

    /// Test if id has any role
    ///
    /// # See also
    ///
    /// * C++ API: `id::has_flags`
    #[doc(alias = "id::has_flags")]
    #[inline(always)]
    pub fn has_any_flags(self) -> bool {
        self.raw_id & RUST_ecs_id_FLAGS_MASK != 0
    }

    /// Return id without role
    ///
    /// # See also
    ///
    /// * C++ API: `id::remove_flags`
    #[doc(alias = "id::remove_flags")]
    #[inline(always)]
    pub fn remove_generation(self) -> Entity<'a> {
        Entity::new_from_existing(self.world, self.raw_id as u32 as u64)
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
    pub fn type_id(self) -> Entity<'a> {
        Entity::new_from_existing(self.world, unsafe {
            ecs_get_typeid(self.world.world_ptr_mut(), self.raw_id)
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
    pub fn has_relationship(self, first: impl IntoEntityId) -> bool {
        if !self.is_pair() {
            return false;
        }

        ecs_pair_first(self.raw_id) == first.get_id()
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
        self.world.get_alive(entity)
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
        ecs_assert!(self.is_pair(), FlecsErrorCode::InvalidOperation);

        let entity = ecs_pair_second(self.raw_id);
        self.world.get_alive(entity)
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
    pub fn to_str(self) -> &'a str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        if let Ok(str) =
            unsafe { std::ffi::CStr::from_ptr(ecs_id_str(self.world.world_ptr_mut(), self.raw_id)) }
                .to_str()
        {
            str
        } else {
            ecs_assert!(
                false,
                FlecsErrorCode::UnwrapFailed,
                "Failed to convert id to string (id: {})",
                self.raw_id
            );

            "invalid_str_from_id"
        }
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
    pub unsafe fn to_str_unchecked(self) -> &'a str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        let c_str_ptr = unsafe { ecs_id_str(self.world.world_ptr_mut(), self.raw_id) };

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
    pub fn flags_str(self) -> &'a str {
        // SAFETY: We assume that `ecs_role_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_role_str never returns null, so we don't need to check for that.
        unsafe { std::ffi::CStr::from_ptr(ecs_id_flag_str(self.raw_id & RUST_ecs_id_FLAGS_MASK)) }
            .to_str()
            .unwrap_or({
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
    pub unsafe fn to_flags_str_unchecked(self) -> &'a str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        let c_str_ptr = unsafe { ecs_id_flag_str(self.raw_id & RUST_ecs_id_FLAGS_MASK) };

        // SAFETY: We assume the C string is valid UTF-8. This is risky if not certain.
        unsafe { std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(c_str_ptr).to_bytes()) }
    }
}
