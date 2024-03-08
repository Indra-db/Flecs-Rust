use std::ops::Deref;

use super::{c_types::EntityT, world::World};
use crate::sys::ecs_set_scope;

/// Utility class used by the `world::scope` method to create entities in a scope
pub struct ScopedWorld {
    pub world: World,
    pub prev_scope: EntityT,
}

impl Deref for ScopedWorld {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl ScopedWorld {
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
    pub fn new(world: &World, scope: EntityT) -> Self {
        let prev_scope = unsafe { ecs_set_scope(world.raw_world, scope) };
        Self {
            world: world.clone(),
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
    pub fn new_from_scoped_world(scoped_world: &ScopedWorld) -> Self {
        let prev_scope = scoped_world.prev_scope;
        Self::new(&scoped_world.world, prev_scope)
    }
}

impl Drop for ScopedWorld {
    /// Restores the previous scope
    ///
    /// # See also
    ///
    /// * C++ API: `scoped_world::~scoped_world`
    #[doc(alias = "scoped_world::~scoped_world")]
    fn drop(&mut self) {
        unsafe { ecs_set_scope(self.world.raw_world, self.prev_scope) };
    }
}
