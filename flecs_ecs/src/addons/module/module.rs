//! Modules organize components, systems and more in reusable units of code.
//!
//! * To define a module, see [`Module`].
//! * To import a module, see [`World::import()`].
//! * To override the name of a module, see [`World::module()`].
use crate::core::*;

/// Define a module
///
/// # Examples:
///
/// ```
/// # use flecs_ecs::prelude::*;
/// #[derive(Component)]
/// struct MyModule;
///
/// impl Module for MyModule {
///     fn module(world: &World) {
///         world.module::<MyModule>("MyModule");
///
///         // Define components, systems, triggers, ... as usual. They will be
///         // automatically created inside the scope of the module.
///     }
/// }
/// ```
///
/// # See also
///
/// * [`addons::module`](crate::addons::module)
/// * [`World::import()`]
/// * [`World::module()`]
pub trait Module: ComponentId {
    /// Perform the module definition.
    ///
    /// This is invoked via [`World::import()`].
    ///
    /// This method should configure the components, systems, observers, and
    /// whatever else is needed for the proper functioning of this module.
    fn module(world: &World);
}
