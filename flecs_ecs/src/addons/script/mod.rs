mod script_builder;
mod script_entity_view;
mod unmanaged_script;

pub use script_builder::*;
pub use script_entity_view::*;
pub use unmanaged_script::*;

use flecs_ecs::core::*;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::string::String;

/// Script mixin implementation
impl World {
    /// Create a new script builder.
    /// This will create a new entity that is associated with a script.
    ///
    /// The entity will receive an [`EcsScript`][crate::sys::EcsScript] component.
    pub fn script(&self) -> ScriptBuilder {
        ScriptBuilder::new(self)
    }

    /// Create a new script builder with a name.
    /// This will create a new named entity that is associated with a script.
    ///
    /// The entity will receive an [`EcsScript`][crate::sys::EcsScript] component.
    pub fn script_named(&self, name: &str) -> ScriptBuilder {
        ScriptBuilder::new_named(self, name)
    }

    /// Create a new script builder that is associated with an entity.
    /// This will not create a new entity, but will associate the script with an existing entity.
    /// This is useful if you want to tie the lifetime of the script to an existing entity.
    ///
    /// The entity will set a (new) [`EcsScript`][crate::sys::EcsScript] component.
    pub fn script_from(&self, entity: impl Into<Entity>) -> ScriptBuilder {
        ScriptBuilder::new_from(self, entity)
    }

    /// Parse script. This parses a script and instantiates the entities in the world.
    /// This operation is the equivalent to doing: [`parse`][flecs_ecs::addons::script::Script::parse], [`eval`][flecs_ecs::addons::script::Script::eval], [`destroy`][flecs_ecs::addons::script::Script::destroy].
    ///
    /// # Arguments
    ///
    /// * name - The script name (typically the file).
    ///
    /// * code - The script.
    ///
    /// # Returns
    ///
    /// True if success, false otherwise.
    ///
    /// # See also
    ///
    /// * C API: `ecs_script_run`
    pub fn run_code(&self, name: &str, code: &str) -> bool {
        Script::run_code(self, name, code)
    }

    /// Parse script file. This parses a script file and instantiates the entities in the world.
    /// This operation is equivalent to loading the file contents and passing it to `run`.
    ///
    /// # Arguments
    ///
    /// * filename - The script file name.
    ///
    /// # Returns
    ///
    /// True if success, false if failed.
    pub fn run_file(&self, filename: &str) -> bool {
        Script::run_file(self, filename)
    }

    /// Serialize value into a String.
    /// This operation serializes a value of the provided type to a string.
    ///     
    /// # See also
    ///
    /// * C API: `ecs_ptr_to_expr`
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn to_expr_id(
        &self,
        id_of_value: impl IntoEntity,
        value: *const core::ffi::c_void,
    ) -> String {
        Script::to_expr(self, id_of_value, value)
    }

    /// Serialize value into a String.
    /// This operation serializes a value of the provided type to a string.
    ///     
    /// # See also
    ///
    /// * C API: `ecs_ptr_to_expr`
    pub fn to_expr<T: ComponentId>(&self, value: &T) -> String {
        Script::to_expr(
            self,
            id::<T>(),
            value as *const T as *const core::ffi::c_void,
        )
    }

    /// Wraps the provided entity id in a [`ScriptEntityView`].
    ///
    /// # Panics
    ///
    /// The entity must have a [`flecs::Script`] component.
    pub fn script_entity_from(&self, id: impl IntoEntity) -> ScriptEntityView {
        ScriptEntityView::new_from(self, id)
    }
}
