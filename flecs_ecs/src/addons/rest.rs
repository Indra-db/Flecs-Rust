use crate::core::flecs::rest::Rest;
use crate::core::World;

use super::module::Module;

impl Module for Rest {
    fn module(world: &World) {
        world.set(Rest::default());
    }
}
