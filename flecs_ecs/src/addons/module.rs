//! Modules organize components, systems and more in reusable units of code.
//!
//! * To define a module, see [`Module`].
//! * To import a module, see [`World::import()`].
//! * To override the name of a module, see [`World::module()`].
use crate::core::{
    ComponentId, EntityView, FlecsConstantId, IdOperations, SEPARATOR, World, WorldProvider,
    ecs_pair, flecs, register_componment_data_explicit,
};
use crate::sys;

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

/// Module mixin implementation
impl World {
    /// Import a module.
    ///
    /// This operation will load a module. The module name will be used to verify if
    /// the module was already loaded, in which case it won't be reimported.
    ///
    /// Module contents will be stored as children of the module entity. This
    /// prevents modules from accidentally defining conflicting identifiers. This is
    /// enforced by setting the scope before and after loading the module to the
    /// module entity id.
    ///
    /// ```
    /// # use flecs_ecs::prelude::*;
    /// # #[derive(Component)]
    /// # struct MyModule;
    /// # impl Module for MyModule {
    /// #     fn module(_world: &World) {
    /// #     }
    /// # }
    /// # let world = World::new();
    /// world.import::<MyModule>();
    /// ```
    ///
    /// # See also
    ///
    /// * [`addons::module`](crate::addons::module)
    /// * [`Module`]
    /// * [`World::module()`]
    /// * C++ API: `world::import`
    pub fn import<T: Module>(&self) -> EntityView {
        // Reset scope
        let prev_scope = self.set_scope_id(0);

        let module = if T::is_registered_with_world(self) {
            self.component::<T>().entity
        } else {
            let id = self.entity_from_id(register_componment_data_explicit::<T, true>(
                self.raw_world.as_ptr(),
                core::ptr::null(),
            ));
            let id_u64 = *id.id();
            let index = T::index() as usize;
            let components_array = self.components_array();
            components_array[index] = id_u64;
            #[cfg(feature = "flecs_meta")]
            {
                self.components_map()
                    .insert(core::any::TypeId::of::<Self>(), id_u64);
            }
            id
        };

        // If we have already registered this type don't re-create the module
        if module.has::<flecs::Module>() {
            return module;
        }

        // Make module component sparse so that it'll never move in memory. This
        // guarantees that a module drop / destructor can be reliably used to cleanup
        // module resources.
        module.add_trait::<flecs::Sparse>();

        // Set scope to our module
        self.set_scope_id(module);

        // Build the module
        T::module(self);

        // Return out scope to the previous scope
        self.set_scope_id(prev_scope);

        // Initialise component for the module and add Module tag
        module.add::<flecs::Module>();

        module
    }

    /// Define a module.
    ///
    /// This operation is not mandatory, but can be called inside the module ctor to
    /// obtain the entity associated with the module, or override the module name.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The type of the module.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to give the module.
    ///
    /// # Returns
    ///
    /// The module entity.
    ///
    /// # See also
    ///
    /// * [`addons::module`](crate::addons::module)
    /// * [`Module`]
    /// * [`World::import()`]
    /// * C++ API: `world::module`
    pub fn module<M: ComponentId>(&self, name: &str) -> EntityView {
        let comp = self.component::<M>();
        let id = comp.id();

        if let Some(existing) = self.try_lookup_recursive(name) {
            self.set_scope_id(existing);
            return existing;
        }

        if !name.is_empty() {
            let name = compact_str::format_compact!("{}\0", name);
            let prev_parent = comp.parent().unwrap_or(EntityView::new_null(self));
            unsafe {
                sys::ecs_add_path_w_sep(
                    self.raw_world.as_ptr(),
                    *id,
                    0,
                    name.as_ptr() as *const _,
                    SEPARATOR.as_ptr(),
                    SEPARATOR.as_ptr(),
                );
            }
            let parent = comp.parent().unwrap_or(EntityView::new_null(self));

            if parent != prev_parent {
                // Module was reparented, cleanup old parent(s)
                let mut cur = prev_parent;
                let mut next;

                while cur.id != 0 {
                    next = cur.parent().unwrap_or(EntityView::new_null(self));

                    let mut it = unsafe {
                        sys::ecs_each_id(
                            self.world_ptr(),
                            ecs_pair(flecs::ChildOf::ID, cur.id.into()),
                        )
                    };
                    if !unsafe { sys::ecs_iter_is_true(&mut it) } {
                        cur.destruct();

                        // Prevent increasing the generation count of the temporary
                        // parent. This allows entities created during
                        // initialization to keep non-recycled ids.
                        self.set_version(cur);
                    }

                    cur = next;

                    if cur.id == 0 {
                        break;
                    }
                }
            }
        }

        EntityView::new_from(self, *id)
    }
}
