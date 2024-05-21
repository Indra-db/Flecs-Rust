use std::ffi::CStr;

use flecs_ecs::core::*;
use flecs_ecs::sys;

#[repr(C)]
/// # Safety
///
/// Assemblies/Templates created by the script rely upon resources in the script object,
/// and for that reason keep the script alive until all assemblies created by the script are deleted.
#[derive(flecs_ecs_derive::Component)]
struct Script<'a> {
    script: *mut sys::ecs_script_t,
    ast: *mut i8,
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
    pub fn parse(world: impl IntoWorld<'a>, name: &str, code: &str) -> Option<Script<'a>> {
        let name = compact_str::format_compact!("{}\0", name);
        let code = compact_str::format_compact!("{}\0", code);
        let world_ptr = world.world_ptr_mut();

        let ptr = unsafe {
            sys::ecs_script_parse(
                world_ptr,
                name.as_ptr() as *const i8,
                code.as_ptr() as *const i8,
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
    pub fn run(world: impl IntoWorld<'a>, name: &str, code: &str) -> bool {
        let name = compact_str::format_compact!("{}\0", name);
        let code = compact_str::format_compact!("{}\0", code);
        let world_ptr = world.world_ptr_mut();

        unsafe {
            sys::ecs_script_run(
                world_ptr,
                name.as_ptr() as *const i8,
                code.as_ptr() as *const i8,
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
    pub fn run_file(world: impl IntoWorld<'a>, filename: &str) -> bool {
        let filename = compact_str::format_compact!("{}\0", filename);
        let world_ptr = world.world_ptr_mut();

        unsafe { sys::ecs_script_run_file(world_ptr, filename.as_ptr() as *const i8) == 0 }
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
    pub fn ast(&mut self) -> Option<&str> {
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
            Some(c_str.to_str().unwrap())
        } else {
            None
        }
    }
}
