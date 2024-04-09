use std::ops::Deref;

use super::{c_types::EntityT, IntoEntityId, IntoWorld, WorldRef};
use crate::sys::ecs_set_scope;

/// Utility class used by the `world::scope` method to create entities in a scope
pub struct ScopedWorld<'a> {
    pub world: WorldRef<'a>,
    pub prev_scope: EntityT,
}

impl<'a> Deref for ScopedWorld<'a> {
    type Target = WorldRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl<'a> ScopedWorld<'a> {
    /// Creates a new scoped world
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the scope in
    /// * `scope` - The entity to scope to
    ///
    /// # See also
    ///
    /// * C++ API: `scoped_world::scoped_world`
    #[doc(alias = "scoped_world::scoped_world")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(world: impl IntoWorld<'a>, scope: impl IntoEntityId) -> Self {
        let prev_scope = unsafe { ecs_set_scope(world.world_ptr_mut(), scope.get_id()) };
        Self {
            world: world.world_ref(),
            prev_scope,
        }
    }

    /// Creates a new scoped world
    ///
    /// # Arguments
    ///
    /// * `scoped_world` - The scoped world to create the scope from
    ///
    /// # See also
    ///
    /// * C++ API: `scoped_world::scoped_world`
    #[doc(alias = "scoped_world::scoped_world")]
    pub fn new_from_scoped_world(scoped_world: &'a ScopedWorld) -> Self {
        let prev_scope = scoped_world.prev_scope;
        Self::new(scoped_world.world, prev_scope)
    }
}

impl<'a> Drop for ScopedWorld<'a> {
    /// Restores the previous scope
    ///
    /// # See also
    ///
    /// * C++ API: `scoped_world::~scoped_world`
    #[doc(alias = "scoped_world::~scoped_world")]
    fn drop(&mut self) {
        unsafe { ecs_set_scope(self.world.world_ptr_mut(), self.prev_scope) };
    }
}
