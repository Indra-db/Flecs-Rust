#[cfg(not(target_family = "wasm"))]
use crate::core::{Entity, WorldRef};
// Shared type definitions for function pointers that need different ABIs for WASM vs non-WASM
use crate::sys;
use core::ffi::{c_char, c_void};

// Serializer functions
#[cfg(not(target_family = "wasm"))]
pub type SerializeFnPtr<T> = extern "C-unwind" fn(&sys::ecs_serializer_t, &T) -> i32;
#[cfg(target_family = "wasm")]
pub type SerializeFnPtr<T> = extern "C" fn(&sys::ecs_serializer_t, &T) -> i32;

#[cfg(not(target_family = "wasm"))]
pub type SerializeFnPtrUnsafe =
    unsafe extern "C-unwind" fn(*const sys::ecs_serializer_t, *const c_void) -> i32;
#[cfg(target_family = "wasm")]
pub type SerializeFnPtrUnsafe =
    unsafe extern "C" fn(*const sys::ecs_serializer_t, *const c_void) -> i32;

#[cfg(not(target_family = "wasm"))]
pub type SerializeMemberFnPtr<T> =
    extern "C-unwind" fn(&sys::ecs_serializer_t, &T, *const c_char) -> i32;
#[cfg(target_family = "wasm")]
pub type SerializeMemberFnPtr<T> = extern "C" fn(&sys::ecs_serializer_t, &T, *const c_char) -> i32;

#[cfg(not(target_family = "wasm"))]
pub type SerializeMemberFnPtrUnsafe =
    unsafe extern "C-unwind" fn(*const sys::ecs_serializer_t, *const c_void, *const c_char) -> i32;
#[cfg(target_family = "wasm")]
pub type SerializeMemberFnPtrUnsafe =
    unsafe extern "C" fn(*const sys::ecs_serializer_t, *const c_void, *const c_char) -> i32;

#[cfg(not(target_family = "wasm"))]
pub type SerializeElementFnPtr<T> = extern "C-unwind" fn(&sys::ecs_serializer_t, &T, usize) -> i32;
#[cfg(target_family = "wasm")]
pub type SerializeElementFnPtr<T> = extern "C" fn(&sys::ecs_serializer_t, &T, usize) -> i32;

#[cfg(not(target_family = "wasm"))]
pub type SerializeElementFnPtrUnsafe =
    unsafe extern "C-unwind" fn(*const sys::ecs_serializer_t, *const c_void, usize) -> i32;
#[cfg(target_family = "wasm")]
pub type SerializeElementFnPtrUnsafe =
    unsafe extern "C" fn(*const sys::ecs_serializer_t, *const c_void, usize) -> i32;

// Assignment functions
#[cfg(not(target_family = "wasm"))]
pub type AssignBoolFnPtr<T> = extern "C-unwind" fn(&mut T, bool);
#[cfg(target_family = "wasm")]
pub type AssignBoolFnPtr<T> = extern "C" fn(&mut T, bool);

#[cfg(not(target_family = "wasm"))]
pub type AssignBoolFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void, bool);
#[cfg(target_family = "wasm")]
pub type AssignBoolFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, bool);

#[cfg(not(target_family = "wasm"))]
pub type AssignCharFnPtr<T> = extern "C-unwind" fn(&mut T, c_char);
#[cfg(target_family = "wasm")]
pub type AssignCharFnPtr<T> = extern "C" fn(&mut T, c_char);

#[cfg(not(target_family = "wasm"))]
pub type AssignCharFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void, c_char);
#[cfg(target_family = "wasm")]
pub type AssignCharFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, c_char);

#[cfg(not(target_family = "wasm"))]
pub type AssignIntFnPtr<T> = extern "C-unwind" fn(&mut T, i64);
#[cfg(target_family = "wasm")]
pub type AssignIntFnPtr<T> = extern "C" fn(&mut T, i64);

#[cfg(not(target_family = "wasm"))]
pub type AssignIntFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void, i64);
#[cfg(target_family = "wasm")]
pub type AssignIntFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, i64);

#[cfg(not(target_family = "wasm"))]
pub type AssignUIntFnPtr<T> = extern "C-unwind" fn(&mut T, u64);
#[cfg(target_family = "wasm")]
pub type AssignUIntFnPtr<T> = extern "C" fn(&mut T, u64);

#[cfg(not(target_family = "wasm"))]
pub type AssignUIntFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void, u64);
#[cfg(target_family = "wasm")]
pub type AssignUIntFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, u64);

#[cfg(not(target_family = "wasm"))]
pub type AssignFloatFnPtr<T> = extern "C-unwind" fn(&mut T, f32);
#[cfg(target_family = "wasm")]
pub type AssignFloatFnPtr<T> = extern "C" fn(&mut T, f32);

