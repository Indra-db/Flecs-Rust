use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
};

use crate::sys;
pub use entity_view_traits::*;
use flecs_ecs::core::*;
use sys::ecs_get_with;

use super::entity_view_traits;

/// `EntityView` is a wrapper around an entity id with the world. It provides methods to interact with entities.
#[derive(Clone, Copy)]
pub struct EntityView<'a> {
    pub(crate) world: WorldRef<'a>,
    pub(crate) id: Entity,
}

impl<'a> Deref for EntityView<'a> {
    type Target = Entity;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl<'a> DerefMut for EntityView<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}

impl<'a> std::fmt::Display for EntityView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.get_name() {
            write!(f, "{}", name)
        } else {
            write!(f, "{}", *self.id)
        }
    }
}

impl<'a> std::fmt::Debug for EntityView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();
        let id = self.id;
        let archetype_str = self
            .archetype()
            .to_string()
            .unwrap_or_else(|| "empty".to_string());
        write!(
            f,
            "Entity name: {} -- id: {} -- archetype: {}",
            name, id, archetype_str
        )
    }
}

impl<'a> EntityId for EntityView<'a> {
    fn entity_id(&self) -> Entity {
        self.id
    }
}

impl<'w> IsEntityView<'w> for EntityView<'w> {}

impl<'a> EntityView<'a> {
    /// Create new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub(crate) fn new(world: impl WorldProvider<'a>) -> Self {
        let world_ptr = world.world_ptr_mut();
        let id = if unsafe { sys::ecs_get_scope(world_ptr) == 0 && ecs_get_with(world_ptr) == 0 } {
            unsafe { sys::ecs_new(world_ptr) }
        } else {
            let desc = sys::ecs_entity_desc_t::default();
            unsafe { sys::ecs_entity_init(world_ptr, &desc) }
        };
        Self {
            world: world.world(),
            id: id.into(),
        }
    }

    /// Creates a wrapper around an existing entity / id.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    pub(crate) fn new_from(world: impl WorldProvider<'a>, id: impl Into<Entity>) -> Self {
        Self {
            world: world.world(),
            id: id.into(),
        }
    }

    /// Create a named entity.
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::entity`
    #[doc(alias = "entity::entity")]
    pub(crate) fn new_named(world: impl WorldProvider<'a>, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            parent: 0,
            symbol: std::ptr::null(),
            use_low_id: false,
            add: std::ptr::null(),
            add_expr: std::ptr::null(),
            set: std::ptr::null(),
        };
        let id = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &desc) };
        Self {
            world: world.world(),
            id: id.into(),
        }
    }

    pub(crate) fn new_named_cstr(world: impl WorldProvider<'a>, name: &CStr) -> Self {
        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            parent: 0,
            symbol: std::ptr::null(),
            use_low_id: false,
            add: std::ptr::null(),
            add_expr: std::ptr::null(),
            set: std::ptr::null(),
        };
        let id = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &desc) };
        Self {
            world: world.world(),
            id: id.into(),
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
    pub(crate) fn new_null(world: &'a World) -> EntityView<'a> {
        Self::new_from(world, 0)
    }
}

impl<'a> EntityView<'a> {
    /// Get Component from entity
    /// use `.unwrap()` or `.unwrap_unchecked()` or `get_unchecked()` if you're sure the entity has the component
    ///
    /// # Safety
    ///
    /// - This guarantees no safety with table locking that the reference cannot be invalidated by other operations.
    ///   Use with caution or use `try_get`, `get` variants.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::get`
    #[doc(alias = "entity_view::get")]
    #[inline(always)]
    pub fn try_get_unchecked<T: ComponentId>(self) -> Option<&'a T::UnderlyingType> {
        if !T::IS_ENUM {
            if T::IS_TAG {
                // ecs_assert!(
                //     false,
                //     FlecsErrorCode::InvalidParameter,
                //     "component {} has no size",
                //     std::any::type_name::<T>()
                // );
                // None

                let component_id = T::id(self.world);

                unsafe {
                    (sys::ecs_get_id(self.world.world_ptr(), *self.id, component_id)
                        as *const T::UnderlyingType)
                        .as_ref()
                }
            } else {
                let component_id = T::id(self.world);

                unsafe {
                    (sys::ecs_get_id(self.world.world_ptr(), *self.id, component_id)
                        as *const T::UnderlyingType)
                        .as_ref()
                }
            }
        } else {
            let component_id: sys::ecs_id_t = T::id(self.world);
            let target: sys::ecs_id_t =
                unsafe { sys::ecs_get_target(self.world.world_ptr(), *self.id, component_id, 0) };

            if target == 0 {
                // if there is no matching pair for (r,*), try just r
                unsafe {
                    (sys::ecs_get_id(self.world.world_ptr(), *self.id, component_id)
                        as *const T::UnderlyingType)
                        .as_ref()
                }
            } else {
                // get constant value from constant entity
                let constant_value = unsafe {
                    sys::ecs_get_id(self.world.world_ptr(), target, component_id)
                        as *const T::UnderlyingType
                };

                ecs_assert!(
                    !constant_value.is_null(),
                    FlecsErrorCode::InternalError,
                    "missing enum constant value {}",
                    std::any::type_name::<T>()
                );

                unsafe { constant_value.as_ref() }
            }
        }
    }
}
