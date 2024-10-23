use flecs_ecs::core::*;
use flecs_ecs::sys;

use super::ScriptEntityView;

/// [`ScriptBuilder`] is a builder pattern for creating scripts.
pub struct ScriptBuilder<'a> {
    script: sys::ecs_script_desc_t,
    world: WorldRef<'a>,
}

impl<'a> ScriptBuilder<'a> {
    /// Create a new script builder.
    /// This will create a new entity that is associated with a script.
    ///
    /// The entity will receive an [`EcsScript`][crate::sys::EcsScript] component.
    pub fn new(world: impl WorldProvider<'a>) -> Self {
        ScriptBuilder {
            script: sys::ecs_script_desc_t {
                entity: 0,
                filename: std::ptr::null(),
                code: std::ptr::null(),
            },
            world: world.world(),
        }
    }

    /// Create a new script builder with a name.
    /// This will create a new named entity that is associated with a script.
    ///
    /// The entity will receive an [`EcsScript`][crate::sys::EcsScript] component.
    pub fn new_named(world: impl WorldProvider<'a>, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);
        let entity_desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };
        ScriptBuilder {
            script: sys::ecs_script_desc_t {
                entity: unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &entity_desc) },
                filename: std::ptr::null(),
                code: std::ptr::null(),
            },
            world: world.world(),
        }
    }

    /// Create a new script builder that is associated with an entity.
    /// This will not create a new entity, but will associate the script with an existing entity.
    /// This is useful if you want to tie the lifetime of the script to an existing entity.
    ///
    /// The entity will set a (new) [`EcsScript`][crate::sys::EcsScript] component.
    pub fn new_from(world: impl WorldProvider<'a>, entity: impl Into<Entity>) -> Self {
        ScriptBuilder {
            script: sys::ecs_script_desc_t {
                entity: entity.into().into(),
                filename: std::ptr::null(),
                code: std::ptr::null(),
            },
            world: world.world(),
        }
    }

    /// Loads a managed script from a file into the ECS world.
    ///
    /// This function initializes an ECS script from a file specified by `filename`.
    ///
    /// # Arguments
    ///
    /// * `world` - A pointer to the ECS world.
    /// * `entity` - The entity handle associated with the script.
    /// * `filename` - The path to the script file as a string slice.
    ///
    /// # Returns
    ///
    /// Returns the script entity handle of the loaded script.
    ///
    /// # See also
    ///
    /// * C API: `ecs_script_init`
    #[doc(alias = "ecs_script_init")]
    pub fn build_from_file(&mut self, filename: &str) -> ScriptEntityView<'a> {
        let filename = compact_str::format_compact!("{}\0", filename);
        let world = self.world.world_ptr_mut();

        self.script.filename = filename.as_ptr() as *const _;

        let result = unsafe { sys::ecs_script_init(world, &self.script) };

        ScriptEntityView::new_from(self.world, result)
    }

    /// Loads a managed script from a code string into the ECS world.
    ///
    /// This function initializes an ECS script from a code string specified by `code`.
    ///
    /// # Arguments
    ///
    /// * `world` - A pointer to the ECS world.
    /// * `entity` - The entity handle associated with the script.
    /// * `code` - The script code as a string slice.
    ///
    /// # Returns
    ///
    /// Returns the script entity handle of the loaded script.
    ///
    /// # See also
    ///
    /// * C API: `ecs_script_init`
    #[doc(alias = "ecs_script_init")]
    pub fn build_from_code(&mut self, code: &str) -> ScriptEntityView<'a> {
        let code = compact_str::format_compact!("{}\0", code);
        let world = self.world.world_ptr_mut();

        self.script.code = code.as_ptr() as *const _;

        let result = unsafe { sys::ecs_script_init(world, &self.script) };

        ScriptEntityView::new_from(self.world, result)
    }
}