#[cfg(not(target_family = "wasm"))]
pub type AssignFloatFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void, f64);
#[cfg(target_family = "wasm")]
pub type AssignFloatFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, f64);

#[cfg(not(target_family = "wasm"))]
pub type AssignStringFnPtr<T> = extern "C-unwind" fn(&mut T, *const c_char);
#[cfg(target_family = "wasm")]
pub type AssignStringFnPtr<T> = extern "C" fn(&mut T, *const c_char);

#[cfg(not(target_family = "wasm"))]
pub type AssignStringFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void, *const c_char);
#[cfg(target_family = "wasm")]
pub type AssignStringFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, *const c_char);

#[cfg(not(target_family = "wasm"))]
pub type AssignEntityFnPtr<'a, T> = extern "C-unwind" fn(&'a mut T, WorldRef<'a>, Entity);
#[cfg(target_family = "wasm")]
pub type AssignEntityFnPtr<'a, T> = extern "C" fn(&'a mut T, WorldRef<'a>, Entity);

#[cfg(not(target_family = "wasm"))]
pub type AssignEntityFnPtrUnsafe =
    unsafe extern "C-unwind" fn(*mut c_void, *mut sys::ecs_world_t, u64);
#[cfg(target_family = "wasm")]
pub type AssignEntityFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, *mut sys::ecs_world_t, u64);

#[cfg(not(target_family = "wasm"))]
pub type AssignNullFnPtr<T> = extern "C-unwind" fn(&mut T);
#[cfg(target_family = "wasm")]
pub type AssignNullFnPtr<T> = extern "C" fn(&mut T);

#[cfg(not(target_family = "wasm"))]
pub type AssignNullFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void);
#[cfg(target_family = "wasm")]
pub type AssignNullFnPtrUnsafe = unsafe extern "C" fn(*mut c_void);

#[cfg(not(target_family = "wasm"))]
pub type ClearFnPtr<T> = extern "C-unwind" fn(&mut T);
#[cfg(target_family = "wasm")]
pub type ClearFnPtr<T> = extern "C" fn(&mut T);

#[cfg(not(target_family = "wasm"))]
pub type ClearFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void);
#[cfg(target_family = "wasm")]
pub type ClearFnPtrUnsafe = unsafe extern "C" fn(*mut c_void);

// Element and member functions
#[cfg(not(target_family = "wasm"))]
pub type EnsureElementFnPtr<T, ElemType> = extern "C-unwind" fn(&mut T, usize) -> &mut ElemType;
#[cfg(target_family = "wasm")]
pub type EnsureElementFnPtr<T, ElemType> = extern "C" fn(&mut T, usize) -> &mut ElemType;

#[cfg(not(target_family = "wasm"))]
pub type EnsureElementFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void, usize) -> *mut c_void;
#[cfg(target_family = "wasm")]
pub type EnsureElementFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void;

#[cfg(not(target_family = "wasm"))]
pub type EnsureMemberFnPtr<T> = extern "C-unwind" fn(&mut T, *const c_char) -> *mut c_void;
#[cfg(target_family = "wasm")]
pub type EnsureMemberFnPtr<T> = extern "C" fn(&mut T, *const c_char) -> *mut c_void;

#[cfg(not(target_family = "wasm"))]
pub type EnsureMemberFnPtrUnsafe =
    unsafe extern "C-unwind" fn(*mut c_void, *const c_char) -> *mut c_void;
#[cfg(target_family = "wasm")]
pub type EnsureMemberFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;

// Utility functions
#[cfg(not(target_family = "wasm"))]
pub type CountFnPtr<T> = extern "C-unwind" fn(&mut T) -> usize;
#[cfg(target_family = "wasm")]
pub type CountFnPtr<T> = extern "C" fn(&mut T) -> usize;

#[cfg(not(target_family = "wasm"))]
pub type CountFnPtrUnsafe = unsafe extern "C-unwind" fn(*const c_void) -> usize;
#[cfg(target_family = "wasm")]
pub type CountFnPtrUnsafe = unsafe extern "C" fn(*const c_void) -> usize;

#[cfg(not(target_family = "wasm"))]
pub type ResizeFnPtr<T> = extern "C-unwind" fn(&mut T, usize);
#[cfg(target_family = "wasm")]
pub type ResizeFnPtr<T> = extern "C" fn(&mut T, usize);

#[cfg(not(target_family = "wasm"))]
pub type ResizeFnPtrUnsafe = unsafe extern "C-unwind" fn(*mut c_void, usize);
#[cfg(target_family = "wasm")]
pub type ResizeFnPtrUnsafe = unsafe extern "C" fn(*mut c_void, usize);
