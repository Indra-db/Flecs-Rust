use std::{ffi::CStr, ptr::NonNull};

use flecs_ecs_sys::ecs_world_t;

use crate::core::{
    ecs_assert, ecs_pair, ComponentId, ComponentType, DataComponent, EntityView, Enum,
    EnumComponentInfo, FlecsErrorCode, IntoId,
};
use crate::core::{Entity, WorldProvider, SEPARATOR};
use crate::sys;

use super::entity_id::EntityId;

pub(crate) trait PrivateConstEntityView:
    for<'a> WorldProvider<'a> + EntityId + Sized
{
    fn path_from_id_default_sep(&self, parent: impl Into<Entity>) -> Option<String> {
        NonNull::new(unsafe {
            sys::ecs_get_path_w_sep(
                self.world_ptr(),
                *parent.into(),
                *self.entity_id(),
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
            )
        })
        .map(|s| unsafe {
            let len = CStr::from_ptr(s.as_ptr()).to_bytes().len();
            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            String::from_utf8_unchecked(Vec::from_raw_parts(s.as_ptr() as *mut u8, len, len))
        })
    }

    /// Lookup an entity by name.
    ///
    /// Lookup an entity in the scope of this entity. The provided path may
    /// contain double colons as scope separators, for example: "`Foo::Bar`".
    ///
    /// # Arguments
    ///
    /// * `path` - The name of the entity to lookup.
    /// * `recursively` - Recursively traverse up the tree until entity is found.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise `None`.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::lookup`
    #[doc(alias = "entity_view::lookup")]
    #[inline(always)]
    fn try_lookup_impl(self, name: &str, recursively: bool) -> Option<EntityView<'_>> {
        let name = compact_str::format_compact!("{}\0", name);

        ecs_assert!(
            self.entity_id() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid lookup from null handle"
        );
        let id = unsafe {
            sys::ecs_lookup_path_w_sep(
                self.world_ptr(),
                *self.entity_id(),
                name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                recursively,
            )
        };

        if id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world(), id))
        }
    }

    //might not be needed, in the original c++ impl it was used in the get_mut functions.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::set_stage`
    #[doc(alias = "entity_view::set_stage")]
    #[doc(hidden)]
    fn set_stage<'a>(self, stage: impl WorldProvider<'a>) -> EntityView<'a> {
        EntityView::new_from(stage, *self.entity_id())
    }

    // TODO this needs a better name and documentation, the rest of the cpp functions still have to be done as well
    // TODO, I removed the second template parameter and changed the fn parameter second to entityT, check validity
    /// Get the target for a given pair of components and a relationship.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first component type to use for deriving the id.
    ///
    /// # Arguments
    ///
    /// * `second` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// * The entity for which the target has been found.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::target`
    #[doc(alias = "entity_view::target_for")]
    // TODO needs to be made safe
    fn target_for_first<First: ComponentId + DataComponent>(
        &self,
        second: impl Into<Entity>,
    ) -> *const First {
        let world = self.world();
        let comp_id = First::id(world);
        ecs_assert!(
            std::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "First element is size 0"
        );
        unsafe {
            sys::ecs_get_id(
                world.world_ptr(),
                comp_id,
                ecs_pair(comp_id, *second.into()),
            ) as *const First
        }
    }

    // this is pub(crate) because it's used for development purposes only
    fn has_enum_id<T>(self, enum_id: impl Into<Entity>, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let world = self.world();
        let enum_constant_entity_id = constant.id_variant(world);
        has_id(
            world.world_ptr(),
            self.entity_id(),
            (enum_id.into(), enum_constant_entity_id),
        )
    }
}

/// Test if an entity has an id.
///
/// # Arguments
///
/// * `entity` - The entity to check.
///
/// # Returns
///
/// True if the entity has or inherits the provided id, false otherwise.
///
/// # See also
///
/// * [`EntityView::has()`]
/// * C++ API: `entity_view::has`
#[doc(alias = "entity_view::has")]
#[inline(always)]
fn has_id(world: *const sys::ecs_world_t, own_id: Entity, id: impl IntoId) -> bool {
    unsafe { sys::ecs_has_id(world, *own_id, *id.into()) }
}
