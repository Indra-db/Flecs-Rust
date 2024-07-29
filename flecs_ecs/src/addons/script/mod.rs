mod script_builder;
mod script_entity_view;
mod unmanaged_script;

pub use script_builder::*;
pub use script_entity_view::*;
pub use unmanaged_script::*;

use flecs_ecs::core::*;

/// Script mixin implementation
impl World {
    /// Create a new script builder.
    pub fn script(&self) -> ScriptBuilder {
        ScriptBuilder::new(self)
    }

    /// Create a new script builder with a name.
    pub fn script_named(&self, name: &str) -> ScriptBuilder {
        ScriptBuilder::new_named(self, name)
    }

    /// Create a new script builder with a name.
    pub fn script_from(&self, entity: impl Into<Entity>) -> ScriptBuilder {
        ScriptBuilder::new_from(self, entity)
    }

    /// Parse script. This parses a script and instantiates the entities in the world.
    /// This operation is the equivalent to doing: `parse`, `eval`, `destroy`.
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
    #[doc(alias = "ecs_script_run")]
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
    ///
    /// # See also
    #[doc(alias = "ecs_script_run_file")]
    pub fn run_file(&self, filename: &str) -> bool {
        Script::run_file(self, filename)
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn to_expr_id(
        &self,
        id_of_value: impl Into<Entity>,
        value: *const std::ffi::c_void,
    ) -> String {
        Script::to_expr_id(self, id_of_value, value)
    }

    pub fn to_expr<T: ComponentId>(&self, value: &T) -> String {
        Script::to_expr(self, value)
    }

    /// Wraps the provided entity id in a [`ScriptEntityView`].
    pub fn script_entity_from_id(&self, id: impl Into<Entity>) -> ScriptEntityView {
        ScriptEntityView::new_from(self, id)
    }

    /// Wraps the provided entity in a [`ScriptEntityView`].
    pub fn script_entity_from<T: ComponentId>(&self) -> ScriptEntityView {
        ScriptEntityView::new_from(self, T::id(self))
    }
}
