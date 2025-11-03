use core::ffi::CStr;

use flecs_ecs::core::*;
use flecs_ecs::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::{borrow::ToOwned, string::String};

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
    ast: *mut core::ffi::c_char,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl Drop for Script<'_> {
    fn drop(&mut self) {
        if !self.ast.is_null() {
            unsafe {
                sys::ecs_os_api.free_.expect("os api is missing")(
                    self.ast as *mut core::ffi::c_void,
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
    pub fn parse(
        world: impl WorldProvider<'a>,
        name: &str,
        code: &str,
        desc: Option<sys::ecs_script_eval_desc_t>,
    ) -> Option<Script<'a>> {
        let name = compact_str::format_compact!("{}\0", name);
        let code = compact_str::format_compact!("{}\0", code);
        let world_ptr = world.world_ptr_mut();

        let ptr = unsafe {
            if let Some(desc) = desc {
                sys::ecs_script_parse(
                    world_ptr,
                    name.as_ptr() as *const _,
                    code.as_ptr() as *const _,
                    &desc,
                    core::ptr::null_mut(),
                )
            } else {
                sys::ecs_script_parse(
                    world_ptr,
                    name.as_ptr() as *const _,
                    code.as_ptr() as *const _,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                )
            }
        };
        if ptr.is_null() {
            None
        } else {
            Some(Script {
                script: ptr,
                ast: core::ptr::null_mut(),
                _phantom: core::marker::PhantomData::<&'a ()>,
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
    pub fn eval(&self, desc: Option<sys::ecs_script_eval_desc_t>) -> bool {
        if let Some(desc) = desc {
            unsafe { sys::ecs_script_eval(self.script, &desc, core::ptr::null_mut()) == 0 }
        } else {
            unsafe {
                sys::ecs_script_eval(self.script, core::ptr::null(), core::ptr::null_mut()) == 0
            }
        }
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
    pub fn run_code(world: impl WorldProvider<'a>, name: &str, code: &str) -> bool {
        let name = compact_str::format_compact!("{}\0", name);
        let code = compact_str::format_compact!("{}\0", code);
        let world_ptr = world.world_ptr_mut();

        unsafe {
            sys::ecs_script_run(
                world_ptr,
                name.as_ptr() as *const _,
                code.as_ptr() as *const _,
                core::ptr::null_mut(),
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
    pub fn ast(&mut self) -> Option<String> {
        let ast = unsafe { sys::ecs_script_ast_to_str(self.script, false) };

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
                    sys::ecs_os_api.free_.expect("os api is missing")(
                        ast as *mut core::ffi::c_void,
                    );
                }
            }
            let c_str = unsafe { CStr::from_ptr(ast) };
            let str = c_str.to_str().unwrap().to_owned();
            unsafe {
                sys::ecs_os_api.free_.expect("os api is missing")(ast as *mut core::ffi::c_void);
            };
            Some(str)
        } else {
            None
        }
    }

    /// Serialize value into a String.
    /// This operation serializes a value of the provided type to a string.
    ///
    /// # Safety
    /// The caller must ensure that `value` points to valid data of the type specified by `id_of_value`.
    ///
    /// # See also
    ///
    /// * C API: `ecs_ptr_to_expr`
    pub unsafe fn to_expr<T: IntoEntity>(
        world: impl WorldProvider<'a>,
        id_of_value: T,
        value: *const T::CastType,
    ) -> String {
        let world = world.world();
        let id = *id_of_value.into_entity(world);
        let world = world.world_ptr_mut();
        let expr = unsafe { sys::ecs_ptr_to_expr(world, id, value as *const core::ffi::c_void) };
        let c_str = unsafe { CStr::from_ptr(expr) };
        let str = c_str.to_str().unwrap().to_owned();
        unsafe {
            sys::ecs_os_api.free_.expect("os api is missing")(expr as *mut core::ffi::c_void);
        };
        str
    }

    pub fn get_const_var(world: impl WorldProvider<'a>, name: &str) -> Option<sys::ecs_value_t> {
        let world_ptr = world.world_ptr();
        let name = compact_str::format_compact!("{}\0", name);
        let v = unsafe {
            sys::ecs_lookup_path_w_sep(
                world_ptr,
                0,
                name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                false,
            )
        };

        if v == 0 {
            // unresolved const variable
            None
        } else {
            let value = unsafe { sys::ecs_const_var_get(world_ptr, v) };

            if value.ptr.is_null() {
                // entity is not a const variable
                None
            } else {
                Some(value)
            }
        }
    }

    pub fn get_const_numeric<T: ConstNumeric>(
        world: impl WorldProvider<'a>,
        value: sys::ecs_value_t,
    ) -> T::ConstType {
        let world_ptr = world.world_ptr();

        let cur = unsafe { sys::ecs_meta_cursor(world_ptr, value.type_, value.ptr) };
        if T::IS_INT {
            let cur_value = unsafe { sys::ecs_meta_get_int(&cur) };
            unsafe { *(&cur_value as *const i64 as *const T::ConstType) }
        } else if T::IS_UINT {
            let cur_value = unsafe { sys::ecs_meta_get_uint(&cur) };
            unsafe { *(&cur_value as *const u64 as *const T::ConstType) }
        } else
        /* float */
        {
            let cur_value = unsafe { sys::ecs_meta_get_float(&cur) };
            unsafe { *(&cur_value as *const f64 as *const T::ConstType) }
        }
    }

    pub fn get_const_char(
        world: impl WorldProvider<'a>,
        value: sys::ecs_value_t,
    ) -> core::ffi::c_char {
        let world_ptr = world.world_ptr();

        let cur = unsafe { sys::ecs_meta_cursor(world_ptr, value.type_, value.ptr) };
        let cur_value = unsafe { sys::ecs_meta_get_char(&cur) };
        cur_value as core::ffi::c_char
    }

    pub fn get_const_str(world: impl WorldProvider<'a>, value: sys::ecs_value_t) -> String {
        let world_ptr = world.world_ptr();

        let cur = unsafe { sys::ecs_meta_cursor(world_ptr, value.type_, value.ptr) };
        let c_str = unsafe { sys::ecs_meta_get_string(&cur) };

        unsafe { CStr::from_ptr(c_str) }
            .to_str()
            .unwrap()
            .to_owned()
    }
}

pub trait ConstNumeric: Sized {
    type ConstType: Sized + Copy;
    const IS_INT: bool;
    const IS_UINT: bool;
    const IS_FLOAT: bool;
}

impl ConstNumeric for i8 {
    type ConstType = i8;
    const IS_INT: bool = true;
    const IS_UINT: bool = false;
    const IS_FLOAT: bool = false;
}
impl ConstNumeric for i16 {
    type ConstType = i16;
    const IS_INT: bool = true;
    const IS_UINT: bool = false;
    const IS_FLOAT: bool = false;
}
impl ConstNumeric for i32 {
    type ConstType = i32;
    const IS_INT: bool = true;
    const IS_UINT: bool = false;
    const IS_FLOAT: bool = false;
}
impl ConstNumeric for i64 {
    type ConstType = i64;
    const IS_INT: bool = true;
    const IS_UINT: bool = false;
    const IS_FLOAT: bool = false;
}
impl ConstNumeric for u8 {
    type ConstType = u8;
    const IS_INT: bool = false;
    const IS_UINT: bool = true;
    const IS_FLOAT: bool = false;
}
impl ConstNumeric for u16 {
    type ConstType = u16;
    const IS_INT: bool = false;
    const IS_UINT: bool = true;
    const IS_FLOAT: bool = false;
}
impl ConstNumeric for u32 {
    type ConstType = u32;
    const IS_INT: bool = false;
    const IS_UINT: bool = true;
    const IS_FLOAT: bool = false;
}
impl ConstNumeric for u64 {
    type ConstType = u64;
    const IS_INT: bool = false;
    const IS_UINT: bool = true;
    const IS_FLOAT: bool = false;
}
impl ConstNumeric for f32 {
    type ConstType = f32;
    const IS_INT: bool = false;
    const IS_UINT: bool = false;
    const IS_FLOAT: bool = true;
}
impl ConstNumeric for f64 {
    type ConstType = f64;
    const IS_INT: bool = false;
    const IS_UINT: bool = false;
    const IS_FLOAT: bool = true;
}
