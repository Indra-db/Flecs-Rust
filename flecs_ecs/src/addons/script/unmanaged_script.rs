use std::ffi::CStr;

use flecs_ecs::core::*;
use flecs_ecs::sys;

/// A Script object is not associated to an entity and will be automatically deleted when it goes out of scope.
/// For scripts that are associated with an entity, use [`ScriptBuilder`][super::ScriptBuilder] alongside [`ScriptEntityView`][super::ScriptEntityView].
///
/// # Safety
///
/// Assemblies/Templates created by the script rely upon resources in the script object,
/// and for that reason keep the script alive until all assemblies created by the script are deleted.
#[repr(C)]
pub struct Script<'a> {
    script: *mut sys::ecs_script_t,
    ast: *mut std::ffi::c_char,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Drop for Script<'a> {
    fn drop(&mut self) {
        if !self.ast.is_null() {
            unsafe {
                sys::ecs_os_api.free_.expect("os api is missing")(
                    self.ast as *mut std::ffi::c_void,
                );
            }
        }
        if !self.script.is_null() {
            unsafe { sys::ecs_script_free(self.script) }
        }
    }
}

impl<'a> Script<'a> {
    /// Parses and creates new script dynamically.
    /// This operation parses a script and returns a script object upon success.
    ///  To run the script, call `eval()`.
    ///
    /// # Arguments
    ///
    /// * name - Name of the script (typically a file/module name).
    ///
    /// * code - The script code.
    ///
    /// # See also
    ///
    /// * C API: `ecs_script_parse`
    #[doc(alias = "ecs_script_parse")]
    pub fn parse(world: impl WorldProvider<'a>, name: &str, code: &str) -> Option<Script<'a>> {
        let name = compact_str::format_compact!("{}\0", name);
        let code = compact_str::format_compact!("{}\0", code);
        let world_ptr = world.world_ptr_mut();

        let ptr = unsafe {
            sys::ecs_script_parse(
                world_ptr,
                name.as_ptr() as *const _,
                code.as_ptr() as *const _,
            )
        };
        if ptr.is_null() {
            None
        } else {
            Some(Script {
                script: ptr,
                ast: std::ptr::null_mut(),
                _phantom: std::marker::PhantomData::<&'a ()>,
            })
        }
    }

    /// Evaluate script. This operation evaluates (runs) a parsed script.
    ///
    /// # Returns
    ///
    /// True if success, false if failed.
    ///
    /// # See also
    ///
    /// * C API: `ecs_script_eval`
    #[doc(alias = "ecs_script_eval")]
    pub fn eval(&self) -> bool {
        unsafe { sys::ecs_script_eval(self.script) == 0 }
    }

    pub fn destroy(self) {
        // Drop
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
    pub fn run_code(world: impl WorldProvider<'a>, name: &str, code: &str) -> bool {
        let name = compact_str::format_compact!("{}\0", name);
        let code = compact_str::format_compact!("{}\0", code);
        let world_ptr = world.world_ptr_mut();

        unsafe {
            sys::ecs_script_run(
                world_ptr,
                name.as_ptr() as *const _,
                code.as_ptr() as *const _,
            ) == 0
        }
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
    pub fn run_file(world: impl WorldProvider<'a>, filename: &str) -> bool {
        let filename = compact_str::format_compact!("{}\0", filename);
        let world_ptr = world.world_ptr_mut();

        unsafe { sys::ecs_script_run_file(world_ptr, filename.as_ptr() as *const _) == 0 }
    }

    /// Convert script AST to string.
    /// This operation converts the script abstract syntax tree to a string, which can be used to debug a script.
    ///
    ///
    /// # Returns
    ///
    /// Some String if success, None if failed.
    ///
    /// # See also
    ///
    /// * C API: `script_ast_to_buf`
    #[doc(alias = "script_ast_to_buf")]
    pub fn ast(&mut self) -> Option<String> {
        let ast = unsafe { sys::ecs_script_ast_to_str(self.script) };

        if !ast.is_null() {
            if self.ast.is_null() {
                self.ast = ast;
            } else {
                ecs_assert!(
                    false,
                    FlecsErrorCode::InvalidOperation,
                    "Script AST already exists"
                );
                unsafe {
                    sys::ecs_os_api.free_.expect("os api is missing")(ast as *mut std::ffi::c_void);
                }
            }
            let c_str = unsafe { CStr::from_ptr(ast) };
            let str = c_str.to_str().unwrap().to_owned();
            unsafe {
                sys::ecs_os_api.free_.expect("os api is missing")(ast as *mut std::ffi::c_void);
            };
            Some(str)
        } else {
            None
        }
    }

    /// Serialize value into a String.
    /// This operation serializes a value of the provided type to a string.
    ///     
    /// # See also
    ///
    /// * C API: `ecs_ptr_to_expr`
    /// * C++ API: `world::to_expr`
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn to_expr_id(
        world: impl WorldProvider<'a>,
        id_of_value: impl Into<Entity>,
        value: *const std::ffi::c_void,
    ) -> String {
        let id = *id_of_value.into();
        let world = world.world_ptr_mut();
        let expr = unsafe { sys::ecs_ptr_to_expr(world, id, value) };
        let c_str = unsafe { CStr::from_ptr(expr) };
        let str = c_str.to_str().unwrap().to_owned();
        unsafe { sys::ecs_os_api.free_.expect("os api is missing")(expr as *mut std::ffi::c_void) };
        str
    }

    /// Convert value to string
    ///
    /// # See also
    ///
    /// * C API: `ecs_ptr_to_expr`
    /// * C++ API: `world::to_expr`
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn to_expr<T: ComponentId>(world: impl WorldProvider<'a>, value: &T) -> String {
        let world = world.world();
        let id = T::get_id(world);
        Self::to_expr_id(world, id, value as *const T as *const std::ffi::c_void)
    }
}
