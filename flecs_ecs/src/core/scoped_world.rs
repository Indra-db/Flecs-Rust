use std::ops::Deref;

use super::{
    c_binding::bindings::ecs_set_scope,
    c_types::{EntityT, WorldT},
    world::World,
};

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
    /// # Safety
    /// This function is unsafe because it assumes world is not nullptr
    /// this is highly unlikely a world would be nullptr, hence this function is not marked as unsafe.
    /// this will be changed in the future where we get rid of the pointers.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(world: *mut WorldT, scope: EntityT) -> Self {
        let prev_scope = unsafe { ecs_set_scope(world, scope) };
        let world = World {
            raw_world: world,
            is_owned: false,
        };
        Self { world, prev_scope }
    }

    pub fn new_from_scoped_world(scoped_world: &ScopedWorld) -> Self {
        let prev_scope = scoped_world.prev_scope;
        let world = World {
            raw_world: scoped_world.raw_world,
            is_owned: scoped_world.is_owned,
        };
        Self { world, prev_scope }
    }
}

impl Drop for ScopedWorld {
    fn drop(&mut self) {
        unsafe { ecs_set_scope(self.world.raw_world, self.prev_scope) };
    }
}
